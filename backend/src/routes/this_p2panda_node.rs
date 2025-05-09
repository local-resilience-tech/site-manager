use iroh::NodeAddr;
use p2panda_net::NodeAddress;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};

use crate::panda_comms::container::P2PandaContainer;
use crate::repos::this_p2panda_node::ThisP2PandaNodeRepoError;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NodeDetails {
    pub panda_node_id: String,
    pub iroh_node_addr: NodeAddr,
    pub peers: Vec<NodeAddress>,
}

#[get("/", format = "json")]
async fn show(panda_container: &State<P2PandaContainer>) -> Result<Json<NodeDetails>, ThisP2PandaNodeRepoError> {
    let public_key: String = panda_container
        .get_public_key()
        .await
        .unwrap()
        .to_string();
    println!("public key: {}", public_key);

    let node_addr = panda_container.get_node_addr().await;
    println!("node addr: {:?}", node_addr);

    let mut peers = panda_container.known_peers().await;

    if peers.is_err() {
        println!("Failed to get known peers {:?}", peers);
        peers = Ok(vec![]);
    } else {
        println!("peers: {:?}", peers);
    }

    let node_details = NodeDetails {
        panda_node_id: public_key,
        iroh_node_addr: node_addr,
        peers: peers.unwrap(),
    };

    Ok(Json(node_details))
}

#[post("/restart", format = "json")]
async fn restart(panda_container: &State<P2PandaContainer>) -> Result<Json<String>, ThisP2PandaNodeRepoError> {
    panda_container.restart().await.map_err(|e| {
        println!("got error: {}", e);
        ThisP2PandaNodeRepoError::InternalServerError(e.to_string())
    })?;

    Ok(Json("Restarted".to_string()))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BootstrapNodePeer {
    pub node_id: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BootstrapNodeData {
    pub network_name: String,
    pub bootstrap_peer: Option<BootstrapNodePeer>,
}

pub fn routes() -> Vec<Route> {
    routes![show, restart]
}
