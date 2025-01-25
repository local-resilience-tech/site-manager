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

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Region {
    pub network_id: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PrivateKeyRow {
    pub private_key_hex: Option<String>,
}
