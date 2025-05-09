use crate::{
    infra::db::MainDb,
    repos::{
        entities::PrivateKeyRow,
        helpers::{NETWORK_CONFIG_ID, NODE_CONFIG_ID},
    },
};
use hex;
use p2panda_core::{identity::PRIVATE_KEY_LEN, PrivateKey};
use rocket_db_pools::Connection;
use sqlx;
use thiserror::Error;

pub struct ThisPandaNodeRepo {}

#[derive(Debug, Error, Responder)]
pub enum ThisNodeError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),
}

#[derive(Clone)]
pub struct SimplifiedNodeAddress {
    pub node_id: String,
}

impl ThisPandaNodeRepo {
    pub fn init() -> Self {
        ThisPandaNodeRepo {}
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

    // TODO: I don't know how to handle the DB connection in these two different ways, this is a
    // temporary solution
    pub async fn get_network_name_conn(&self, connection: &mut Connection<MainDb>) -> Result<Option<String>, ThisNodeError> {
        let result = sqlx::query!(
            "
            SELECT network_name
            FROM network_configs
            WHERE network_configs.id = ?
            LIMIT 1
            ",
            NETWORK_CONFIG_ID
        )
        .fetch_optional(&mut ***connection)
        .await
        .map_err(|_| ThisNodeError::InternalServerError("Database error".to_string()))?;

        match result {
            None => return Ok(None),
            Some(result) => return Ok(result.network_name),
        }
    }

    pub async fn get_bootstrap_details(&self, db: &MainDb) -> Result<Option<SimplifiedNodeAddress>, ThisNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let result = sqlx::query!(
            "
            SELECT bootstrap_node_id
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
                Some(node_id) => Ok(Some(SimplifiedNodeAddress { node_id })),
            },
        }
    }

    pub async fn set_network_config(
        &self,
        db: &mut Connection<MainDb>,
        network_name: String,
        peer_address: Option<SimplifiedNodeAddress>,
    ) -> Result<(), ThisNodeError> {
        let bootstrap_node_id = peer_address
            .as_ref()
            .map(|peer| peer.node_id.clone());

        let _region = sqlx::query!(
            "
            UPDATE network_configs
            SET network_name = ?, bootstrap_node_id = ?
            WHERE network_configs.id = ?
            ",
            network_name,
            bootstrap_node_id,
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
            UPDATE node_configs
            SET private_key_hex = ?
            WHERE node_configs.id = ?
            ",
            private_key_hex,
            NODE_CONFIG_ID
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
            FROM node_configs
            WHERE node_configs.id = ?
            LIMIT 1
            ",
            NODE_CONFIG_ID
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
