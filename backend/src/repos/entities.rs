use rocket::serde::{Deserialize, Serialize};
use sqlx;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Site {
    pub id: String,
    pub name: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SiteConfig {
    pub id: String,
    pub this_site_id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Node {
    pub node_id: String,
    pub site: Option<Site>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Region {
    pub network_id: String,
    pub nodes: Vec<Node>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PrivateKeyRow {
    pub private_key_hex: Option<String>,
}
