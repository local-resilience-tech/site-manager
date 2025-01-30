use iroh_net::NodeAddr;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::entities::{Node, Region};
use crate::repos::this_node::{SimplifiedNodeAddress, ThisNodeError, ThisNodeRepo};

use super::this_node::BootstrapNodeData;

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>, panda_container: &State<P2PandaContainer>) -> Result<Json<Option<Region>>, ThisNodeError> {
    let is_started = panda_container.is_started().await;

    if !is_started {
        return Ok(Json(None));
    }

    let repo = ThisNodeRepo::init();
    let network_name_result = repo.get_network_name_conn(&mut db).await?;

    if None == network_name_result {
        return Ok(Json(None));
    }

    let network_name = network_name_result.unwrap();

    let peers: Vec<NodeAddr> = panda_container
        .known_peers()
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Error finding peers".to_string()))?;

    let nodes: Vec<Node> = peers
        .iter()
        .map(|peer| Node {
            node_id: peer.node_id.to_string(),
            site: None,
        })
        .collect();

    return Ok(Json(Some(Region {
        network_id: network_name,
        nodes,
    })));
}

#[post("/bootstrap", format = "json", data = "<data>")]
async fn bootstrap(
    mut db: Connection<MainDb>,
    data: Json<BootstrapNodeData>,
    panda_container: &State<P2PandaContainer>,
) -> Result<Json<()>, ThisNodeError> {
    let repo = ThisNodeRepo::init();

    let bootstrap_peer = &data.bootstrap_peer;

    let peer_address: Option<SimplifiedNodeAddress> = bootstrap_peer
        .as_ref()
        .map(|peer| SimplifiedNodeAddress {
            node_id: peer.node_id.clone(),
            ip4: peer.ip4.clone(),
        });

    repo.set_network_config(&mut db, data.network_name.clone(), peer_address.clone())
        .await?;

    panda_container
        .set_network_name(data.network_name.clone())
        .await;

    let direct_address = match peer_address.clone() {
        Some(bootstrap) => Some(
            panda_container
                .build_direct_address(bootstrap.node_id, bootstrap.ip4)
                .unwrap(),
        ),
        None => None,
    };

    // start the container
    if let Err(e) = panda_container.start(direct_address).await {
        println!("Failed to start P2PandaContainer: {:?}", e);
    }

    Ok(Json(()))
}

pub fn routes() -> Vec<Route> {
    routes![show, bootstrap]
}
