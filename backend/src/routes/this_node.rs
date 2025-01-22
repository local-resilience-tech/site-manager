use iroh_net::NodeAddr;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Route, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::this_node::{ThisNodeError, ThisNodeRepo};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NodeDetails {
    pub panda_node_id: String,
    pub iroh_node_addr: NodeAddr,
    pub peers: Vec<NodeAddr>,
}

#[get("/", format = "json")]
async fn show(panda_container: &State<P2PandaContainer>) -> Result<Json<NodeDetails>, ThisNodeError> {
    let public_key: String = panda_container
        .get_public_key()
        .await
        .unwrap()
        .to_string();

    let node_addr = panda_container.get_node_addr().await;

    let peers = panda_container.known_peers().await;

    let node_details = NodeDetails {
        panda_node_id: public_key,
        iroh_node_addr: node_addr,
        peers: peers.unwrap(),
    };

    Ok(Json(node_details))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BootstrapNodeData {
    network_name: String,
    node_id: String,
    ip_address: String,
}

#[post("/bootstrap", format = "json", data = "<data>")]
async fn bootstrap(
    mut db: Connection<MainDb>,
    data: Json<BootstrapNodeData>,
    panda_container: &State<P2PandaContainer>,
) -> Result<(), ThisNodeError> {
    println!(
        "Bootstrapping to node: {:?}, {:?} , {:?}, ",
        data.network_name, data.node_id, data.ip_address
    );

    let repo = ThisNodeRepo::init();

    repo.set_network_config(&mut db, data.network_name.clone(), data.node_id.clone(), data.ip_address.clone())
        .await?;

    panda_container
        .set_network_name(data.network_name.clone())
        .await;

    let direct_address = panda_container
        .build_direct_address(data.node_id.clone(), data.ip_address.clone())
        .map_err(|e| ThisNodeError::InternalServerError(e.to_string()))?;

    // start the container
    if let Err(e) = panda_container.start(Some(direct_address)).await {
        println!("Failed to start P2PandaContainer: {:?}", e);
    }

    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![show, bootstrap]
}
