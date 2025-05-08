use rocket::fairing::{Fairing, Info, Kind};
use rocket::tokio;
use rocket::{Orbit, Rocket};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::panda_comms::site_events::SiteEvent;

pub struct EventHandlerFairing {
    events_rx: Arc<Mutex<mpsc::Receiver<SiteEvent>>>,
}

impl EventHandlerFairing {
    pub fn new(events_rx: mpsc::Receiver<SiteEvent>) -> Self {
        EventHandlerFairing {
            events_rx: Arc::new(Mutex::new(events_rx)),
        }
    }

    fn start_event_loop(&self) {
        let events_rx_arc = Arc::clone(&self.events_rx);

        tokio::spawn(async move {
            let mut events_rx = events_rx_arc.lock().await;

            while let Some(event) = events_rx.recv().await {
                // Handle the event
                println!("Received event in event handler: {:?}", event);
            }
        });
    }
}

#[rocket::async_trait]
impl Fairing for EventHandlerFairing {
    fn info(&self) -> Info {
        Info {
            name: "EventHandlerFairing",
            kind: Kind::Liftoff | Kind::Singleton,
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        // Start the event loop when the application starts
        self.start_event_loop();

        // Log the event handler initialization
        println!("EventHandlerFairing initialized");
    }
}
