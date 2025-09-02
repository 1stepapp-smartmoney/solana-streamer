use crate::impl_unified_event;
use crate::streaming::event_parser::common::EventMetadata;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MeteoraDBCSwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool: Pubkey,
    pub config: Pubkey,
    pub trade_direction: TradeDirection,
    pub has_referral: u8,
    pub param_amount_in: u64,
    pub param_minimum_amount_out: u64,
    pub actual_input_amount: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub referral_fee: u64,
    pub amount_in: u64,
    pub current_timestamp: u64,
    #[borsh(skip)]
    pub payer: Pubkey,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub input_token_account: Pubkey,
    #[borsh(skip)]
    pub output_token_account: Pubkey,
    #[borsh(skip)]
    pub base_vault: Pubkey,
    #[borsh(skip)]
    pub quote_vault: Pubkey,
    #[borsh(skip)]
    pub token_base_program: Pubkey,
    #[borsh(skip)]
    pub token_quote_program: Pubkey,
    #[borsh(skip)]
    pub remaining_accounts: Vec<Pubkey>,
}

impl_unified_event!(
    MeteoraDBCSwapEvent,
    pool,
    config,
    trade_direction,
    has_referral,
    param_amount_in,
    param_minimum_amount_out,
    actual_input_amount,
    output_amount,
    next_sqrt_price,
    trading_fee,
    protocol_fee,
    referral_fee,
    amount_in,
    current_timestamp
);

/// 事件鉴别器常量
pub mod discriminators {

    pub const TRADE_EVENT: &str = "0xe445a52e51cb9a1d1b3c15d58aaabb93";
    // 指令鉴别器
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];
    pub const SWAP2: &[u8] = &[65, 75, 63, 76, 235, 91, 91, 136];
}
