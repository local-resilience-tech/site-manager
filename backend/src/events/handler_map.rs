use sqlx::Sqlite;

use crate::{
    panda_comms::site_events::{SiteEvent, SiteEventPayload},
    repos::{entities::Site, nodes::NodesRepo},
};

pub async fn handle_event(event: SiteEvent, pool: &sqlx::Pool<Sqlite>) {
    let header = event.header;

    match event.payload {
        SiteEventPayload::SiteAnnounced(site_announced) => {
            let repo = NodesRepo::init();

            println!("Site announced: {:?}", site_announced);

            let site: Site = Site {
                id: header.author_node_id.clone(),
                name: site_announced.name.clone(),
            };

            repo.upsert(pool, site).await.unwrap();
        }
    }
}
