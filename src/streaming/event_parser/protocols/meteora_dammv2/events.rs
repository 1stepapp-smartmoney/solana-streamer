use crate::impl_unified_event;
use crate::streaming::event_parser::common::EventMetadata;
// use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteoraDAMMv2SwapEvent {
    pub metadata: EventMetadata,
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub payer: Pubkey,
    pub config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub token_base_program: Pubkey,
    pub token_quote_program: Pubkey,
    pub is_buy: bool,
    pub remaining_accounts: Vec<Pubkey>,
}

impl_unified_event!(MeteoraDAMMv2SwapEvent,);

/// 事件鉴别器常量
pub mod discriminators {
    // 指令鉴别器
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];
}
