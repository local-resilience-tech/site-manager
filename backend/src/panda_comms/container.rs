use anyhow::Result;
use gethostname::gethostname;
use iroh::NodeAddr;
use p2panda_core::identity::PUBLIC_KEY_LEN;
use p2panda_core::{Body, PrivateKey, PublicKey};
use p2panda_net::{FromNetwork, NodeAddress, ToNetwork, TopicId};
use p2panda_store::MemoryStore;
use p2panda_stream::operation::{ingest_operation, IngestResult};
use p2panda_stream::{DecodeExt, IngestExt};
use rocket::tokio::sync::mpsc;
use rocket::tokio::{self, task};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use super::messages::Message;
use super::node::Node;
use super::operations::{create_header, decode_gossip_message, encode_gossip_message, prepare_for_logging, CustomExtensions};
use super::site_messages::{SiteMessages, SiteRegistration};
// use super::sites::Sites;
use super::topics::{AuthorLogMap, ChatTopic, LogId};

pub struct DirectAddress {
    pub node_id: PublicKey,
    pub _addresses: Vec<SocketAddr>,
}

#[derive(Default)]
pub struct P2PandaContainer {
    network_name: Arc<Mutex<Option<String>>>,
    private_key: Arc<Mutex<Option<PrivateKey>>>,
    node: Arc<Mutex<Option<Node<ChatTopic>>>>,
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
        let boostrap_node_id = direct_address.map(|da| da.node_id);
        let author_log_map = AuthorLogMap::new();
        let operation_store = MemoryStore::<LogId, CustomExtensions>::new();
        let node = Node::new(
            network_name,
            private_key.clone(),
            boostrap_node_id,
            author_log_map.clone(),
            operation_store.clone(),
        )
        .await?;

        let topic = ChatTopic::new("site_management");
        self.setup_subscriptions(topic, &node, operation_store, author_log_map, site_name, private_key)
            .await?;

        // put the node in the container
        let mut node_lock = self.node.lock().await;
        *node_lock = Some(node);

        Ok(())
    }

    async fn setup_subscriptions(
        &self,
        topic: ChatTopic,
        node: &Node<ChatTopic>,
        operation_store: MemoryStore<[u8; 32], CustomExtensions>,
        author_log_map: AuthorLogMap,
        site_name: String,
        private_key: PrivateKey,
    ) -> Result<(), anyhow::Error> {
        let (network_tx, network_rx, gossip_ready) = node.subscribe(topic.clone()).await?;

        {
            let mut operation_store = operation_store.clone();
            let mut author_log_map = author_log_map.clone();
            let topic = topic.clone();

            task::spawn(async move {
                announce_site_regularly(site_name, &mut operation_store, &mut author_log_map, topic, &private_key, &network_tx).await;
                if gossip_ready.await.is_ok() {
                    println!("- JOINED GOSSIP NETWORK -");

                    // announce_site_regularly(site_name, &mut operation_store, &mut author_log_map, topic, &private_key, &network_tx);
                }
            });
        }

        // let mut sites = Sites::build();

        let stream = ReceiverStream::new(network_rx);
        let stream = stream.filter_map(|event| match event {
            FromNetwork::GossipMessage { bytes, .. } => match decode_gossip_message(&bytes) {
                Ok(result) => {
                    println!("Got gossip message");
                    Some(result)
                }
                Err(err) => {
                    println!("could not decode gossip message: {err}");
                    None
                }
            },
            FromNetwork::SyncMessage { header, payload, .. } => {
                println!("Got network message.");
                Some((header, payload))
            }
        });

        // Decode and ingest the p2panda operations.
        let mut stream = stream
            .decode()
            .filter_map(|result| match result {
                Ok(operation) => {
                    let header = operation.0.clone();

                    println!(
                        "- Decoded incoming operation: key={:?} timestamp={:?} seq_num={:?}",
                        header.public_key.to_string(),
                        header.timestamp,
                        header.seq_num
                    );
                    Some(operation)
                }
                Err(err) => {
                    println!("- decode operation error: {err}");
                    None
                }
            })
            .ingest(operation_store.clone(), 128)
            .filter_map(|result| match result {
                Ok(operation) => {
                    println!("- Ingested incoming operation");
                    Some(operation)
                }
                Err(err) => {
                    println!("- ingest operation error: {err}");
                    None
                }
            });

        {
            let mut author_log_map = author_log_map.clone();
            let topic = topic.clone();

            task::spawn(async move {
                while let Some(operation) = stream.next().await {
                    author_log_map
                        .add_author(topic.clone(), operation.header.public_key)
                        .await;

                    println!("+ Received operation: {:?}", prepare_for_logging(operation));
                }
            });
        }

        Ok(())
    }

    pub async fn get_public_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let node = self.node.lock().await;
        let node = node.as_ref().ok_or("Network not started")?;

        let node_id = node.node_id();
        Ok(node_id.to_string())
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let node = self.node.lock().await;
        let node = node.as_ref().unwrap();
        node.get_node_addr().await
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        let node = self.node.lock().await;
        let node = node.as_ref().unwrap();
        node.known_peers().await
    }
}

async fn announce_site_regularly(
    site_name: String,
    operation_store: &mut MemoryStore<[u8; 32], CustomExtensions>,
    author_log_map: &AuthorLogMap,
    topic: ChatTopic,
    private_key: &PrivateKey,
    network_tx: &mpsc::Sender<ToNetwork>,
) {
    let mut operation_store = operation_store.clone();
    let mut author_log_map = author_log_map.clone();
    let private_key = private_key.clone();
    let network_tx = network_tx.clone();

    // spawn a task to announce the site every 30 seconds
    task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            let body = build_announce_site_body(&site_name);
            publish_operation(
                Some(body),
                &mut operation_store,
                &mut author_log_map,
                topic.clone(),
                &private_key,
                &network_tx,
            )
            .await
            .ok();
        }
    });
}

fn get_site_name() -> String {
    gethostname().to_string_lossy().to_string()
}

fn build_announce_site_body(name: &str) -> Body {
    let message = SiteMessages::SiteRegistration(SiteRegistration { name: name.to_string() });
    let bytes = Message::encode(message).unwrap();

    Body::new(&bytes)
}

async fn publish_operation(
    body: Option<Body>,
    operation_store: &mut MemoryStore<[u8; 32], CustomExtensions>,
    author_log_map: &mut AuthorLogMap,
    topic: ChatTopic,
    private_key: &PrivateKey,
    network_tx: &mpsc::Sender<ToNetwork>,
) -> Result<()> {
    let header = create_header(&mut operation_store.clone(), topic.id(), &private_key, body.clone(), false).await;

    let gossip_message_bytes: Vec<u8> = encode_gossip_message(&header, body.as_ref())?;
    let header_bytes = header.to_bytes();

    let ingest_result = ingest_operation(&mut operation_store.clone(), header, body, header_bytes, &topic.id(), false).await?;

    match ingest_result {
        IngestResult::Complete(operation) => {
            author_log_map
                .add_author(topic, operation.header.public_key)
                .await;

            if network_tx
                .send(ToNetwork::Message { bytes: gossip_message_bytes })
                .await
                .is_err()
            {
                println!("Failed to send gossip message");
            } else {
                println!("  Publish Operation - Sent gossip message: {:?}", prepare_for_logging(operation));
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
