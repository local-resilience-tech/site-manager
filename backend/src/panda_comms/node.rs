use anyhow::Result;
use iroh::NodeAddr;
use p2panda_core::{Hash, PrivateKey, PublicKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{FromNetwork, Network, NetworkBuilder, NetworkId, NodeAddress, RelayUrl, SyncConfiguration, ToNetwork, TopicId};
use p2panda_store::MemoryStore;
use p2panda_sync::{
    log_sync::{LogSyncProtocol, TopicLogMap},
    TopicQuery,
};
use rocket::tokio::sync::{mpsc, oneshot};

use crate::panda_comms::{operations::CustomExtensions, topics::LogId};

pub struct Node<Topic> {
    network: Network<Topic>,
}

// This Iroh relay node is hosted by Liebe Chaos for P2Panda development. It is not intended for
// production use, and LoRes tech will eventually provide a relay node for production use.
const RELAY_URL: &str = "https://staging-euw1-1.relay.iroh.network/";

impl<Topic> Node<Topic>
where
    Topic: TopicQuery + TopicId + 'static,
{
    pub async fn new<TM: TopicLogMap<Topic, LogId> + 'static>(
        network_name: String,
        private_key: PrivateKey,
        bootstrap_node_id: Option<PublicKey>,
        topic_map: TM,
        operation_store: MemoryStore<[u8; 32], CustomExtensions>,
    ) -> Result<Self> {
        println!("P2Panda: Starting network: {}", network_name);

        let network_id: NetworkId = Hash::new(network_name).into();

        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();

        //let topic = ChatTopic::new("site_management");

        let mut builder: NetworkBuilder<Topic> = NetworkBuilder::new(network_id)
            .private_key(private_key.clone())
            .relay(relay_url.clone(), false, 0)
            .discovery(LocalDiscovery::new());

        if let Some(bootstrap_node_id) = bootstrap_node_id {
            println!("P2Panda: Direct address provided for peer: {}", bootstrap_node_id);
            builder = builder.direct_address(bootstrap_node_id, vec![], Some(relay_url));
        } else {
            // I am probably the bootstrap node since I know of no others
            println!("P2Panda: No direct address provided, starting as bootstrap node");
            builder = builder.bootstrap();
        }

        // Setup operations
        let sync_protocol = LogSyncProtocol::new(topic_map, operation_store.clone());
        let sync_config: SyncConfiguration<Topic> = SyncConfiguration::new(sync_protocol);
        builder = builder.sync(sync_config);

        // Create network
        let network: Network<Topic> = builder.build().await?;

        Ok(Self { network })
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

    pub async fn subscribe(&self, topic: Topic) -> Result<(mpsc::Sender<ToNetwork>, mpsc::Receiver<FromNetwork>, oneshot::Receiver<()>)> {
        self.network.subscribe(topic).await
    }
}
