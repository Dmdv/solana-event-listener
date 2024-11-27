use log::{debug, error, info, warn};
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use tokio_postgres::{Client, NoTls};

use crate::{config::PostgresConfig, data::MintedEventData};

const POSTGRES_RECONNECT_SEC: u64 = 5;

pub(crate) struct PostgresWriter {
    postgres_config: PostgresConfig,
    mint_data_receiver: Arc<Mutex<UnboundedReceiver<MintedEventData>>>,
}

impl PostgresWriter {
    pub(crate) fn new(
        postgres_config: PostgresConfig,
        mint_data_receiver: UnboundedReceiver<MintedEventData>,
    ) -> PostgresWriter {
        PostgresWriter {
            postgres_config,
            mint_data_receiver: Arc::new(Mutex::new(mint_data_receiver)),
        }
    }

    pub(crate) async fn execute(&self) {
        info!("Connecting to the postgres to  write minted events");
        let conn_settings = format!(
            "host={} port={} user={} password={} dbname={}",
            self.postgres_config.host,
            self.postgres_config.port,
            self.postgres_config.username,
            self.postgres_config.password,
            self.postgres_config.dbname
        );
        loop {
            let Ok((client, connection)) =
                tokio_postgres::connect(conn_settings.as_str(), NoTls).await
            else {
                warn!("Failed to connect to the postgres, reconnect in 5 seconds");
                tokio::time::sleep(Duration::from_secs(POSTGRES_RECONNECT_SEC)).await;
                continue;
            };
            info!(
                "postgres connected: {}:{}:{}:{}",
                self.postgres_config.host,
                self.postgres_config.port,
                self.postgres_config.username,
                self.postgres_config.dbname
            );
            let receiver = self.mint_data_receiver.clone();
            let handle = tokio::spawn(process_minted_events(receiver, client));

            if let Err(e) = connection.await {
                error!("connection error: {}", e);
            }
            handle.abort();
        }

        async fn process_minted_events(
            receiver: Arc<Mutex<UnboundedReceiver<MintedEventData>>>,
            client: Client,
        ) {
            while let Some(mint_event_data) = receiver.lock().await.recv().await {
                debug!("Write to the database: {}", mint_event_data);
                let dst_block_time =
                    chrono::DateTime::from_timestamp(mint_event_data.dst_block_time, 0)
                        .expect("Expected dst_block_time be converted well");

                let query = format!("INSERT INTO minted_gorple (dst_address, amount, fee, src_tx_hash, src_chain_id, nonce, dst_tx_hash, dst_block_slot, dst_block_time) VALUES ('{dst_address}', {amount}, {fee}, '{src_tx_hash}', {src_chain_id}, {nonce}, '{dst_tx_hash}', {dst_block_slot}, '{dst_block_time}')",
                    dst_address=mint_event_data.dst_address,
                    amount=mint_event_data.amount,
                    fee=mint_event_data.fee,
                    src_tx_hash=hex::encode(mint_event_data.src_tx_hash),
                    src_chain_id=mint_event_data.src_chain_id,
                    nonce=mint_event_data.nonce,
                    dst_tx_hash=mint_event_data.dst_tx_hash,
                    dst_block_slot=mint_event_data.dst_block_slot,
                    dst_block_time= dst_block_time
                );

                if let Err(err) = client.simple_query(query.as_str()).await {
                    warn!("Failed to write minted_gorple, error: {}", err);
                }
            }
        }
    }
}
