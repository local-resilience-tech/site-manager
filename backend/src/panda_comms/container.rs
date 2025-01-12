use anyhow::Result;
use gethostname::gethostname;
use iroh_net::NodeAddr;
use p2panda_core::{Hash, PrivateKey};
use p2panda_discovery::mdns::LocalDiscovery;
use p2panda_net::{FromNetwork, Network, NetworkBuilder, NetworkId, ToNetwork, TopicId};
use p2panda_sync::TopicQuery;
use rocket::tokio;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::panda_comms::manual_discovery::ManualDiscovery;
use crate::panda_comms::messages::Message;
use crate::panda_comms::site_messages::{SiteMessages, SiteRegistration};
use crate::panda_comms::sites::Sites;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ChatTopic(String, [u8; 32]);

impl ChatTopic {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned(), *Hash::new(name).as_bytes())
    }
}

impl TopicQuery for ChatTopic {}

impl TopicId for ChatTopic {
    fn id(&self) -> [u8; 32] {
        self.1
    }
}

#[derive(Default)]
pub struct P2PandaContainer {
    pub network_name: Arc<Mutex<Option<String>>>,
    pub network: Arc<Mutex<Option<Network<ChatTopic>>>>,
}

impl P2PandaContainer {
    pub async fn set_network_name(&self, network_name: String) {
        let mut network_name_lock = self.network_name.lock().await;
        *network_name_lock = Some(network_name);
    }

    pub async fn start(&self, private_key: PrivateKey) -> Result<()> {
        let mut sites = Sites::build();

        println!("P2Panda: starting up");

        let site_name = get_site_name();
        println!("Starting client for site: {}", site_name);

        let network_slug = "merri-bek.tech";
        let network_id: NetworkId = Hash::new(network_slug).into();

        let topic = ChatTopic::new("site_management");

        let network: Network<ChatTopic> = NetworkBuilder::new(network_id)
            .discovery(LocalDiscovery::new()?)
            .discovery(ManualDiscovery::new()?)
            .build()
            .await?;

        let (tx, mut rx, _ready) = network.subscribe(topic).await?;

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

        // put the network in the container
        let mut network_lock = self.network.lock().await;
        *network_lock = Some(network);

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
