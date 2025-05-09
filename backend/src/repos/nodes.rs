use sqlx::Sqlite;
use thiserror::Error;

use super::entities::Node;

pub struct NodesRepo {}

#[derive(Debug, Error, Responder)]
pub enum NodesError {
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

impl NodesRepo {
    pub fn init() -> Self {
        NodesRepo {}
    }

    pub async fn upsert(&self, pool: &sqlx::Pool<Sqlite>, site: Node) -> Result<(), NodesError> {
        let mut connection = pool.acquire().await.unwrap();

        let _site = sqlx::query!(
            "INSERT INTO nodes (id, name) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET name = ?",
            site.id,
            site.name,
            site.name
        )
        .execute(&mut *connection)
        .await
        .map_err(|_| NodesError::InternalServerError("Database error".to_string()))?;

        Ok(())
    }
}
