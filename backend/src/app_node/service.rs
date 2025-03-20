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
    messages::{ChannelEvent, NetworkEvent},
    node::Node,
    stream::StreamEvent,
    topic::{Topic, TopicMap},
};

pub struct Service {
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

impl Service {
    pub async fn new(
        network_name: String,
        private_key: PrivateKey,
        bootstrap_node_id: Option<PublicKey>,
        operation_store: MemoryStore<LogId, Extensions>,
    ) -> Result<Self> {
        let temp_blobs_root_dir = tempfile::tempdir().expect("temp dir");

        let relay_url: RelayUrl = RELAY_URL.parse().unwrap();

        let topic_map = TopicMap::new();

        let (node, stream_rx, network_events_rx) = Node::new(
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

    /// Spawn the service task.
    // #[cfg(not(test))]
    // pub fn run() {
    //     //use crate::toolkitty_node::rpc::Rpc;

    //     tokio::task::spawn(async move {
    //         let private_key = PrivateKey::new();
    //         let store = MemoryStore::new();
    //         let mut service = Self::new("test_network".to_string(), private_key, None, store)
    //             .await
    //             .expect("build stream");
    //         // let rpc = Rpc {
    //         //     context: service.context.clone(),
    //         // };
    //         let channel = service
    //             .recv_channel()
    //             .await
    //             .expect("receive on channel rx");
    //         service
    //             .inner_run(channel)
    //             .await
    //             .expect("run stream task");
    //     });
    // }

    /// Spawn the service task.
    #[cfg(test)]
    pub async fn run() -> Arc<RwLock<Context>> {
        let private_key = PrivateKey::new();
        let store = MemoryStore::new();
        let mut service = Self::new("test_network".to_string(), private_key, None, store)
            .await
            .expect("build stream");
        let context = service.context.clone();
        let rt = tokio::runtime::Handle::current();

        rt.spawn(async move {
            let channel = service
                .recv_channel()
                .await
                .expect("receive on channel rx");
            service
                .inner_run(channel)
                .await
                .expect("run stream task");
        });

        context
    }

    /// Run the inner service loop which awaits events arriving on the app, network, stream and
    /// invite codes channels.
    async fn inner_run(mut self, mut channel: broadcast::Sender<ChannelEvent>) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Ok(event) = self.to_app_rx.recv() => {
                    channel.send(event)?;
                }
                Ok(event) = self.network_events_rx.recv() => {
                    channel.send(ChannelEvent::NetworkEvent(NetworkEvent(event)))?;
                },
                Some(event) = self.stream_rx.recv() => {
                    channel.send(ChannelEvent::Stream(event))?;
                },
                Some(new_channel) = self.channel_rx.recv() => {
                    channel = new_channel;
                },
                // @TODO(sam): Need a way to handle ephemeral topics in the stream controller as
                // we now don't have a static topic we can subscribe to on startup.
                //
                // Some(event) = self.invite_codes_rx.recv() => {
                //     let json = match event {
                //         FromNetwork::GossipMessage { bytes, .. } => {
                //             serde_json::from_slice(&bytes)?
                //         },
                //         FromNetwork::SyncMessage { .. } => unreachable!(),
                //     };
                //     channel.send(ChannelEvent::InviteCodes(json))?;
                // }
            }
        }
    }

