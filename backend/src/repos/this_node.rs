use crate::{
    infra::db::MainDb,
    repos::{
        entities::PrivateKeyRow,
        helpers::{NETWORK_CONFIG_ID, SITE_CONFIG_ID},
    },
};
use hex;
use p2panda_core::{identity::PRIVATE_KEY_LEN, PrivateKey};
use rocket_db_pools::Connection;
use sqlx;
use thiserror::Error;

pub struct ThisNodeRepo {}

#[derive(Debug, Error, Responder)]
pub enum ThisNodeError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),
}

pub struct BootstrapNodeDetails {
    pub node_id: String,
    pub ip4: String,
}

impl ThisNodeRepo {
    pub fn init() -> Self {
        ThisNodeRepo {}
    }

    pub async fn get_network_name(&self, db: &MainDb) -> Result<Option<String>, ThisNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let result = sqlx::query!(
            "
            SELECT network_name
            FROM network_configs
            WHERE network_configs.id = ?
            LIMIT 1
            ",
            NETWORK_CONFIG_ID
        )
        .fetch_optional(&mut *connection)
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Database error".to_string()))?;

        match result {
            None => return Ok(None),
            Some(result) => return Ok(result.network_name),
        }
    }

    pub async fn get_bootstrap_details(&self, db: &MainDb) -> Result<Option<BootstrapNodeDetails>, ThisNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let result = sqlx::query!(
            "
            SELECT bootstrap_node_id, bootstrap_node_ip4
            FROM network_configs
            WHERE network_configs.id = ?
            LIMIT 1
            ",
            NETWORK_CONFIG_ID
        )
        .fetch_optional(&mut *connection)
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Database error".to_string()))?;

        match result {
            None => return Ok(None),
            Some(result) => match result.bootstrap_node_id {
                None => return Ok(None),
                Some(node_id) => Ok(Some(BootstrapNodeDetails {
                    node_id: node_id,
                    ip4: result.bootstrap_node_ip4.unwrap(),
                })),
            },
        }
    }

    pub async fn set_network_config(
        &self,
        db: &mut Connection<MainDb>,
        network_name: String,
        bootstrap_node: BootstrapNodeDetails,
    ) -> Result<(), ThisNodeError> {
        let _region = sqlx::query!(
            "
            UPDATE network_configs
            SET network_name = ?, bootstrap_node_id = ?, bootstrap_node_ip4 = ?
            WHERE network_configs.id = ?
            ",
            network_name,
            bootstrap_node.node_id,
            bootstrap_node.ip4,
            NETWORK_CONFIG_ID
        )
        .execute(&mut ***db)
        .await;

        return Ok(());
    }

    pub async fn get_or_create_private_key(&self, db: &MainDb) -> Result<PrivateKey, ThisNodeError> {
        let private_key = self.get_private_key(db).await?;

        match private_key {
            None => {
                let new_private_key: PrivateKey = self.create_private_key(db).await?;
                return Ok(new_private_key);
            }
            Some(private_key) => {
                return Ok(private_key);
            }
        }
    }

    async fn get_private_key(&self, db: &MainDb) -> Result<Option<PrivateKey>, ThisNodeError> {
        let private_key_hex: Option<String> = self.get_private_key_hex(db).await?;

        match private_key_hex {
            None => return Ok(None),
            Some(private_key_hex) => {
                let private_key = Self::build_private_key_from_hex(private_key_hex)
                    .ok_or(ThisNodeError::InternalServerError("Failed to build private key".to_string()))?;

                return Ok(Some(private_key));
            }
        }
    }

    async fn create_private_key(&self, db: &MainDb) -> Result<PrivateKey, ThisNodeError> {
        let new_private_key = PrivateKey::new();

        self.set_private_key_hex(db, new_private_key.to_hex())
            .await?;

        println!("Created new private key");

        return Ok(new_private_key);
    }

    async fn set_private_key_hex(&self, db: &MainDb, private_key_hex: String) -> Result<(), ThisNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let _region = sqlx::query!(
            "
            UPDATE site_configs
            SET private_key_hex = ?
            WHERE site_configs.id = ?
            ",
            private_key_hex,
            SITE_CONFIG_ID
        )
        .execute(&mut *connection)
        .await;

        return Ok(());
    }

    async fn get_private_key_hex(&self, db: &MainDb) -> Result<Option<String>, ThisNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let result = sqlx::query_as!(
            PrivateKeyRow,
            "
            SELECT private_key_hex
            FROM site_configs
            WHERE site_configs.id = ?
            LIMIT 1
            ",
            SITE_CONFIG_ID
        )
        .fetch_one(&mut *connection)
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Database error".to_string()))?;

        return Ok(result.private_key_hex);
    }

    // TODO: This should be in p2panda-core, submit a PR
    fn build_private_key_from_hex(private_key_hex: String) -> Option<PrivateKey> {
        let private_key_bytes = hex::decode(private_key_hex).ok()?;
        let private_key_byte_array: [u8; PRIVATE_KEY_LEN] = private_key_bytes.try_into().ok()?;
        Some(PrivateKey::from_bytes(&private_key_byte_array))
    }
}
