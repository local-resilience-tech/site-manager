use rocket::tokio;
use tokio::sync::mpsc;

use crate::panda_comms::site_events::SiteEvent;

pub struct EventHandler {}

impl EventHandler {
    pub fn new(events_rx: mpsc::Receiver<SiteEvent>) -> Self {
        let result = EventHandler {};

        result.start_event_loop(events_rx);

        return result;
    }

    fn start_event_loop(&self, mut events_rx: mpsc::Receiver<SiteEvent>) {
        tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                // Handle the event
                println!("Received event in event handler: {:?}", event);
            }
        });
    }
}
