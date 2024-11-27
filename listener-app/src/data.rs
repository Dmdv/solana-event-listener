use anchor_lang::prelude::Pubkey;
use derive_more::Display;

use crate::solana_logs_processor::Minted;

#[derive(Debug, Display)]
#[display(
    fmt = "{{ dst_address: {}, amount: {}, fee: {}, src_tx_hash: {}, src_chain_id: {}, nonce: {},  dst_tx_hash: {}, dst_block_slot: {}, dst_block_time: {} }}",
    dst_address,
    amount,
    fee,
    "hex::encode(src_tx_hash)",
    src_chain_id,
    nonce,
    dst_tx_hash,
    dst_block_slot,
    "chrono::DateTime::from_timestamp(*dst_block_time, 0).unwrap()"
)]
pub(crate) struct MintedEventData {
    pub(crate) dst_address: Pubkey,
    pub(crate) amount: u64,
    pub(crate) fee: u64,
    pub(crate) src_tx_hash: Vec<u8>,
    pub(crate) src_chain_id: u64,
    pub(crate) nonce: u64,
    pub(crate) dst_tx_hash: String,
    pub(crate) dst_block_slot: u64,
    pub(crate) dst_block_time: i64,
}

#[derive(Debug)]
pub(crate) struct SolanaTxMetadata {
    pub(crate) dst_tx_hash: String,
    pub(crate) dst_block_slot: u64,
    pub(crate) dst_block_time: Option<i64>,
}

impl From<(Minted, SolanaTxMetadata)> for MintedEventData {
    fn from(value: (Minted, SolanaTxMetadata)) -> Self {
        let minted_event = value.0;
        let solana_meta = value.1;
        MintedEventData {
            dst_address: minted_event.to,
            amount: minted_event.amount,
            fee: minted_event.fee,
            src_tx_hash: minted_event.src_tx_hash,
            src_chain_id: minted_event.src_chain_id,
            nonce: u64::from_be_bytes(
                *minted_event.nonce.last_chunk::<8>().expect("Malformed event nonce data"),
            ),
            dst_tx_hash: solana_meta.dst_tx_hash,
            dst_block_slot: solana_meta.dst_block_slot,
            dst_block_time: solana_meta.dst_block_time.expect("Expected dst_block_time to be set"),
        }
    }
}
