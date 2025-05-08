use anyhow::Result;
use iroh::NodeAddr;
use p2panda_core::identity::PUBLIC_KEY_LEN;
use p2panda_core::{PrivateKey, PublicKey};
use p2panda_net::{NodeAddress, RelayUrl, SystemEvent};
use p2panda_node::api::NodeApi;
use p2panda_node::extensions::{LogId, NodeExtensions};
use p2panda_node::node::Node;
use p2panda_node::stream::{EventData, StreamEvent};
use p2panda_node::topic::{Topic, TopicMap};
use p2panda_store::MemoryStore;
use rocket::tokio::{self};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};

use super::site_events::{SiteAnnounced, SiteEvent, SiteEventPayload};

const RELAY_URL: &str = "https://staging-euw1-1.relay.iroh.network/";
const TOPIC_NAME: &str = "site_management";
const LOG_ID: &str = "site_management";

pub struct P2PandaContainer {
    params: Arc<Mutex<NodeParams>>,
    node_api: Arc<Mutex<Option<NodeApi<NodeExtensions>>>>,
    events_tx: mpsc::Sender<SiteEvent>,
}

#[derive(Default, Clone)]
pub struct NodeParams {
    pub private_key: Option<PrivateKey>,
    pub network_name: Option<String>,
    pub bootstrap_node_id: Option<PublicKey>,
}

impl P2PandaContainer {
    pub fn new(events_tx: mpsc::Sender<SiteEvent>) -> Self {
        let params = Arc::new(Mutex::new(NodeParams::default()));
        let node_api = Arc::new(Mutex::new(None));

        P2PandaContainer { params, node_api, events_tx }
    }

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
        // let node = self.node.lock().await;
        // let node = node
        //     .as_ref()
        //     .ok_or(anyhow::Error::msg("Network not started"))?;

        // node.shutdown().await?;
        // self.set_node(None).await;

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        println!("Starting client");

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

