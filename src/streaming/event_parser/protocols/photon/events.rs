use crate::impl_unified_event;
use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::bonk::types::{
    CurveParams, MintParams, PoolStatus, TradeDirection, VestingParams,
};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Trade event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PhotonPumpFunTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,

    #[borsh(skip)]
    pub max_sol_cost: u64,
    #[borsh(skip)]
    pub min_sol_output: u64,
    #[borsh(skip)]
    pub amount: u64,
    #[borsh(skip)]
    pub is_bot: bool,
    #[borsh(skip)]
    pub is_dev_create_token_trade: bool, // 是否是dev创建token的交易

    #[borsh(skip)]
    pub global: Pubkey,
    // #[borsh(skip)]
    // pub fee_recipient: Pubkey,
    // #[borsh(skip)]
    // pub mint: Pubkey,
    #[borsh(skip)]
    pub bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_bonding_curve: Pubkey,
    #[borsh(skip)]
    pub associated_user: Pubkey,
    // #[borsh(skip)]
    // pub user: Pubkey,
    #[borsh(skip)]
    pub system_program: Pubkey,
    #[borsh(skip)]
    pub token_program: Pubkey,
    #[borsh(skip)]
    pub creator_vault: Pubkey,
    #[borsh(skip)]
    pub event_authority: Pubkey,
    #[borsh(skip)]
    pub program: Pubkey,
    #[borsh(skip)]
    pub global_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub user_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub fee_config: Pubkey,
    #[borsh(skip)]
    pub fee_program: Pubkey,
}

impl_unified_event!(
    PhotonPumpFunTradeEvent,
    mint,
    sol_amount,
    token_amount,
    is_buy,
    user,
    timestamp,
    virtual_sol_reserves,
    virtual_token_reserves,
    real_sol_reserves,
    real_token_reserves,
    fee_recipient,
    fee_basis_points,
    fee,
    creator,
    creator_fee_basis_points,
    creator_fee
);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct PhotonPumpSwapTradeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub base_amount_in: u64,
    pub min_quote_amount_out: u64,
    pub is_buy: bool,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    #[borsh(skip)]
    pub base_mint: Pubkey,
    #[borsh(skip)]
    pub quote_mint: Pubkey,
    #[borsh(skip)]
    pub pool_base_token_account: Pubkey,
    #[borsh(skip)]
    pub pool_quote_token_account: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_ata: Pubkey,
    #[borsh(skip)]
    pub coin_creator_vault_authority: Pubkey,
    #[borsh(skip)]
    pub base_token_program: Pubkey,
    #[borsh(skip)]
    pub quote_token_program: Pubkey,
    #[borsh(skip)]
    pub global_volume_accumulator: Pubkey,
    #[borsh(skip)]
    pub user_volume_accumulator: Pubkey,
}

// 使用宏生成UnifiedEvent实现，指定需要合并的字段
impl_unified_event!(
    PhotonPumpSwapTradeEvent,
    timestamp,
    base_amount_out,
    max_quote_amount_in,
    base_amount_in,
    min_quote_amount_out,
    is_buy,
    user_base_token_reserves,
    user_quote_token_reserves,
    pool_base_token_reserves,
    pool_quote_token_reserves,
    quote_amount_in,
    lp_fee_basis_points,
    lp_fee,
    protocol_fee_basis_points,
    protocol_fee,
    quote_amount_in_with_lp_fee,
    user_quote_amount_in,
    pool,
    user,
    user_base_token_account,
    user_quote_token_account,
    protocol_fee_recipient,
    protocol_fee_recipient_token_account,
    coin_creator,
    coin_creator_fee_basis_points,
    coin_creator_fee
);


/// Event discriminator constants
pub mod discriminators {

    pub const PHOTON_PUMPFUN_BUY_IX: &[u8] = &[82, 225, 119, 231, 78, 29, 45, 70];
    pub const PHOTON_PUMPFUN_SELL_IX: &[u8] = &[93, 88, 60, 34, 91, 18, 86, 197];

    pub const PHOTON_PUMPSWAP_TRADE_IX: &[u8] = &[44, 119, 175, 218, 199, 77, 196, 235];
}
