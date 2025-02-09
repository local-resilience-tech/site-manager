use anyhow::Result;
use gethostname::gethostname;
use iroh::NodeAddr;
use p2panda_core::identity::PUBLIC_KEY_LEN;
use p2panda_core::{Hash, PrivateKey, PublicKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{FromNetwork, Network, NetworkBuilder, NetworkId, NodeAddress, RelayUrl, SyncConfiguration, ToNetwork};
use p2panda_store::MemoryStore;
use p2panda_sync::log_sync::LogSyncProtocol;
use rocket::tokio;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::messages::Message;
use super::operations::Extensions;
use super::site_messages::{SiteMessages, SiteRegistration};
use super::sites::Sites;
use super::topics::{AuthorStore, ChatTopic, LogId};

pub struct DirectAddress {
    pub node_id: PublicKey,
    pub addresses: Vec<SocketAddr>,
}

// This Iroh relay node is hosted by Liebe Chaos for P2Panda development. It is not intended for
// production use, and LoRes tech will eventually provide a relay node for production use.
const RELAY_URL: &str = "https://staging-euw1-1.relay.iroh.network/";

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
        Ok(DirectAddress { node_id, addresses })
    }

    pub async fn is_started(&self) -> bool {
        let network = self.network.lock().await;
        network.is_some()
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

        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();

        // Setup Network Builder
        let mut builder = NetworkBuilder::new(network_id).private_key(private_key.clone());

        // Setup connection config and bootstrapping
        builder = builder
            .relay(relay_url.clone(), false, 0)
            .discovery(LocalDiscovery::new());

        if let Some(direct_address) = direct_address {
            let DirectAddress { node_id, addresses: _ } = direct_address;
            builder = builder.direct_address(node_id, vec![], None);
        }

        // Setup operations
        let operation_store = MemoryStore::<LogId, Extensions>::new();
        let author_store = AuthorStore::new();
        let sync_protocol = LogSyncProtocol::new(author_store.clone(), operation_store.clone());
        let sync_config = SyncConfiguration::new(sync_protocol);
        builder = builder.sync(sync_config);

        // Create network
        let network: Network<ChatTopic> = builder.build().await?;

        // put the network in the container
        let mut network_lock = self.network.lock().await;
        *network_lock = Some(network);

        // Setup subscriptions
        self.setup_subscriptions(site_name, private_key)
            .await?;

        Ok(())
    }

    async fn setup_subscriptions(&self, site_name: String, private_key: PrivateKey) -> Result<(), anyhow::Error> {
        let topic = ChatTopic::new("site_management");

        let network = self.network.lock().await;
        let network = network
            .as_ref()
            .ok_or(anyhow::Error::msg("Network not started"))?;

        let (tx, mut rx, _ready) = network.subscribe(topic).await?;

        let mut sites = Sites::build();

        tokio::task::spawn(async move {
            while let Some(event) = rx.recv().await {
                handle_gossip_event(event, &mut sites);
            }
        });

        // spawn a task to announce the site every 30 seconds
        tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                announce_site(&private_key, &site_name, &tx)
                    .await
                    .ok();
            }
        });

        Ok(())
    }
}

fn get_site_name() -> String {
    gethostname().to_string_lossy().to_string()
}

fn handle_gossip_event(event: FromNetwork, sites: &mut Sites) {
    match event {
        FromNetwork::GossipMessage { bytes, .. } => match Message::decode_and_verify(&bytes) {
            Ok(message) => {
                handle_message(message, sites);
            }
            Err(err) => {
                eprintln!("Invalid gossip message: {}", err);
            }
        },
        _ => panic!("no sync messages expected"),
    }
}

fn handle_message(message: Message<SiteMessages>, sites: &mut Sites) {
    match message.payload {
        SiteMessages::SiteRegistration(registration) => {
            println!("Received SiteRegistration: {:?}", registration);
            sites.register(registration.name);
            sites.log();
        }
        SiteMessages::SiteNotification(notification) => {
            println!("Received SiteNotification: {:?}", notification);
        }
    }
}

async fn announce_site(private_key: &PrivateKey, name: &str, tx: &tokio::sync::mpsc::Sender<ToNetwork>) -> Result<()> {
    println!("Announcing myself: {}", name);
    tx.send(ToNetwork::Message {
        bytes: Message::sign_and_encode(private_key, SiteMessages::SiteRegistration(SiteRegistration { name: name.to_string() }))?,
    })
    .await?;
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
