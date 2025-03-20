use anyhow::Result;
use gethostname::gethostname;
use iroh::NodeAddr;
use p2panda_core::identity::PUBLIC_KEY_LEN;
use p2panda_core::{Body, PrivateKey, PublicKey};
use p2panda_net::{FromNetwork, NodeAddress, ToNetwork};
use p2panda_store::MemoryStore;
use p2panda_stream::{DecodeExt, IngestExt};
use rocket::tokio::sync::mpsc;
use rocket::tokio::{self, task};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::app_node::node::Node;
use crate::app_node::operations::prepare_for_logging;
use crate::toolkitty_node::extensions::{Extensions, LogId, LogPath};
use crate::toolkitty_node::operation::{create_operation, decode_gossip_message};
use crate::toolkitty_node::topic::{Topic, TopicMap};

use super::messages::Message;
use super::site_messages::{SiteMessages, SiteRegistration};

#[derive(Default)]
pub struct P2PandaContainer {
    params: Arc<Mutex<NodeParams>>,
    node: Arc<Mutex<Option<Node>>>,
}

#[derive(Default, Clone)]
pub struct NodeParams {
    pub private_key: Option<PrivateKey>,
    pub network_name: Option<String>,
    pub bootstrap_node_id: Option<PublicKey>,
}

impl P2PandaContainer {
    pub async fn get_params(&self) -> NodeParams {
        let params_lock = self.params.lock().await;
        params_lock.clone()
    }

    pub async fn set_network_name(&self, network_name: String) {
        let mut params_lock = self.params.lock().await;
        params_lock.network_name = Some(network_name);
    }

    pub async fn set_private_key(&self, private_key: PrivateKey) {
        let mut params_lock = self.params.lock().await;
        params_lock.private_key = Some(private_key);
    }

    pub async fn set_bootstrap_node_id(&self, bootstrap_node_id: Option<PublicKey>) {
        let mut params_lock = self.params.lock().await;
        params_lock.bootstrap_node_id = bootstrap_node_id;
    }

    pub async fn restart(&self) -> Result<()> {
        println!("Restarting node: shutting down");
        self.shutdown().await?;
        println!("Restarting node: starting");
        self.start().await?;
        println!("Restarting node: done");

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let node = self.node.lock().await;
        let node = node
            .as_ref()
            .ok_or(anyhow::Error::msg("Network not started"))?;

        node.shutdown().await?;
        self.set_node(None).await;

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let site_name = get_site_name();
        println!("Starting client for site: {}", site_name);

        let params = self.get_params().await;

        let private_key: Option<PrivateKey> = params.private_key;
        let network_name: Option<String> = params.network_name;
        let boostrap_node_id: Option<PublicKey> = params.bootstrap_node_id;

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

        self.start_for(site_name, private_key, network_name, boostrap_node_id)
            .await
    }

    async fn start_for(&self, site_name: String, private_key: PrivateKey, network_name: String, boostrap_node_id: Option<PublicKey>) -> Result<()> {
        let operation_store = MemoryStore::<LogId, Extensions>::new();
        let node = Node::new(network_name, private_key.clone(), boostrap_node_id, operation_store.clone()).await?;

        let topic = Topic::Persisted("site_management".to_string());

        self.setup_subscriptions(topic, &node, operation_store, site_name, private_key)
            .await?;

        // put the node in the container
        self.set_node(Some(node)).await;

        Ok(())
    }

    async fn setup_subscriptions(
        &self,
        topic: Topic,
        node: &Node,
        operation_store: MemoryStore<LogId, Extensions>,
        site_name: String,
        private_key: PrivateKey,
    ) -> Result<(), anyhow::Error> {
        let (_network_tx, network_rx, gossip_ready) = node.subscribe(topic.clone()).await?;

        {
            let mut operation_store = operation_store.clone();

            task::spawn(async move {
                announce_site_regularly(site_name, &mut operation_store, &private_key).await;
                if gossip_ready.await.is_ok() {
                    println!("- JOINED GOSSIP NETWORK -");

                    // announce_site_regularly(site_name, &mut operation_store, &mut topic_map, topic, &private_key, &network_tx);
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
            // let mut topic_map = topic_map.clone();
            // let topic = topic.clone();

            task::spawn(async move {
                while let Some(operation) = stream.next().await {
                    // topic_map
                    //     .add_author(topic.clone(), operation.header.public_key)
                    //     .await;

                    println!("+ Received operation: {:?}", prepare_for_logging(operation));
                }
            });
        }

        Ok(())
    }

    pub async fn get_public_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let node = self.node.lock().await;
        let node = node.as_ref().ok_or("Network not started")?;

        let node_id = node.node_id().await;
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

    async fn set_node(&self, maybe_node: Option<Node>) {
        let mut node_lock = self.node.lock().await;
        *node_lock = maybe_node;
    }
}

async fn announce_site_regularly(site_name: String, operation_store: &mut MemoryStore<LogId, Extensions>, private_key: &PrivateKey) {
    let mut operation_store = operation_store.clone();
    let private_key = private_key.clone();

    // spawn a task to announce the site every 30 seconds
    task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            let body = build_announce_site_body(&site_name);
            publish_operation(Some(body), &mut operation_store, &private_key)
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

async fn publish_operation(body: Option<Body>, operation_store: &mut MemoryStore<LogId, Extensions>, private_key: &PrivateKey) -> Result<()> {
    let log_path: LogPath = json!("site_management").into();

    let extensions = Extensions {
        log_path: Some(log_path),
        ..Default::default()
    };
    let body_bytes: Option<Vec<u8>> = body.map(|body| body.to_bytes());
    let body_bytes_array = body_bytes
        .as_ref()
        .map(|body| body.as_slice())
        .unwrap_or(&[]);

    let (header, body) = create_operation(&mut operation_store.clone(), &private_key, extensions, Some(body_bytes_array)).await;

    // let ingest_result = ingest_operation(&mut operation_store.clone(), header, body, header_bytes, &topic.id(), false).await?;

    // match ingest_result {
    //     IngestResult::Complete(operation) => {
    //         // topic_map
    //         //     .add_author(topic, operation.header.public_key)
    //         //     .await;

    //         if network_tx
    //             .send(ToNetwork::Message { bytes: gossip_message_bytes })
    //             .await
    //             .is_err()
    //         {
    //             println!("Failed to send gossip message");
    //         } else {
    //             println!("  Publish Operation - Sent gossip message: {:?}", prepare_for_logging(operation));
    //         }
    //     }
    //     _ => unreachable!(),
    // }

    Ok(())
}

// TODO: This should be in p2panda-core, submit a PR
pub fn build_public_key_from_hex(key_hex: String) -> Option<PublicKey> {
    let key_bytes = hex::decode(key_hex).ok()?;
    let key_byte_array: [u8; PUBLIC_KEY_LEN] = key_bytes.try_into().ok()?;
    let result = PublicKey::from_bytes(&key_byte_array);

    match result {
        Ok(key) => Some(key),
        Err(_) => None,
    }
}
