use sqlx::Sqlite;
use thiserror::Error;

use super::entities::Site;

pub struct SitesRepo {}

#[derive(Debug, Error, Responder)]
pub enum SitesError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),
    // #[error("Cannot create site")]
    // #[response(status = 409)]
    // CannotCreate(String),

    // #[error("Site not found")]
    // #[response(status = 404)]
    // NotFound(String),
}

impl SitesRepo {
    pub fn init() -> Self {
        SitesRepo {}
    }

    pub async fn create_site(&self, pool: &sqlx::Pool<Sqlite>, site: Site) -> Result<(), SitesError> {
        let mut connection = pool.acquire().await.unwrap();

        let _site = sqlx::query!("INSERT INTO sites (id, name) VALUES (?, ?)", site.id, site.name)
            .execute(&mut *connection)
            .await
            .map_err(|_| SitesError::InternalServerError("Database error".to_string()))?;

        Ok(())
    }
}
