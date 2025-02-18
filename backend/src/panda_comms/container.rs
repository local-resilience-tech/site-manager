use anyhow::Result;
use gethostname::gethostname;
use iroh::NodeAddr;
use p2panda_core::identity::PUBLIC_KEY_LEN;
use p2panda_core::{Body, Hash, PrivateKey, PublicKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{FromNetwork, Network, NetworkBuilder, NetworkId, NodeAddress, SyncConfiguration, ToNetwork, TopicId};
use p2panda_store::MemoryStore;
use p2panda_stream::operation::{ingest_operation, IngestResult};
use p2panda_stream::{DecodeExt, IngestExt};
use p2panda_sync::log_sync::LogSyncProtocol;
use rocket::tokio::sync::mpsc;
use rocket::tokio::{self, task};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use super::messages::Message;
use super::operations::{create_header, decode_gossip_message, encode_gossip_message, prepare_for_logging, CustomExtensions};
use super::site_messages::{SiteMessages, SiteRegistration};
// use super::sites::Sites;
use super::topics::{AuthorStore, ChatTopic, LogId};

pub struct DirectAddress {
    pub node_id: PublicKey,
    pub _addresses: Vec<SocketAddr>,
}

#[derive(Default)]
pub struct P2PandaContainer {
    network_name: Arc<Mutex<Option<String>>>,
    private_key: Arc<Mutex<Option<PrivateKey>>>,
    pub network: Arc<Mutex<Option<Network<ChatTopic>>>>,
}

impl P2PandaContainer {
    pub async fn set_network_name(&self, network_name: String) {
        let mut network_name_lock = self.network_name.lock().await;
        *network_name_lock = Some(network_name);
    }

    pub async fn get_network_name(&self) -> Option<String> {
        let network_name_lock = self.network_name.lock().await;
        network_name_lock.clone().or(None)
    }

    pub async fn set_private_key(&self, private_key: PrivateKey) {
        let mut private_key_lock = self.private_key.lock().await;
        *private_key_lock = Some(private_key);
    }

    pub async fn get_private_key(&self) -> Option<PrivateKey> {
        let private_key_lock = self.private_key.lock().await;
        private_key_lock.clone().or(None)
    }

    pub fn build_direct_address(&self, node_id_hex: String, ip4_address: String) -> Result<DirectAddress, anyhow::Error> {
        let node_id = build_public_key_from_hex(node_id_hex).ok_or(anyhow::Error::msg("Invalid node id"))?;
        let addr = SocketAddr::V4(SocketAddrV4::new(ip4_address.parse()?, 2022));
        let addresses: Vec<SocketAddr> = vec![addr];
        Ok(DirectAddress {
            node_id,
            _addresses: addresses,
        })
    }

    pub async fn start(&self, direct_address: Option<DirectAddress>) -> Result<()> {
        let site_name = get_site_name();
        println!("Starting client for site: {}", site_name);

        let private_key: Option<PrivateKey> = self.get_private_key().await;
        let network_name: Option<String> = self.get_network_name().await;

        if private_key.is_none() {
            println!("P2Panda: No private key found, not starting network");
            return Ok(());
        }

        if network_name.is_none() {
            println!("P2Panda: No network name found, not starting network");
            return Ok(());
        }

        let private_key = private_key.unwrap();
        let network_name = network_name.unwrap();

        self.start_for(site_name, private_key, network_name, direct_address)
            .await
    }

    async fn start_for(&self, site_name: String, private_key: PrivateKey, network_name: String, direct_address: Option<DirectAddress>) -> Result<()> {
        println!("P2Panda: Starting network: {}", network_name);

        let network_id: NetworkId = Hash::new(network_name).into();

        let topic = ChatTopic::new("site_management");

        // Bootstrap node details
        // let node_id = build_public_key_from_hex("073912eccc459a93f71b998373097d6e6bdd96ccffdab9be4d3da6ac6358030a".to_string()).unwrap();
        // let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(170, 64, 151, 138), 2022));
        // let addresses: Vec<SocketAddr> = vec![addr];

        let mut builder = NetworkBuilder::new(network_id)
            .private_key(private_key.clone())
            .discovery(LocalDiscovery::new());

        if let Some(direct_address) = direct_address {
            let DirectAddress { node_id, _addresses: _ } = direct_address;
            builder = builder.direct_address(node_id, vec![], None);
        }

        // Setup operations
        let operation_store = MemoryStore::<LogId, CustomExtensions>::new();
        let author_store = AuthorStore::new();
        let sync_protocol = LogSyncProtocol::new(author_store.clone(), operation_store.clone());
        let sync_config = SyncConfiguration::new(sync_protocol);
        builder = builder.sync(sync_config);

        // Create network
        let network: Network<ChatTopic> = builder.build().await?;

        self.setup_subscriptions(topic, &network, operation_store.clone(), site_name, private_key)
            .await?;

        // put the network in the container
        let mut network_lock = self.network.lock().await;
        *network_lock = Some(network);

        Ok(())
    }

    async fn setup_subscriptions(
        &self,
        topic: ChatTopic,
        network: &Network<ChatTopic>,
        operation_store: MemoryStore<[u8; 32], CustomExtensions>,
        site_name: String,
        private_key: PrivateKey,
    ) -> Result<(), anyhow::Error> {
        let (network_tx, network_rx, gossip_ready) = network.subscribe(topic.clone()).await?;

        task::spawn(async move {
            if gossip_ready.await.is_ok() {
                println!("- Joined gossip overlay");
            }
        });

        // let mut sites = Sites::build();

        let stream = ReceiverStream::new(network_rx);
        let stream = stream.filter_map(|event| match event {
            FromNetwork::GossipMessage { bytes, .. } => match decode_gossip_message(&bytes) {
                Ok(result) => Some(result),
                Err(err) => {
                    warn!("could not decode gossip message: {err}");
                    None
                }
            },
            FromNetwork::SyncMessage { header, payload, .. } => Some((header, payload)),
        });

        // Decode and ingest the p2panda operations.
        let mut stream = stream
            .decode()
            .filter_map(|result| match result {
                Ok(operation) => Some(operation),
                Err(err) => {
                    warn!("decode operation error: {err}");
                    None
                }
            })
            .ingest(operation_store.clone(), 128)
            .filter_map(|result| match result {
                Ok(operation) => Some(operation),
                Err(err) => {
                    warn!("ingest operation error: {err}");
                    None
                }
            });

        {
            task::spawn(async move {
                while let Some(operation) = stream.next().await {
                    println!("Received operation: {:?}", prepare_for_logging(operation));
                }
            });
        }

        // task::spawn(async move {
        //     while let Some(event) = network_rx.recv().await {
        //         handle_gossip_event(event, &mut sites);
        //     }
        // });

        let mut operation_store = operation_store.clone();
        let topic = topic.clone();

        // spawn a task to announce the site every 30 seconds
        task::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;

                let body = build_announce_site_body(&site_name);
                publish_operation(Some(body), &mut operation_store, topic.id(), &private_key, &network_tx)
                    .await
                    .ok();
            }
        });
        Ok(())
    }

    pub async fn get_public_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let network = self.network.lock().await;
        let network = network.as_ref().ok_or("Network not started")?;
        let node_id = network.node_id();
        Ok(node_id.to_string())
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let network = self.network.lock().await;
        let network = network.as_ref().unwrap();
        let endpoint = network.endpoint();
        let node_addr = endpoint.node_addr().await.unwrap();
        node_addr
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        let network = self.network.lock().await;
        let network = network.as_ref().unwrap();
        return network.known_peers().await;
    }
}

