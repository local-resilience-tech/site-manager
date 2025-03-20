use anyhow::Result;
use iroh::NodeAddr;
use p2panda_core::{Hash, PrivateKey, PublicKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{FromNetwork, Network, NetworkBuilder, NetworkId, NodeAddress, RelayUrl, SyncConfiguration, ToNetwork, TopicId};
use p2panda_store::MemoryStore;
use p2panda_sync::log_sync::{LogSyncProtocol, TopicLogMap};
use rocket::tokio::sync::{mpsc, oneshot};

use crate::toolkitty_node::{
    extensions::{Extensions, LogId},
    node::Node as ToolkittyNode,
    topic::{Topic, TopicMap},
};

pub struct Node {
    toolkitty_node: ToolkittyNode<Topic>,
}

// This Iroh relay node is hosted by Liebe Chaos for P2Panda development. It is not intended for
// production use, and LoRes tech will eventually provide a relay node for production use.
const RELAY_URL: &str = "https://staging-euw1-1.relay.iroh.network/";

impl Node {
    pub async fn new<TM: TopicLogMap<Topic, LogId> + 'static>(
        network_name: String,
        private_key: PrivateKey,
        bootstrap_node_id: Option<PublicKey>,
        topic_map: TM,
        operation_store: MemoryStore<LogId, Extensions>,
    ) -> Result<Self> {
        println!("P2Panda: Starting network: {}", network_name);

        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();

        // New Toolkitty alternative
        let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");
        let (node, stream_rx, network_events_rx) = ToolkittyNode::new(
            private_key,
            bootstrap_node_id,
            Some(relay_url),
            operation_store,
            temp_blobs_root_dir.into_path(),
            topic_map,
        )
        .await?;

        Ok(Self { toolkitty_node: node })
    }

    pub fn node_id(&self) -> PublicKey {
        self.toolkitty_node.network.node_id()
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let endpoint = self.toolkitty_node.network.endpoint();
        endpoint.node_addr().await.unwrap()
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        self.toolkitty_node.network.known_peers().await
    }

    pub async fn subscribe(&self, topic: Topic) -> Result<(mpsc::Sender<ToNetwork>, mpsc::Receiver<FromNetwork>, oneshot::Receiver<()>)> {
        self.toolkitty_node.network.subscribe(topic).await
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.toolkitty_node
            .network
            .clone()
            .shutdown()
            .await
    }
}
