use p2panda_core::PublicKey;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::{build_public_key_from_hex, P2PandaContainer};
use crate::repos::entities::Region;
use crate::repos::this_node::{SimplifiedNodeAddress, ThisNodeError, ThisNodeRepo};

use super::this_node::BootstrapNodeData;

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Option<Region>>, ThisNodeError> {
    let repo = ThisNodeRepo::init();

    repo.get_network_name_conn(&mut db)
        .await
        .map(|network_id| match network_id {
            Some(network_id) => {
                println!("got network id {}", network_id);
                Json(Some(Region { network_id }))
            }
            None => {
                println!("no network id");
                Json(None)
            }
        })
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

    let bootstrap_node_id: Option<PublicKey> = match peer_address.clone() {
        Some(bootstrap) => build_public_key_from_hex(bootstrap.node_id.clone()),
        None => None,
    };
    panda_container
        .set_bootstrap_node_id(bootstrap_node_id)
        .await;

    // start the container
    if let Err(e) = panda_container.start().await {
        println!("Failed to start P2PandaContainer: {:?}", e);
    }

    Ok(Json(()))
}

pub fn routes() -> Vec<Route> {
    routes![show, bootstrap]
}
