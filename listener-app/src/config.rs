use config::{Config, File};
use log::{error, info};
use serde::Deserialize;
use solana_tools::solana_logs::SolanaListenerConfig;
use tokio_postgres::NoTls;

use super::error::ListenError;

#[derive(Deserialize)]
pub(crate) struct ListenConfig {
    pub(crate) solana: SolanaListenerConfig,
    pub(crate) postgres: PostgresConfig,
}

#[derive(Clone, Deserialize)]
pub(crate) struct PostgresConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) dbname: String,
}

impl ListenConfig {
    pub(super) fn try_from_path(config: &str) -> Result<ListenConfig, ListenError> {
        info!("Read config from path: {}", config);
        let config = Config::builder()
            .add_source(File::with_name(config))
            .add_source(config::Environment::with_prefix("SOLANA").separator("_"))
            .build()
            .map_err(|err| {
                error!("Failed to build envs due to the error: {}", err);
                ListenError::Config
            })?;

        config.try_deserialize().map_err(|err| {
            error!("Failed to deserialize config: {}", err);
            ListenError::Config
        })
    }

    pub(super) async fn get_tx_read_from(&self) -> Result<String, ()> {
        if let Some(tx_read_from_force) = &self.solana.tx_read_from_force {
            return Ok(tx_read_from_force.clone());
        }
        Ok(self
            .get_tx_read_from_from_db()
            .await
            .unwrap_or_else(|_| self.solana.tx_read_from.clone()))
    }

    async fn get_tx_read_from_from_db(&self) -> Result<String, ()> {
        let conn_settings = format!(
            "host={} port={} user={} password={} dbname={}",
            self.postgres.host,
            self.postgres.port,
            self.postgres.username,
            self.postgres.password,
            self.postgres.dbname
        );
        let (client, connection) =
            tokio_postgres::connect(conn_settings.as_str(), NoTls).await.map_err(|err| {
                error!("Failed to connect to the postgres to read last_processed_tx: {}", err);
            })?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("connection error: {}", e);
            }
        });
        let result = client
            .query_one("select dst_tx_hash from minted_gorple order by id desc limit 1", &[])
            .await
            .map_err(|err| {
                error!("Failed to get last dst_tx_hash from minted_gorple: {}", err);
            })?;
        Ok(result.get(0))
    }
}
