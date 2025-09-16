use std::cmp::min;
use std::collections::HashMap;
use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use spl_associated_token_account::get_associated_token_address;
use crate::streaming::event_parser::common::{read_u64_le, EventMetadata, EventType, ProtocolType};
use crate::streaming::event_parser::core::event_parser::GenericEventParseConfig;
use crate::streaming::event_parser::protocols::meteora_dbc::discriminators;
use crate::streaming::event_parser::protocols::meteora_dbc::events::MeteoraDBCSwapEvent;
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;
use crate::streaming::event_parser::protocols::pumpfun::PumpFunTradeEvent;
use crate::streaming::event_parser::UnifiedEvent;

/// Meteroa DBC程序ID
pub const METEORA_DBC_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");

/// 匹配所有事件
pub const CONFIGS: &[GenericEventParseConfig] = &[
    GenericEventParseConfig {
        program_id: METEORA_DBC_PROGRAM_ID,
        protocol_type: ProtocolType::MeteoraDBC,
        inner_instruction_discriminator: discriminators::TRADE_EVENT,
        instruction_discriminator: discriminators::SWAP,
        event_type: EventType::MeteoraDBCSwap,
        inner_instruction_parser: Some(parse_trade_inner_instruction),
        instruction_parser: Some(parse_swap_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: METEORA_DBC_PROGRAM_ID,
        protocol_type: ProtocolType::MeteoraDBC,
        inner_instruction_discriminator: discriminators::TRADE_EVENT,
        instruction_discriminator: discriminators::SWAP2,
        event_type: EventType::MeteoraDBCSwap2,
        inner_instruction_parser: Some(parse_trade_inner_instruction),
        instruction_parser: Some(parse_swap_instruction),
        requires_inner_instruction: false,
    },
];

fn parse_trade_inner_instruction(
    _data: &[u8],
    _metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    if let Ok(event) = borsh::from_slice::<MeteoraDBCSwapEvent>(_data) {
        let metadata = _metadata;
        Some(Box::new(MeteoraDBCSwapEvent {
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
    if data.len() < 16 || accounts.len() < 15 {
        return None;
    }
    let amount_in = read_u64_le(data, 0)?;
    let minimum_amount_out = read_u64_le(data, 8)?;

    let payer = accounts[9];
    let base_mint = accounts[7];
    let quote_mint = accounts[8];

    let input_token_account = accounts[3];
    let output_token_account = accounts[4];

    let calculated_base_token_account = get_associated_token_address(&payer,&base_mint);
    // let calculated_quote_token_account = get_associated_token_address(&payer,&quote_mint);

    let tradedir = if calculated_base_token_account == output_token_account {
        TradeDirection::Buy
    } else {
        TradeDirection::Sell
    };

    Some(Box::new(MeteoraDBCSwapEvent {
        metadata,
        pool: accounts[0].clone(),
        config: accounts[1].clone(),
        trade_direction: tradedir,
        param_amount_in: amount_in,
        param_minimum_amount_out: minimum_amount_out,
        amount_in,
        payer,
        base_mint,
        quote_mint,
        input_token_account,
        output_token_account,
        base_vault: accounts[5].clone(),
        quote_vault: accounts[6].clone(),
        token_base_program: accounts[10].clone(),
        token_quote_program: accounts[11].clone(),
        remaining_accounts: vec![],
        ..Default::default()
    }))
}