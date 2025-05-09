use sqlx::Sqlite;

use crate::{
    panda_comms::lores_events::{LoResEvent, LoResEventPayload},
    repos::{entities::Node, nodes::NodesRepo},
};

pub async fn handle_event(event: LoResEvent, pool: &sqlx::Pool<Sqlite>) {
    let header = event.header;

    match event.payload {
        LoResEventPayload::NodeAnnounced(payload) => {
            let repo = NodesRepo::init();

            println!("Node announced: {:?}", payload);

            let node: Node = Node {
                id: header.author_node_id.clone(),
                name: payload.name.clone(),
            };

            repo.upsert(pool, node).await.unwrap();
        }
    }
}
