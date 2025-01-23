use super::entities::Region;
use crate::{infra::db::MainDb, repos::helpers::SITE_CONFIG_ID};
use rocket_db_pools::Connection;
use thiserror::Error;

pub struct ThisRegionRepo {}

#[derive(Debug, Error, Responder)]
pub enum ThisRegionError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),

    #[error("Site not found")]
    #[response(status = 404)]
    NotFound(String),
}

impl ThisRegionRepo {
    pub fn init() -> Self {
        ThisRegionRepo {}
    }

    pub async fn get_region(&self, db: &mut Connection<MainDb>) -> Result<Region, ThisRegionError> {
        let region = sqlx::query_as!(
            Region,
            "
            SELECT regions.name as network_id, regions.id, regions.name
            FROM regions
            INNER JOIN site_configs ON site_configs.this_region_id = regions.id
            WHERE site_configs.id = ?
            LIMIT 1
            ",
            SITE_CONFIG_ID
        )
        .fetch_one(&mut ***db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ThisRegionError::NotFound("Site not found".to_string()),
            _ => ThisRegionError::InternalServerError("Database error".to_string()),
        })?;

        return Ok(region);
    }
}
