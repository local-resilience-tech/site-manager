use iroh_net::NodeAddr;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};
use thiserror::Error;

use crate::panda_comms::container::P2PandaContainer;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NodeDetails {
    pub panda_node_id: String,
    pub iroh_node_addr: NodeAddr,
}

#[derive(Debug, Error, Responder)]
pub enum ThisNodeError {}

#[get("/", format = "json")]
async fn show(panda_container: &State<P2PandaContainer>) -> Result<Json<NodeDetails>, ThisNodeError> {
    let public_key: String = panda_container
        .get_public_key()
        .await
        .unwrap()
        .to_string();

    let node_addr = panda_container.get_node_addr().await;

    let dummy_details = NodeDetails {
        panda_node_id: public_key,
        iroh_node_addr: node_addr,
    };

    let peers = panda_container.known_peers().await;
    match peers {
        Ok(peers) => {
            println!("Got peers: {:?}", peers);
        }
        Err(e) => {
            println!("Error fetching peers: {}", e)
        }
    }

    Ok(Json(dummy_details))
}

pub fn routes() -> Vec<Route> {
    routes![show]
}
