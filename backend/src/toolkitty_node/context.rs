use rocket::tokio::sync::{broadcast, mpsc};
use std::collections::HashMap;

use super::{
    messages::ChannelEvent,
    node::Node,
    topic::{Topic, TopicMap},
};

/// Shared application context which can be accessed from within the main application runtime loop
/// as well as any tauri command.
pub struct Context {
    /// Local p2panda node.
    pub node: Node<Topic>,

    /// All topics we have subscribed to.
    pub subscriptions: HashMap<[u8; 32], Topic>,

    /// Channel for sending messages to the frontend application. Messages are forwarded on the
    /// main event channel and will be received by the channel processor on the frontend.
    pub to_app_tx: broadcast::Sender<ChannelEvent>,

    /// Sync protocol topic map.
    pub topic_map: TopicMap,

    /// Channel for sending the actual tauri channel where all backend->frontend events are sent.
    /// We need this so that when the `init` command is called with the channel as an argument it
    /// can be forwarded into the main application service task which is waiting for it.
    pub channel_tx: mpsc::Sender<broadcast::Sender<ChannelEvent>>,

    /// Flag indicating that we already received a channel from the frontend (init was already
    /// called). There is a bug if `init` is called twice.
    pub channel_set: bool,
}

impl Context {
    pub fn new(
        node: Node<Topic>,
        to_app_tx: broadcast::Sender<ChannelEvent>,
        topic_map: TopicMap,
        channel_tx: mpsc::Sender<broadcast::Sender<ChannelEvent>>,
    ) -> Self {
        Self {
            node,
            subscriptions: HashMap::new(),
            to_app_tx,
            topic_map: topic_map.clone(),
            channel_tx,
            channel_set: false,
        }
    }
}