fn get_site_name() -> String {
    gethostname().to_string_lossy().to_string()
}

// fn handle_gossip_event(event: FromNetwork, sites: &mut Sites) {
//     match event {
//         FromNetwork::GossipMessage { bytes, .. } => match Message::decode(&bytes) {
//             Ok(message) => {
//                 handle_message(message, sites);
//             }
//             Err(err) => {
//                 eprintln!("Invalid gossip message: {}", err);
//             }
//         },
//         _ => panic!("no sync messages expected"),
//     }
// }

// fn handle_message(message: Message<SiteMessages>, sites: &mut Sites) {
//     match message.payload {
//         SiteMessages::SiteRegistration(registration) => {
//             println!("Received SiteRegistration: {:?}", registration);
//             sites.register(registration.name);
//             sites.log();
//         }
//         SiteMessages::SiteNotification(notification) => {
//             println!("Received SiteNotification: {:?}", notification);
//         }
//     }
// }

fn build_announce_site_body(name: &str) -> Body {
    let message = SiteMessages::SiteRegistration(SiteRegistration { name: name.to_string() });
    let bytes = Message::encode(message).unwrap();

    Body::new(&bytes)
}

async fn publish_operation(
    body: Option<Body>,
    operation_store: &mut MemoryStore<[u8; 32], CustomExtensions>,
    log_id: LogId,
    private_key: &PrivateKey,
    network_tx: &mpsc::Sender<ToNetwork>,
) -> Result<()> {
    println!("Announcing myself operation");

    let header = create_header(&mut operation_store.clone(), log_id, &private_key, body.clone(), false).await;

    let gossip_message_bytes: Vec<u8> = encode_gossip_message(&header, body.as_ref())?;
    let header_bytes = header.to_bytes();

    let ingest_result = ingest_operation(&mut operation_store.clone(), header, body, header_bytes, &log_id, false).await?;

    match ingest_result {
        IngestResult::Complete(operation) => {
            // author_store
            //     .add_author(Topic::new(log_id), operation.header.public_key)
            //     .await;

            println!("Ingested operation into store: {:?}", prepare_for_logging(operation));

            if network_tx
                .send(ToNetwork::Message { bytes: gossip_message_bytes })
                .await
                .is_err()
            {
                println!("Failed to send gossip message");
            } else {
                println!("Sent gossip message");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

// TODO: This should be in p2panda-core, submit a PR
fn build_public_key_from_hex(key_hex: String) -> Option<PublicKey> {
    let key_bytes = hex::decode(key_hex).ok()?;
    let key_byte_array: [u8; PUBLIC_KEY_LEN] = key_bytes.try_into().ok()?;
    let result = PublicKey::from_bytes(&key_byte_array);

    match result {
        Ok(key) => Some(key),
        Err(_) => None,
    }
}
