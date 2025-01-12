use crate::{
    infra::db::MainDb,
    repos::{entities::PrivateKeyRow, helpers::SITE_CONFIG_ID},
};
use hex;
use p2panda_core::{identity::PRIVATE_KEY_LEN, PrivateKey};
use sqlx;
use thiserror::Error;

pub struct ThisP2PandaNodeRepo {}

#[derive(Debug, Error, Responder)]
pub enum ThisP2PandaNodeError {
    #[error("Internal server error: {0}")]
    #[response(status = 500)]
    InternalServerError(String),
}

impl ThisP2PandaNodeRepo {
    pub fn init() -> Self {
        ThisP2PandaNodeRepo {}
    }

    pub async fn get_network_name(&self, db: &MainDb) -> Result<Option<String>, ThisP2PandaNodeError> {
        let mut connection = db.sqlite_pool().acquire().await.unwrap();

        let result = sqlx::query!(
            "
            SELECT name
            FROM regions
            INNER JOIN site_configs ON regions.id = site_configs.this_region_id
            WHERE site_configs.id = ?
            LIMIT 1
            ",
            SITE_CONFIG_ID
        )
        .fetch_optional(&mut *connection)
        .await
        .map_err(|_| ThisP2PandaNodeError::InternalServerError("Database error".to_string()))?;

        match result {
            None => return Ok(None),
            Some(result) => return Ok(Some(result.name)),
        }
    }

    pub async fn get_or_create_private_key(&self, db: &MainDb) -> Result<PrivateKey, ThisP2PandaNodeError> {
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

    async fn get_private_key(&self, db: &MainDb) -> Result<Option<PrivateKey>, ThisP2PandaNodeError> {
        let private_key_hex: Option<String> = self.get_private_key_hex(db).await?;

        match private_key_hex {
            None => return Ok(None),
            Some(private_key_hex) => {
                let private_key = Self::build_private_key_from_hex(private_key_hex)
                    .ok_or(ThisP2PandaNodeError::InternalServerError("Failed to build private key".to_string()))?;

                return Ok(Some(private_key));
            }
        }
    }

    async fn create_private_key(&self, db: &MainDb) -> Result<PrivateKey, ThisP2PandaNodeError> {
        let new_private_key = PrivateKey::new();

        self.set_private_key_hex(db, new_private_key.to_hex())
            .await?;

        println!("Created new private key");

        return Ok(new_private_key);
    }

    async fn set_private_key_hex(&self, db: &MainDb, private_key_hex: String) -> Result<(), ThisP2PandaNodeError> {
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

    async fn get_private_key_hex(&self, db: &MainDb) -> Result<Option<String>, ThisP2PandaNodeError> {
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
        .map_err(|_| ThisP2PandaNodeError::InternalServerError("Database error".to_string()))?;

        return Ok(result.private_key_hex);
    }

    // TODO: This should be in p2panda-core, submit a PR
    fn build_private_key_from_hex(private_key_hex: String) -> Option<PrivateKey> {
        let private_key_bytes = hex::decode(private_key_hex).ok()?;
        let private_key_byte_array: [u8; PRIVATE_KEY_LEN] = private_key_bytes.try_into().ok()?;
        Some(PrivateKey::from_bytes(&private_key_byte_array))
    }
}
