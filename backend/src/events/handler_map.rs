use sqlx::Sqlite;

use crate::panda_comms::site_events::{SiteEvent, SiteEventPayload};

pub async fn handle_event(event: SiteEvent, pool: &sqlx::Pool<Sqlite>) {
    match event.payload {
        SiteEventPayload::SiteAnnounced(site_announced) => {
            println!("Site announced: {:?}", site_announced);
        }
    }
}
