use anyhow::Result;
use iroh::NodeAddr;
use p2panda_core::{PrivateKey, PublicKey};
use p2panda_net::{FromNetwork, NodeAddress, RelayUrl, SystemEvent, ToNetwork};
use p2panda_store::MemoryStore;
use rocket::tokio;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};

use crate::toolkitty_node::{
    context::Context,
    extensions::{Extensions, LogId},
    messages::ChannelEvent,
    node::Node as ToolkittyNode,
    stream::StreamEvent,
    topic::{Topic, TopicMap},
};

pub struct Node {
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

// This Iroh relay node is hosted by Liebe Chaos for P2Panda development. It is not intended for
// production use, and LoRes tech will eventually provide a relay node for production use.
const RELAY_URL: &str = "https://staging-euw1-1.relay.iroh.network/";

impl Node {
    pub async fn new(
        network_name: String,
        private_key: PrivateKey,
        bootstrap_node_id: Option<PublicKey>,
        operation_store: MemoryStore<LogId, Extensions>,
    ) -> Result<Self> {
        let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");

        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();

        let topic_map = TopicMap::new();

        let (node, stream_rx, network_events_rx) = ToolkittyNode::new(
            network_name,
            private_key.clone(),
            bootstrap_node_id,
            Some(relay_url),
            operation_store.clone(),
            temp_blobs_root_dir.into_path(),
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

    pub async fn node_id(&self) -> PublicKey {
        let context = self.context.read().await;
        context.node.network.node_id()
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let context = self.context.read().await;
        let network = &context.node.network;
        let endpoint = network.endpoint();
        endpoint.node_addr().await.unwrap()
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        let context = self.context.read().await;
        context.node.network.known_peers().await
    }

    pub async fn subscribe(&self, topic: Topic) -> Result<(mpsc::Sender<ToNetwork>, mpsc::Receiver<FromNetwork>, oneshot::Receiver<()>)> {
        let context = self.context.read().await;
        context.node.network.subscribe(topic).await
    }

    pub async fn shutdown(&self) -> Result<()> {
        let context = self.context.read().await;
        context.node.network.clone().shutdown().await
    }
}
