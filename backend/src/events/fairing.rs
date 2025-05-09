use rocket::fairing::{Fairing, Info, Kind};
use rocket::tokio;
use rocket::{Orbit, Rocket};
use rocket_db_pools::Database;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::infra::db::MainDb;
use crate::panda_comms::lores_events::LoResEvent;

use super::handler_map::handle_event;

pub struct EventHandlerFairing {
    events_rx: Arc<Mutex<mpsc::Receiver<LoResEvent>>>,
}

impl EventHandlerFairing {
    pub fn new(events_rx: mpsc::Receiver<LoResEvent>) -> Self {
        EventHandlerFairing {
            events_rx: Arc::new(Mutex::new(events_rx)),
        }
    }

    // fn start_event_loop(&self, rocket: &Rocket<Orbit>) {
    //     let events_rx_arc = Arc::clone(&self.events_rx);

    //     tokio::spawn(async move {
    //         let mut events_rx = events_rx_arc.lock().await;
    //         let db

    //         while let Some(event) = events_rx.recv().await {
    //             handle_event(event, db).await;
    //         }
    //     });
    // }
}

#[rocket::async_trait]
impl Fairing for EventHandlerFairing {
    fn info(&self) -> Info {
        Info {
            name: "EventHandlerFairing",
            kind: Kind::Liftoff | Kind::Singleton,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let events_rx_arc = Arc::clone(&self.events_rx);

        if let Some(db) = MainDb::fetch(&rocket) {
            let db_pool = db.sqlite_pool().clone();

            tokio::spawn(async move {
                let mut events_rx = events_rx_arc.lock().await;

                while let Some(event) = events_rx.recv().await {
                    handle_event(event, &db_pool).await;
                }
            });
        } else {
            println!("MainDb state not found, won't handle event");
        }
    }
}
