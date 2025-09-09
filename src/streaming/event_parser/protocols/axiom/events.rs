use crate::impl_unified_event;
use crate::streaming::event_parser::common::EventMetadata;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Trade event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct AxiomPumpFunTradeEvent {
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
    AxiomPumpFunTradeEvent,
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


/// Event discriminator constants
pub mod discriminators {

    pub const AXIOM_PUMPFUN_BUY_IX: &[u8] = &[0];

}
