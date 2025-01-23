use rocket::serde::json::Json;
use rocket::Route;
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::repos::entities::Region;
use crate::repos::this_region::{ThisRegionError, ThisRegionRepo};

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Region>, ThisRegionError> {
    let repo = ThisRegionRepo::init();

    repo.get_region(&mut db)
        .await
        .map(|region| Json(region))
}

pub fn routes() -> Vec<Route> {
    routes![show]
}
