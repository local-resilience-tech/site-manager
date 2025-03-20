use p2panda_core::{PrivateKey, PublicKey};
use p2panda_net::{RelayUrl, SystemEvent};
use p2panda_store::MemoryStore;
use rocket::tokio;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::toolkitty_node::messages::NetworkEvent;

use super::{
    context::Context,
    messages::ChannelEvent,
    node::Node,
    stream::StreamEvent,
    topic::{Topic, TopicMap},
};

pub struct Service {
    /// Handle onto the tauri application. The shared Context can be accessed and modified here.
    context: Arc<RwLock<Context>>,

    /// Stream where we receive all topic events from the p2panda node.
    stream_rx: mpsc::Receiver<StreamEvent>,

    /// Channel where we receive network status events from the p2panda node.
    network_events_rx: broadcast::Receiver<SystemEvent<Topic>>,

    /// Channel where we receive messages which should be forwarded up to the frontend.
    to_app_rx: broadcast::Receiver<ChannelEvent>,

    /// Channel where we receive the actual backend->frontend event channel.
    channel_rx: mpsc::Receiver<broadcast::Sender<ChannelEvent>>,
}

impl Service {
    /// Construct node, context and channels required for running the app service. Already
    /// subscribe to the invite codes topic.
    ///
    /// The node and several channel senders are added to the shared app context while channel
    /// receivers are stored on the Service struct for use during the runtime loop.
    pub async fn build(blobs_base_dir: PathBuf, bootstrap_node_id: Option<PublicKey>, relay_url: Option<RelayUrl>) -> anyhow::Result<Self> {
        let private_key = PrivateKey::new();
        let store = MemoryStore::new();
        let topic_map = TopicMap::new();

        let (node, stream_rx, network_events_rx) = Node::new(
            private_key.clone(),
            bootstrap_node_id,
            relay_url,
            store.clone(),
            blobs_base_dir,
            topic_map.clone(),
        )
        .await?;

        let (to_app_tx, to_app_rx) = broadcast::channel(32);
        let (channel_tx, channel_rx) = mpsc::channel(32);

        let context = Context::new(node, to_app_tx, topic_map, channel_tx);

        Ok(Self {
            context: Arc::new(RwLock::new(context)),
            stream_rx,
            network_events_rx,
            to_app_rx,
            channel_rx,
        })
    }

    /// Spawn the service task.
    #[cfg(not(test))]
    pub fn run() {
        use rocket::tokio::task;

        use super::rpc::Rpc;

        task::spawn(async move {
            let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");
            let mut app = Self::build(temp_blobs_root_dir.into_path(), None, None)
                .await
                .expect("build stream");
            let rpc = Rpc {
                context: app.context.clone(),
            };
            let channel = app
                .recv_channel()
                .await
                .expect("receive on channel rx");
            app.inner_run(channel)
                .await
                .expect("run stream task");
        });
    }

    /// Spawn the service task.
    #[cfg(test)]
    pub async fn run() -> Arc<RwLock<Context>> {
        let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");
        let mut app = Self::build(temp_blobs_root_dir.into_path(), None, None)
            .await
            .expect("build stream");
        let context = app.context.clone();
        let rt = tokio::runtime::Handle::current();

        rt.spawn(async move {
            let channel = app
                .recv_channel()
                .await
                .expect("receive on channel rx");
            app.inner_run(channel)
                .await
                .expect("run stream task");
        });

        context
    }

    /// Run the inner service loop which awaits events arriving on the app, network, stream and
    /// invite codes channels.
    pub(crate) async fn inner_run(mut self, mut channel: broadcast::Sender<ChannelEvent>) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Ok(event) = self.to_app_rx.recv() => {
                    channel.send(event)?;
                }
                Ok(event) = self.network_events_rx.recv() => {
                    channel.send(ChannelEvent::NetworkEvent(NetworkEvent(event)))?;
                },
                Some(event) = self.stream_rx.recv() => {
                    channel.send(ChannelEvent::Stream(event))?;
                },
                Some(new_channel) = self.channel_rx.recv() => {
                    channel = new_channel;
                },
                // @TODO(sam): Need a way to handle ephemeral topics in the stream controller as
                // we now don't have a static topic we can subscribe to on startup.
                //
                // Some(event) = self.invite_codes_rx.recv() => {
                //     let json = match event {
                //         FromNetwork::GossipMessage { bytes, .. } => {
                //             serde_json::from_slice(&bytes)?
                //         },
                //         FromNetwork::SyncMessage { .. } => unreachable!(),
                //     };
                //     channel.send(ChannelEvent::InviteCodes(json))?;
                // }
            }
        }
    }

    async fn recv_channel(&mut self) -> anyhow::Result<broadcast::Sender<ChannelEvent>> {
        let Some(channel) = self.channel_rx.recv().await else {
            return Err(anyhow::anyhow!("channel tx closed"));
        };

        self.context.write().await.channel_set = true;

        Ok(channel)
    }
}
