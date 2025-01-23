use super::entities::Region;
use crate::{
    infra::db::MainDb,
    repos::helpers::{NETWORK_CONFIG_ID, SITE_CONFIG_ID},
};
use rocket::serde::Deserialize;
use rocket_db_pools::Connection;
use thiserror::Error;
use uuid::Uuid;

pub struct ThisRegionRepo {}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateRegionData {
    name: String,
}

#[derive(Debug, Error, Responder)]
pub enum ThisRegionError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),

    #[error("Cannot create site")]
    #[response(status = 409)]
    CannotCreate(String),

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

    pub async fn create_this_region(&self, db: &mut Connection<MainDb>, data: CreateRegionData) -> Result<Region, ThisRegionError> {
        let existing = self.get_region(db).await;

        if existing.is_ok() {
            println!("Region already exists");
            // There is already a region, don't create another
            return Err(ThisRegionError::CannotCreate("Region already exists".to_string()));
        }

        let region: Region = self.create_region(db, data).await?;

        self.set_region_on_config(db, region.id.clone(), region.name.clone())
            .await?;

        println!("Created region");

        return self.get_region(db).await;
    }

    async fn create_region(&self, db: &mut Connection<MainDb>, data: CreateRegionData) -> Result<Region, ThisRegionError> {
        println!("Creating region");

        let region_id = Uuid::new_v4().to_string();

        let _region = sqlx::query!("INSERT INTO regions (id, name) VALUES (?, ?)", region_id, data.name)
            .execute(&mut ***db)
            .await
            .map_err(|_| ThisRegionError::InternalServerError("Database error".to_string()))?;

        return Ok(Region {
            network_id: data.name.clone(),
            id: region_id,
            name: data.name.clone(),
        });
    }

    async fn set_region_on_config(&self, db: &mut Connection<MainDb>, region_id: String, network_name: String) -> Result<(), ThisRegionError> {
        sqlx::query!(
            "
            UPDATE site_configs
            SET this_region_id = ?
            WHERE id = ?
            ",
            region_id,
            SITE_CONFIG_ID
        )
        .execute(&mut ***db)
        .await
        .map_err(|_| ThisRegionError::InternalServerError("Database error".to_string()))?;

        sqlx::query!(
            "
            UPDATE network_configs
            SET network_name = ?
            WHERE network_configs.id = ?
            ",
            network_name,
            NETWORK_CONFIG_ID
        )
        .execute(&mut ***db)
        .await
        .map_err(|_| ThisRegionError::InternalServerError("Database error".to_string()))?;

        return Ok(());
    }
}
