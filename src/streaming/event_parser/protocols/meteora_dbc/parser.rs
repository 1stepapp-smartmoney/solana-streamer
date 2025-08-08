use std::cmp::min;
use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use spl_associated_token_account::get_associated_token_address;
use crate::streaming::event_parser::{
    common::{read_u128_le, read_u64_le, read_u8_le, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
};
use crate::streaming::event_parser::protocols::meteora_dbc::discriminators;
use crate::streaming::event_parser::protocols::meteora_dbc::events::MeteoraDBCSwapEvent;
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;
use crate::streaming::event_parser::protocols::pumpfun::PumpFunTradeEvent;

/// Meteroa DBC程序ID
pub const METEORA_DBC_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");

/// Meteroa DBC事件解析器
pub struct MeteoraDBCEventParser {
    inner: GenericEventParser,
}

impl Default for MeteoraDBCEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MeteoraDBCEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SWAP,
                event_type: EventType::MeteoraDBCSwap,
                inner_instruction_parser: Self::parse_trade_inner_instruction,
                instruction_parser: Self::parse_swap_instruction,
            },
        ];

        let inner =
            GenericEventParser::new(METEORA_DBC_PROGRAM_ID, ProtocolType::MeteoraDBC, configs);

        Self { inner }
    }

    /// 解析交易事件
    fn parse_trade_inner_instruction(
        _data: &[u8],
        _metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<MeteoraDBCSwapEvent>(_data) {
            let mut metadata = _metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature,
                event.pool,
                event.config,
                event.current_timestamp
            ));
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

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[2], accounts[7], accounts[9]
        ));

        let payer = accounts[9];
        let base_mint = accounts[7];
        let quote_mint = accounts[8];

        let input_token_account = accounts[3];
        let output_token_account = accounts[4];

        let calculated_base_token_account = get_associated_token_address(&payer,&base_mint);
        // let calculated_quote_token_account = get_associated_token_address(&payer,&quote_mint);

        let isBuy = if calculated_base_token_account == output_token_account {
            TradeDirection::Buy
        } else {
            TradeDirection::Sell
        };


        Some(Box::new(MeteoraDBCSwapEvent {
            metadata,
            pool: accounts[0].clone(),
            config: accounts[1].clone(),
            trade_direction: isBuy,
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
}

#[async_trait::async_trait]
impl EventParser for MeteoraDBCEventParser {
    fn parse_events_from_inner_instruction(
        &self,
        inner_instruction: &UiCompiledInstruction,
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_inner_instruction(
            inner_instruction,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

    fn parse_events_from_instruction(
        &self,
        instruction: &CompiledInstruction,
        accounts: &[Pubkey],
        signature: &str,
        slot: u64,
        block_time: Option<Timestamp>,
        program_received_time_ms: i64,
        index: String,
    ) -> Vec<Box<dyn UnifiedEvent>> {
        self.inner.parse_events_from_instruction(
            instruction,
            accounts,
            signature,
            slot,
            block_time,
            program_received_time_ms,
            index,
        )
    }

    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.inner.should_handle(program_id)
    }

    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.inner.supported_program_ids()
    }
}