        self.start_for(private_key, network_name, boostrap_node_id)
            .await
    }

    async fn start_for(&self, private_key: PrivateKey, network_name: String, boostrap_node_id: Option<PublicKey>) -> Result<()> {
        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();
        let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");

        let store = MemoryStore::<LogId, NodeExtensions>::new();
        let topic_map = TopicMap::new();

        println!(
            "Starting node. Network name: {}, Bootstrap ID: {:?}",
            network_name,
            boostrap_node_id.map(|key| key.to_string())
        );

        let (node, stream_rx, network_events_rx) = Node::new(
            network_name,
            private_key.clone(),
            boostrap_node_id,
            Some(relay_url),
            store,
            temp_blobs_root_dir.into_path(),
            topic_map.clone(),
        )
        .await?;

        let mut node_api = NodeApi::new(node, topic_map);

        let public_key = private_key.public_key();

        node_api
            .add_topic_log(&public_key, TOPIC_NAME, LOG_ID)
            .await?;

        // subscribe to site management topic
        node_api.subscribe_persisted(TOPIC_NAME).await?;

        // put the node in the container
        self.set_node_api(Some(node_api)).await;

        self.listen_for_messages(stream_rx, network_events_rx);

        Ok(())
    }

    pub async fn get_public_key(&self) -> Result<String, Box<dyn std::error::Error>> {
        let node_api = self.node_api.lock().await;
        let node_api = node_api.as_ref().ok_or("Network not started")?;

        let node_id = node_api.node.network.node_id();
        Ok(node_id.to_string())
    }

    pub async fn get_node_addr(&self) -> NodeAddr {
        let node_api = self.node_api.lock().await;
        let node_api = node_api.as_ref().unwrap();
        let network = &node_api.node.network;
        let endpoint = network.endpoint();
        endpoint.node_addr().await.unwrap()
    }

    pub async fn known_peers(&self) -> Result<Vec<NodeAddress>> {
        let node_api = self.node_api.lock().await;
        let node_api = node_api.as_ref().unwrap();
        node_api.node.network.known_peers().await
    }

    async fn set_node_api(&self, maybe_node_api: Option<NodeApi<NodeExtensions>>) {
        let mut node_api_lock = self.node_api.lock().await;
        *node_api_lock = maybe_node_api;
    }

    pub async fn announce_site(&self, site_name: String) -> Result<()> {
        let mut node_api = self.node_api.lock().await;
        let node_api = node_api
            .as_mut()
            .ok_or(anyhow::Error::msg("Network not started"))?;

        let site_announced = SiteAnnounced { name: site_name.clone() };
        let event_payload = SiteEventPayload::SiteAnnounced(site_announced);

        let payload = serde_json::to_vec(&event_payload)?;

        let extensions = NodeExtensions {
            log_id: Some(LogId(LOG_ID.to_string())),
            ..Default::default()
        };

        node_api
            .publish_persisted(TOPIC_NAME, &payload, Some(LOG_ID), Some(extensions))
            .await?;

        println!("Announcing site: {}", site_name);

        Ok(())
    }

    fn listen_for_messages(
        &self,
        mut stream_rx: mpsc::Receiver<StreamEvent<NodeExtensions>>,
        mut network_events_rx: broadcast::Receiver<SystemEvent<Topic>>,
    ) {
        let node_api = self.node_api.clone();

        // handle received network events. This exists mainly for debugging
        // at the moment, but the addition of a peer to the topic map on the
        // PeerDiscovered event is important.
        tokio::spawn(async move {
            println!("Listening for network events...");
            while let Ok(event) = network_events_rx.recv().await {
                let event: SystemEvent<Topic> = event;
                match event {
                    SystemEvent::GossipJoined { topic_id, peers } => {
                        println!("Gossip joined: {:?}", topic_id);
                        println!(
                            "Peers: {:?}",
                            peers
                                .iter()
                                .map(|peer| peer.to_hex())
                                .collect::<Vec<_>>()
                        );
                    }
                    SystemEvent::GossipLeft { topic_id } => {
                        println!("Gossip left: {:?}", topic_id);
                    }
                    SystemEvent::GossipNeighborUp { topic_id: _, peer } => {
                        println!("Gossip neighbor up: {:?}", peer.to_hex());
                    }
                    SystemEvent::GossipNeighborDown { topic_id: _, peer } => {
                        println!("Gossip neighbor down: {:?}", peer.to_hex());
                    }
                    SystemEvent::PeerDiscovered { peer } => {
                        println!("Peer discovered: {:?}", peer.to_hex());
                        let mut node_api = node_api.lock().await;
                        let node_api = node_api.as_mut().unwrap();

                        node_api
                            .add_topic_log(&peer, TOPIC_NAME, LOG_ID)
                            .await
                            .unwrap();
                    }
                    SystemEvent::SyncStarted { topic, peer } => {
                        println!("Sync started: topic {:?}, peer {:?}", topic, peer.to_hex());
                    }
                    SystemEvent::SyncDone { topic, peer } => {
                        println!("Sync done: topic {:?}, peer {:?}", topic, peer.to_hex());
                    }
                    SystemEvent::SyncFailed { topic, peer } => {
                        println!("Sync failed: topic {:?}, peer {:?}", topic, peer.to_hex());
                    }
                }
            }
            println!("Network events stream closed");
        });

        let events_tx = self.events_tx.clone();

        // handle received messages
        tokio::spawn(async move {
            println!("Listening for messages...");
            while let Some(event) = stream_rx.recv().await {
                println!("Received message: {:?}", event);
                let data: EventData = event.data;

                match data {
                    EventData::Application(payload) => {
                        let site_event: Result<SiteEventPayload, _> = serde_json::from_slice(&payload);
                        match site_event {
                            Ok(event) => {
                                println!("  Parsed SiteEvent: {:?}", event);

                                // emit to the event handler

                                let event = SiteEvent::new(event);
                                let send_result = events_tx.send(event).await;

                                if let Err(err) = send_result {
                                    println!("  Failed to send event: {:?}", err);
                                }
                            }
                            Err(err) => println!("  Failed to parse Site Event: {:?}", err),
                        }
                    }
                    EventData::Ephemeral(payload) => {
                        let payload: serde_json::Value = serde_json::from_slice(&payload).unwrap();
                        println!("  Ephemeral Payload: {:?}", payload);
                    }
                    EventData::Error(error) => {
                        println!("  Stream Error: {:?}", error);
                    }
                }
            }
            println!("Message stream closed");
        });
    }
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
