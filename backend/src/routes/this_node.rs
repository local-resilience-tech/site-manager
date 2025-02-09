use iroh::NodeAddr;
use p2panda_net::NodeAddress;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};

use crate::panda_comms::container::P2PandaContainer;
use crate::repos::this_node::ThisNodeError;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NodeDetails {
    pub network_name: String,
    pub panda_node_id: String,
    pub iroh_node_addr: NodeAddr,
    pub peers: Vec<NodeAddress>,
}

#[get("/", format = "json")]
async fn show(panda_container: &State<P2PandaContainer>) -> Result<Json<Option<NodeDetails>>, ThisNodeError> {
    let is_started = panda_container.is_started().await;
    println!("Getting Panda Node");

    if !is_started {
        println!("Getting Panda Node: not started");
        return Ok(Json(None));
    }

    let public_key: String = panda_container
        .get_public_key()
        .await
        .unwrap()
        .to_string();

    let node_addr = panda_container.get_node_addr().await;

    let peers = panda_container
        .known_peers()
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Error finding peers".to_string()))?;

    let network_name = panda_container.get_network_name().await.unwrap();

    let node_details = NodeDetails {
        network_name: network_name.to_string(),
        panda_node_id: public_key,
        iroh_node_addr: node_addr,
        peers: peers,
    };

    Ok(Json(Some(node_details)))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BootstrapNodePeer {
    pub node_id: String,
    pub ip4: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BootstrapNodeData {
    pub network_name: String,
    pub bootstrap_peer: Option<BootstrapNodePeer>,
}

pub fn routes() -> Vec<Route> {
    routes![show]
}
