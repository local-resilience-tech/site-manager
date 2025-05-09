use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::Route;
use rocket::{post, State};
use rocket_db_pools::Connection;

use crate::infra::db::MainDb;
use crate::panda_comms::container::P2PandaContainer;
use crate::repos::entities::Node;
use crate::repos::this_node::{ThisNodeRepo, ThisNodeRepoError};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct CreateNodeDetails {
    name: String,
}

#[post("/create", data = "<data>")]
async fn create(data: Json<CreateNodeDetails>, panda_container: &State<P2PandaContainer>) -> Result<Json<Node>, ThisNodeRepoError> {
    panda_container
        .announce_node(data.name.clone())
        .await
        .map_err(|e| {
            println!("got error: {}", e);
            ThisNodeRepoError::InternalServerError(e.to_string())
        })?;

    return Ok(Json(Node {
        id: "1".to_string(),
        name: data.name.clone(),
    }));
}

#[get("/", format = "json")]
async fn show(mut db: Connection<MainDb>) -> Result<Json<Node>, ThisNodeRepoError> {
    let repo = ThisNodeRepo::init();

    repo.find(&mut db).await.map(|node| Json(node))
}

pub fn routes() -> Vec<Route> {
    routes![create, show]
}
