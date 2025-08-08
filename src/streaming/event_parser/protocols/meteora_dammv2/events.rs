use crate::impl_unified_event;
use crate::streaming::event_parser::common::EventMetadata;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDAMMv2SwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub trade_direction: TradeDirection,
    pub has_referral: u8,
    pub param_amount_in: u64,
    pub param_minimum_amount_out: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub partner_fee: u64,
    pub referral_fee: u64,
    pub amount_in: u64,
    pub current_timestamp: u64,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub token_a_mint: Pubkey,
    #[borsh(skip)]
    pub token_b_mint: Pubkey,
    #[borsh(skip)]
    pub input_token_account: Pubkey,
    #[borsh(skip)]
    pub output_token_account: Pubkey,
    #[borsh(skip)]
    pub token_a_vault: Pubkey,
    #[borsh(skip)]
    pub token_b_vault: Pubkey,
    #[borsh(skip)]
    pub token_a_program: Pubkey,
    #[borsh(skip)]
    pub token_b_program: Pubkey,
    #[borsh(skip)]
    pub remaining_accounts: Vec<Pubkey>,
}

impl_unified_event!(
    MeteoraDAMMv2SwapEvent,
    pool,
    trade_direction,
    has_referral,
    param_amount_in,
    param_minimum_amount_out,
    output_amount,
    next_sqrt_price,
    lp_fee,
    protocol_fee,
    partner_fee,
    referral_fee,
    amount_in,
    current_timestamp
);

/// 事件鉴别器常量
pub mod discriminators {

    pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1d1b3c15d58aaabb93";
    // 指令鉴别器
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];
}
