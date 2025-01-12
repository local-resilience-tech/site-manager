use rocket::serde::json::Json;
use rocket::Route;
use rocket::{post, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::entities::Region;
use crate::repos::this_region::{CreateRegionData, ThisRegionError, ThisRegionRepo};

#[post("/create", data = "<data>")]
async fn create(
    mut db: Connection<MainDb>,
    panda_container: &State<P2PandaContainer>,
    data: Json<CreateRegionData>,
) -> Result<Json<Region>, ThisRegionError> {
    let repo = ThisRegionRepo::init();

    let result = repo
        .create_region(&mut db, data.into_inner())
        .await
        .map(|region| Json(region));

    if let Ok(region) = &result {
        panda_container
            .set_network_name(region.name.clone())
            .await;

        // start the container
        if let Err(e) = panda_container.start().await {
            println!("Failed to start P2PandaContainer: {:?}", e);
        }
    }

    result
}

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Region>, ThisRegionError> {
    let repo = ThisRegionRepo::init();

    repo.get_region(&mut db)
        .await
        .map(|region| Json(region))
}

pub fn routes() -> Vec<Route> {
    routes![create, show]
}
