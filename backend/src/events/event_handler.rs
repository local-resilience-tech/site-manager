use rocket::tokio;
use tokio::sync::mpsc;

use crate::panda_comms::site_events::SiteEvent;

pub struct EventHandler {
    events_rx: mpsc::Receiver<SiteEvent>,
}

impl EventHandler {
    pub fn new(events_rx: mpsc::Receiver<SiteEvent>) -> Self {
        EventHandler { events_rx: events_rx }
    }
}
