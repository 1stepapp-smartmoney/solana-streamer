use std::cmp::min;
use std::collections::HashMap;
use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use spl_associated_token_account::get_associated_token_address;
use crate::streaming::event_parser::{common::{read_u128_le, read_u64_le, read_u8_le, EventMetadata, EventType, ProtocolType}, UnifiedEvent};
use crate::streaming::event_parser::core::event_parser::GenericEventParseConfig;
use crate::streaming::event_parser::protocols::meteora_dammv2::{discriminators, MeteoraDAMMv2SwapEvent};
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;

/// Meteroa DBC程序ID
pub const METEORA_DAMM_V2_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");



pub const CONFIGS: &[GenericEventParseConfig] = &[
    GenericEventParseConfig {
        program_id: METEORA_DAMM_V2_PROGRAM_ID,
        protocol_type: ProtocolType::MeteoraDAMMv2,
        inner_instruction_discriminator: discriminators::TRADE_EVENT,
        instruction_discriminator: discriminators::SWAP,
        event_type: EventType::MeteoraDAMMv2Swap,
        inner_instruction_parser: Some(parse_trade_inner_instruction),
        instruction_parser: Some(parse_swap_instruction),
        requires_inner_instruction: false,
    },
];

fn parse_trade_inner_instruction(
    _data: &[u8],
    _metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    if let Ok(event) = borsh::from_slice::<MeteoraDAMMv2SwapEvent>(_data) {
        let mut metadata = _metadata;

        Some(Box::new(MeteoraDAMMv2SwapEvent {
            metadata,
            ..event
        }))
    } else {
        None
    }
}

/// 解析交易指令事件
fn parse_swap_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    if data.len() < 16 || accounts.len() < 14 {
        return None;
    }
    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    let mut metadata = metadata;

    let payer = accounts[8];
    let mint_a = accounts[6];
    let ming_b = accounts[7];

    let input_token_account = accounts[2];
    let output_token_account = accounts[3];

    let calculated_base_token_account = get_associated_token_address(&payer, &mint_a);
    // let calculated_quote_token_account = get_associated_token_address(&payer,&quote_mint);

    let tradedir = if calculated_base_token_account == output_token_account {
        TradeDirection::Buy
    } else {
        TradeDirection::Sell
    };



    Some(Box::new(MeteoraDAMMv2SwapEvent {
        metadata,
        pool: accounts[1].clone(),
        trade_direction: tradedir,
        param_amount_in: amount_in,
        param_minimum_amount_out: minimum_amount_out,
        amount_in,
        payer,
        token_a_mint: mint_a.clone(),
        token_b_mint: ming_b.clone(),
        input_token_account,
        output_token_account,
        token_a_vault: accounts[4].clone(),
        token_b_vault: accounts[5].clone(),
        token_a_program: accounts[9].clone(),
        token_b_program: accounts[10].clone(),
        remaining_accounts: vec![],
        ..Default::default()
    }))
}