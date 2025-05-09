use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::Route;
use rocket::{post, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::entities::Site;
use crate::repos::this_node::{ThisNodeRepo, ThisNodeRepoError};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreateSiteDetails {
    name: String,
}

#[post("/create", data = "<data>")]
async fn create(data: Json<CreateSiteDetails>, panda_container: &State<P2PandaContainer>) -> Result<Json<Site>, ThisNodeRepoError> {
    panda_container
        .announce_site(data.name.clone())
        .await
        .map_err(|e| {
            println!("got error: {}", e);
            ThisNodeRepoError::InternalServerError(e.to_string())
        })?;

    return Ok(Json(Site {
        id: "1".to_string(),
        name: data.name.clone(),
    }));
}

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Site>, ThisNodeRepoError> {
    let repo = ThisNodeRepo::init();

    repo.get_site(&mut db)
        .await
        .map(|site| Json(site))
}

pub fn routes() -> Vec<Route> {
    routes![create, show]
}
