use sqlx::Sqlite;

use crate::{
    panda_comms::lores_events::{LoResEvent, LoResEventPayload},
    repos::{entities::Node, nodes::NodesRepo},
};

pub async fn handle_event(event: LoResEvent, pool: &sqlx::Pool<Sqlite>) {
    let header = event.header;

    match event.payload {
        LoResEventPayload::NodeAnnounced(site_announced) => {
            let repo = NodesRepo::init();

            println!("Site announced: {:?}", site_announced);

            let site: Node = Node {
                id: header.author_node_id.clone(),
                name: site_announced.name.clone(),
            };

            repo.upsert(pool, site).await.unwrap();
        }
    }
}
