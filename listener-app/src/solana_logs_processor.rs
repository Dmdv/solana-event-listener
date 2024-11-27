use anchor_lang::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use log::info;
use solana_sdk::commitment_config::CommitmentConfig;

use crate::data::{MintedEventData, SolanaTxMetadata};
use solana_tools::{
    solana_logs::{EventProcessor, LogsBunch, SolanaListenerConfig},
    solana_transactor::RpcPool,
};
use std::str::FromStr;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub(super) struct MintedEventProcessor {
    solana_config: SolanaListenerConfig,
    logs_receiver: Mutex<UnboundedReceiver<LogsBunch>>,
    rpc_pool: RpcPool,
    minted_sender: UnboundedSender<(Minted, SolanaTxMetadata)>,
    minted_receiver: Mutex<UnboundedReceiver<(Minted, SolanaTxMetadata)>>,
    minted_data_sender: UnboundedSender<MintedEventData>,
}

impl MintedEventProcessor {
    pub(super) fn new(
        solana_config: SolanaListenerConfig,
        logs_receiver: UnboundedReceiver<LogsBunch>,
        minted_data_sender: UnboundedSender<MintedEventData>,
    ) -> MintedEventProcessor {
        let rpc_pool =
            RpcPool::new(&solana_config.client.read_rpcs, &solana_config.client.write_rpcs)
                .expect("Expected rpc_pool be constructed");
        let (sender, receiver) = unbounded_channel();
        MintedEventProcessor {
            solana_config,
            logs_receiver: Mutex::new(logs_receiver),
            rpc_pool,
            minted_sender: sender,
            minted_receiver: Mutex::new(receiver),
            minted_data_sender,
        }
    }

    pub async fn execute(&self) {
        tokio::select! {
            _ = self.process_log_bunches() => {},
            _ = self.extend_minted_with_timestamp() => {}
        }
    }

    async fn process_log_bunches(&self) {
        let program = Pubkey::from_str(&self.solana_config.program_listen_to)
            .expect("Expected pubkey to be constructed from b58");
        while let Some(logs_bunch) = self.logs_receiver.lock().await.recv().await {
            self.on_logs(logs_bunch, program);
        }
    }

    async fn extend_minted_with_timestamp(&self) {
        info!("Start listening for Minted event");
        while let Some((minted, mut solana_meta)) = self.minted_receiver.lock().await.recv().await {
            let timestamp = self
                .rpc_pool
                .with_read_rpc_loop(
                    |client: RpcClient| async move {
                        client.get_block_time(solana_meta.dst_block_slot).await
                    },
                    CommitmentConfig::finalized(),
                )
                .await;
            solana_meta.dst_block_time = Some(timestamp);
            let minted_data = MintedEventData::from((minted, solana_meta));
            self.minted_data_sender.send(minted_data).expect("Expected minted_data to be sent");
        }
    }
}

#[derive(Debug)]
#[event]
pub(crate) struct Minted {
    pub(crate) to: Pubkey,
    pub(crate) amount: u64,
    pub(crate) fee: u64,
    pub(crate) src_tx_hash: Vec<u8>,
    pub(crate) src_chain_id: u64,
    pub(crate) nonce: Vec<u8>,
}

impl EventProcessor for MintedEventProcessor {
    type Event = Minted;

    fn on_event(&self, event: Self::Event, tx_signature: &str, slot: u64, _need_check: bool) {
        self.minted_sender
            .send((
                event,
                SolanaTxMetadata {
                    dst_tx_hash: tx_signature.to_string(),
                    dst_block_slot: slot,
                    dst_block_time: None,
                },
            ))
            .expect("Expected to be sent");
    }
}
