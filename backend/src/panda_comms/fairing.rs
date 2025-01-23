use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Orbit, Rocket};
use rocket_db_pools::Database;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::this_node::ThisNodeRepo;

#[derive(Default)]
pub struct P2PandaCommsFairing {}

#[rocket::async_trait]
impl Fairing for P2PandaCommsFairing {
    fn info(&self) -> Info {
        Info {
            name: "P2PandaCommsFairing",
            kind: Kind::Liftoff | Kind::Singleton,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        if let Some(db) = MainDb::fetch(&rocket) {
            let repo = ThisNodeRepo::init();

            if let Some(container) = rocket.state::<P2PandaContainer>() {
                match repo.get_network_name(db).await {
                    Ok(network_name) => {
                        if let Some(network_name) = network_name {
                            println!("Got network name: {:?}", network_name);
                            container.set_network_name(network_name).await;
                        }
                    }
                    Err(_) => {
                        println!("Failed to get network name");
                    }
                }

                match repo.get_or_create_private_key(db).await {
                    Ok(private_key) => {
                        println!("Got private key");
                        container.set_private_key(private_key).await;
                    }
                    Err(_) => {
                        println!("Failed to get private key");
                    }
                }

                let bootstrap_details = repo.get_bootstrap_details(db).await.unwrap();
                let direct_address = match bootstrap_details {
                    Some(bootstrap) => Some(
                        container
                            .build_direct_address(bootstrap.node_id, bootstrap.ip4)
                            .unwrap(),
                    ),
                    None => None,
                };

                if let Err(e) = container.start(direct_address).await {
                    println!("Failed to start P2PandaContainer on liftoff: {:?}", e);
                }
            } else {
                println!("P2PandaContainer state not found.");
            }
        } else {
            println!("MainDb state not found, wont start Panda node");
        }
    }
}