    async fn recv_channel(&mut self) -> anyhow::Result<broadcast::Sender<ChannelEvent>> {
        let Some(channel) = self.channel_rx.recv().await else {
            return Err(anyhow::anyhow!("channel tx closed"));
        };

        self.context.write().await.channel_set = true;

        Ok(channel)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rocket::tokio;
    use serde_json::json;
    use tokio::sync::broadcast;

    use crate::toolkitty_node::{
        extensions::{LogId, LogPath, Stream, StreamOwner, StreamRootHash},
        messages::{ChannelEvent, StreamArgs},
        rpc::Rpc,
        stream::{EventData, EventMeta, StreamEvent},
        topic::Topic,
    };

    use super::Service;

    #[tokio::test]
    async fn public_key() {
        let context = Service::run().await;
        let node_private_key = context.read().await.node.private_key.clone();
        let rpc = Rpc { context };

        let (channel_tx, _channel_rx) = broadcast::channel(10);
        let result = rpc.init(channel_tx).await;
        assert!(result.is_ok());

        let result = rpc.public_key().await;
        assert!(result.is_ok());
        let public_key = result.unwrap();
        assert_eq!(public_key, node_private_key.public_key());
    }

    #[tokio::test]
    async fn subscribe() {
        let context = Service::run().await;
        let rpc = Rpc { context };

        let (channel_tx, mut channel_rx) = broadcast::channel(10);
        let result = rpc.init(channel_tx).await;
        assert!(result.is_ok());

        let topic = Topic::Persisted("some_topic".into());
        let result = rpc.subscribe(&topic).await;
        assert!(result.is_ok());

        let event = channel_rx.recv().await.unwrap();
        match event {
            ChannelEvent::SubscribedToTopic(received_topic) => {
                assert_eq!(received_topic, topic);
            }
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn subscribe_ephemeral() {
        let context = Service::run().await;
        let rpc = Rpc { context };

        let (channel_tx, mut channel_rx) = broadcast::channel(10);
        let result = rpc.init(channel_tx).await;
        assert!(result.is_ok());

        let topic = Topic::Ephemeral("some_topic".to_string());
        let result = rpc.subscribe(&topic).await;
        assert!(result.is_ok());

        let event = channel_rx.recv().await.unwrap();
        match event {
            ChannelEvent::SubscribedToTopic(received_topic) => {
                assert_eq!(received_topic, topic);
            }
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn publish() {
        let context = Service::run().await;
        let private_key = context.read().await.node.private_key.clone();
        let topic = "some_topic";

        let rpc = Rpc { context };

        let (channel_tx, mut channel_rx) = broadcast::channel(10);
        let result = rpc.init(channel_tx).await;
        assert!(result.is_ok());

        let log_path = json!("calendar/inbox");

        let payload = json!({
            "message": "organize!"
        });

        let stream_args = StreamArgs {
            id: None,
            root_hash: None,
            owner: None,
        };

        let result = rpc
            .publish_persisted(
                &serde_json::to_vec(&payload).unwrap(),
                &stream_args,
                Some(&log_path.clone().into()),
                Some(&topic),
            )
            .await;

        assert!(result.is_ok());
        let (operation_hash, stream_id) = result.unwrap();

        let expected_log_path = log_path;
        let event = channel_rx.recv().await.unwrap();
        match event {
            ChannelEvent::Stream(stream_event) => {
                let EventMeta {
                    operation_id,
                    author,
                    stream,
                    log_path,
                } = stream_event.meta.unwrap();

                assert_eq!(author, private_key.public_key());
                assert_eq!(operation_id, operation_hash);
                assert_eq!(stream.id, stream_id);
                assert_eq!(stream.root_hash, StreamRootHash::from(operation_hash));
                assert_eq!(stream.owner, StreamOwner::from(private_key.public_key()));
                assert_eq!(log_path, Some(LogPath::from(expected_log_path)));

                let EventData::Application(value) = stream_event.data else {
                    panic!();
                };

                assert_eq!(value, payload);
            }
            _ => panic!(),
        }
    }

    #[tokio::test]
    async fn two_peers_subscribe() {
        let peer_a = Rpc {
            context: Service::run().await,
        };
        let peer_b = Rpc {
            context: Service::run().await,
        };

        let (peer_a_tx, _peer_a_rx) = broadcast::channel(100);
        let (peer_b_tx, mut peer_b_rx) = broadcast::channel(100);

        let result = peer_a.init(peer_a_tx).await;
        assert!(result.is_ok());

        let result = peer_b.init(peer_b_tx).await;
        assert!(result.is_ok());

        let topic = "some_topic";
        let result = peer_a.subscribe_ephemeral(&topic).await;
        assert!(result.is_ok());

        let result = peer_b.subscribe_ephemeral(&topic).await;
        assert!(result.is_ok());

        let send_payload = json!({
            "message": "organize!"
        });

        {
            let send_payload = send_payload.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let result = peer_a
                        .publish_ephemeral(&topic, &serde_json::to_vec(&send_payload).unwrap())
                        .await;
                    assert!(result.is_ok());
                }
            });
        }

        let mut message_received = false;
        while let Ok(event) = peer_b_rx.recv().await {
            if let ChannelEvent::Stream(StreamEvent {
                data: EventData::Ephemeral(payload),
                ..
            }) = event
            {
                assert_eq!(send_payload, payload);
                message_received = true;
                break;
            }
        }

        assert!(message_received);
    }

    #[tokio::test]
    async fn two_peers_sync() {
        let peer_a = Rpc {
            context: Service::run().await,
        };
        let peer_b = Rpc {
            context: Service::run().await,
        };

        let peer_a_public_key = peer_a.public_key().await.unwrap();
        let peer_b_public_key = peer_b.public_key().await.unwrap();

        let (peer_a_tx, mut peer_a_rx) = broadcast::channel(100);
        let (peer_b_tx, mut peer_b_rx) = broadcast::channel(100);

        let result = peer_a.init(peer_a_tx).await;
        assert!(result.is_ok());

        let result = peer_b.init(peer_b_tx).await;
        assert!(result.is_ok());

        let topic = "messages";
        let log_path = json!("messages").into();
        let stream_args = StreamArgs::default();

        let peer_a_payload = json!({
            "message": "organize!"
        });

        // Peer A publishes the first message to a new stream.
        let result = peer_a
            .publish_persisted(&serde_json::to_vec(&peer_a_payload).unwrap(), &stream_args, Some(&log_path), Some(&topic))
            .await;
        assert!(result.is_ok());

        // We need these values so Peer B can subscribe and publish to the correct stream.
        let (operation_id, stream_id) = result.unwrap();

        let stream_args = StreamArgs {
            id: Some(stream_id),
            root_hash: Some(operation_id.clone()),
            owner: Some(peer_a_public_key.clone()),
        };

        let peer_b_payload = json!({
            "message": "Hell yeah!"
        });

        // Peer B publishes it's own message to the stream.
        let result = peer_b
            .publish_persisted(&serde_json::to_vec(&peer_b_payload).unwrap(), &stream_args, Some(&log_path), Some(&topic))
            .await;
        assert!(result.is_ok());

        // Both peers add themselves and each other to their topic map.
        let stream = Stream {
            root_hash: operation_id.into(),
            owner: peer_a_public_key.into(),
        };
        let log_id = LogId {
            stream,
            log_path: Some(log_path),
        };

        peer_a
            .add_topic_log(&peer_a_public_key, &topic, &log_id)
            .await
            .unwrap();
        peer_a
            .add_topic_log(&peer_b_public_key, &topic, &log_id)
            .await
            .unwrap();

        peer_b
            .add_topic_log(&peer_a_public_key, &topic, &log_id)
            .await
            .unwrap();
        peer_b
            .add_topic_log(&peer_b_public_key, &topic, &log_id)
            .await
            .unwrap();

        // Finally they both subscribe to the topic.
        let result = peer_a.subscribe_persisted(&topic).await;
        assert!(result.is_ok());
        let result = peer_b.subscribe_persisted(&topic).await;
        assert!(result.is_ok());

        // Peer A should receive Peer B's message via sync.
        let mut message_received = false;
        while let Ok(event) = peer_a_rx.recv().await {
            if let ChannelEvent::Stream(StreamEvent {
                data: EventData::Application(payload),
                meta: Some(EventMeta { author, .. }),
            }) = event
            {
                if author == peer_b_public_key {
                    assert_eq!(peer_b_payload, payload);
                    message_received = true;
                    break;
                }
            }
        }

        assert!(message_received);

        // Peer B should receive Peer A's message via sync.
        let mut message_received = false;
        while let Ok(event) = peer_b_rx.recv().await {
            if let ChannelEvent::Stream(StreamEvent {
                data: EventData::Application(payload),
                meta: Some(EventMeta { author, .. }),
            }) = event
            {
                if author == peer_a_public_key {
                    assert_eq!(peer_a_payload, payload);
                    message_received = true;
                    break;
                }
            }
        }

        assert!(message_received);
    }
}
