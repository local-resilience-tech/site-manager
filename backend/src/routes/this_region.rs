use rocket::post;
use rocket::serde::json::Json;
use rocket::Route;
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::repos::entities::Region;
use crate::repos::this_region::{CreateRegionData, ThisRegionError, ThisRegionRepo};

#[post("/create", data = "<data>")]
async fn create(mut db: Connection<MainDb>, data: Json<CreateRegionData>) -> Result<Json<Region>, ThisRegionError> {
    let repo = ThisRegionRepo::init();

    let result = repo
        .create_region(&mut db, data.into_inner())
        .await
        .map(|region| Json(region));

    // if result ok

    if let Ok(_region) = &result {
        // Set the region on the P2Panda node
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
