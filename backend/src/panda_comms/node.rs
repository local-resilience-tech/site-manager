use anyhow::Result;
use iroh::NodeAddr;
use p2panda_core::PublicKey;
use p2panda_net::{Network, NodeAddress, TopicId};
use p2panda_sync::TopicQuery;

pub struct Node<Topic> {
    network: Network<Topic>,
}

impl<Topic> Node<Topic>
where
    Topic: TopicQuery + TopicId + 'static,
{
    pub fn new(network: Network<Topic>) -> Self {
        Self { network }
    }

    pub fn node_id(&self) -> PublicKey {
        self.network.node_id()
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let endpoint = self.network.endpoint();
        endpoint.node_addr().await.unwrap()
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        self.network.known_peers().await
    }
}
