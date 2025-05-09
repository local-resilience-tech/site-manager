use p2panda_core::PublicKey;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::{build_public_key_from_hex, P2PandaContainer};
use crate::repos::entities::{Node, Region};
use crate::repos::this_p2panda_node::{SimplifiedNodeAddress, ThisP2PandaNodeRepo, ThisP2PandaNodeRepoError};

use super::this_p2panda_node::BootstrapNodeData;

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Option<Region>>, ThisP2PandaNodeRepoError> {
    let repo = ThisP2PandaNodeRepo::init();

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

#[get("/sites", format = "json")]
async fn sites() -> Result<Json<Vec<Node>>, ThisP2PandaNodeRepoError> {
    // create dummy data
    let sites = vec![
        Node {
            id: "1".to_string(),
            name: "Site 1".to_string(),
        },
        Node {
            id: "2".to_string(),
            name: "Site 2".to_string(),
        },
    ];

    Ok(Json(sites))
}

#[post("/bootstrap", format = "json", data = "<data>")]
async fn bootstrap(
    mut db: Connection<MainDb>,
    data: Json<BootstrapNodeData>,
    panda_container: &State<P2PandaContainer>,
) -> Result<Json<()>, ThisP2PandaNodeRepoError> {
    let repo = ThisP2PandaNodeRepo::init();

    let bootstrap_peer = &data.bootstrap_peer;

    let peer_address: Option<SimplifiedNodeAddress> = bootstrap_peer
        .as_ref()
        .map(|peer| SimplifiedNodeAddress {
            node_id: peer.node_id.clone(),
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
    routes![show, sites, bootstrap]
}
