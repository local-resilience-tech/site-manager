use sqlx::Sqlite;
use thiserror::Error;

use super::entities::Node;

pub struct NodesRepo {}

#[derive(Debug, Error, Responder)]
pub enum NodesError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),
    // #[error("Cannot create node")]
    // #[response(status = 409)]
    // CannotCreate(String),

    // #[error("Node not found")]
    // #[response(status = 404)]
    // NotFound(String),
}

impl NodesRepo {
    pub fn init() -> Self {
        NodesRepo {}
    }

    pub async fn upsert(&self, pool: &sqlx::Pool<Sqlite>, node: Node) -> Result<(), NodesError> {
        let mut connection = pool.acquire().await.unwrap();

        let _node = sqlx::query!(
            "INSERT INTO nodes (id, name) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET name = ?",
            node.id,
            node.name,
            node.name
        )
        .execute(&mut *connection)
        .await
        .map_err(|_| NodesError::InternalServerError("Database error".to_string()))?;

        Ok(())
    }
}
