use p2panda_core::{Hash, PublicKey};
use p2panda_net::TopicId;
use p2panda_sync::log_sync::TopicLogMap;
use rocket::tokio;
use serde::Serialize;
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};

use crate::toolkitty_node::{
    extensions::{Extensions, Stream},
    operation::create_operation,
};

use super::{
    context::Context,
    extensions::{LogId, LogPath},
    messages::{ChannelEvent, StreamArgs},
    node::{BlobError, PublishError},
    stream::StreamControllerError,
    topic::Topic,
};

pub struct Rpc {
    pub(crate) context: Arc<RwLock<Context>>,
}

impl Rpc {
    /// Initialize the app by passing it a channel from the frontend.
    pub async fn init(&self, channel: broadcast::Sender<ChannelEvent>) -> Result<(), RpcError> {
        let context = self.context.write().await;

        context
            .channel_tx
            .send(channel)
            .await
            .expect("send on channel");

        Ok(())
    }
    /// The public key of the local node.
    pub async fn public_key(&self) -> Result<PublicKey, RpcError> {
        let context = self.context.read().await;
        let public_key = context.node.private_key.public_key();
        Ok(public_key)
    }

    /// Acknowledge operations to mark them as successfully processed in the stream controller.
    pub async fn ack(&self, operation_id: Hash) -> Result<(), RpcError> {
        let mut context = self.context.write().await;
        context.node.ack(operation_id).await?;
        Ok(())
    }

    /// Replay any un-ack'd messages on a persisted topic.
    pub async fn replay(&self, topic: &str) -> Result<(), RpcError> {
        let mut context = self.context.write().await;
        let topic = Topic::Persisted(topic.to_string());
        if let Some(logs) = context.topic_map.get(&topic).await {
            context.node.replay(logs).await?;
        };
        Ok(())
    }

    /// Add a persisted topic to the topic log map.
    pub async fn add_topic_log(&self, public_key: &PublicKey, topic: &str, log_id: &LogId) -> Result<(), RpcError> {
        let context = self.context.write().await;
        let topic = Topic::Persisted(topic.to_string());
        context
            .topic_map
            .add_log(&topic, public_key, log_id)
            .await;
        Ok(())
    }

    /// Subscribe to a persisted topic.
    pub async fn subscribe_persisted(&self, topic: &str) -> Result<(), RpcError> {
        let topic = Topic::Persisted(topic.to_string());
        return self.subscribe(&topic).await;
    }

    /// Subscribe to a ephemeral topic.
    pub async fn subscribe_ephemeral(&self, topic: &str) -> Result<(), RpcError> {
        let topic = Topic::Ephemeral(topic.to_string());
        return self.subscribe(&topic).await;
    }

    async fn subscribe(&self, topic: &Topic) -> Result<(), RpcError> {
        let mut context = self.context.write().await;

        if context
            .subscriptions
            .insert(topic.id(), topic.clone())
            .is_some()
        {
            return Ok(());
        };

        match topic {
            Topic::Ephemeral(_) => {
                context
                    .node
                    .subscribe_ephemeral(topic)
                    .await
                    .expect("can subscribe to topic");
            }
            Topic::Persisted(_) => {
                context
                    .node
                    .subscribe_processed(topic)
                    .await
                    .expect("can subscribe to topic");
            }
        }

        context
            .to_app_tx
            .send(ChannelEvent::SubscribedToTopic(topic.clone()))?;

        Ok(())
    }

    /// Publish to a persisted topic.
    pub async fn publish_persisted(
        &self,
        payload: &[u8],
        stream_args: &StreamArgs,
        log_path: Option<&LogPath>,
        topic: Option<&str>,
    ) -> Result<(Hash, Hash), RpcError> {
        let mut context = self.context.write().await;
        let private_key = context.node.private_key.clone();

        let extensions = Extensions {
            stream_root_hash: stream_args.root_hash.map(Into::into),
            stream_owner: stream_args.owner.map(Into::into),
            log_path: log_path.cloned(),
            ..Default::default()
        };

        let (header, body) = create_operation(&mut context.node.store, &private_key, extensions, Some(payload)).await;

        match topic {
            Some(topic) => {
                let topic = Topic::Persisted(topic.to_string());
                context
                    .node
                    .publish_to_stream(&topic, &header, body.as_ref())
                    .await?;
            }
            None => {
                context
                    .node
                    .ingest(&header, body.as_ref())
                    .await?;
            }
        }

        debug!("publish operation: {}", header.hash());

        let stream: Stream = header
            .extension()
            .expect("extract stream extension");

        Ok((header.hash(), stream.id()))
    }

    /// Publish to an ephemeral topic.
    pub async fn publish_ephemeral(&self, topic: &str, payload: &[u8]) -> Result<(), RpcError> {
        let mut context = self.context.write().await;
        let topic = Topic::Ephemeral(topic.to_string());
        context
            .node
            .publish_ephemeral(&topic, payload)
            .await?;
        Ok(())
    }

    /// Upload a file.
    pub async fn upload_file(&self, path: PathBuf) -> Result<Hash, RpcError> {
        let context = self.context.read().await;
        let file_hash = context.node.upload_file(path).await?;
        Ok(file_hash)
    }
}

#[derive(Debug, Error)]
pub enum RpcError {
    #[error(transparent)]
    StreamController(#[from] StreamControllerError),

    #[error(transparent)]
    Publish(#[from] PublishError),

    #[error(transparent)]
    Blob(#[from] BlobError),

    #[error("payload decoding failed")]
    Serde(#[from] serde_json::Error),

    #[error("sending message on channel failed")]
    ChannelSender(#[from] tokio::sync::broadcast::error::SendError<ChannelEvent>),
}

impl Serialize for RpcError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
