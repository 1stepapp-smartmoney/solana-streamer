use std::collections::HashMap;

use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::pumpfun::{
        discriminators, PumpFunCreateTokenEvent, PumpFunMigrateEvent, PumpFunTradeEvent,
    },
};

/// PumpFun程序ID
pub const PUMPFUN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

/// PumpFun事件解析器
pub struct PumpFunEventParser {
    inner: GenericEventParser,
}

impl Default for PumpFunEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PumpFunEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                program_id: PUMPFUN_PROGRAM_ID,
                protocol_type: ProtocolType::PumpFun,
                inner_instruction_discriminator: discriminators::CREATE_TOKEN_EVENT,
                instruction_discriminator: discriminators::CREATE_TOKEN_IX,
                event_type: EventType::PumpFunCreateToken,
                inner_instruction_parser: Some(Self::parse_create_token_inner_instruction),
                instruction_parser: Some(Self::parse_create_token_instruction),
            },
            GenericEventParseConfig {
                program_id: PUMPFUN_PROGRAM_ID,
                protocol_type: ProtocolType::PumpFun,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::BUY_IX,
                event_type: EventType::PumpFunBuy,
                inner_instruction_parser: Some(Self::parse_trade_inner_instruction),
                instruction_parser: Some(Self::parse_buy_instruction),
            },
            GenericEventParseConfig {
                program_id: PUMPFUN_PROGRAM_ID,
                protocol_type: ProtocolType::PumpFun,
                inner_instruction_discriminator: discriminators::TRADE_EVENT,
                instruction_discriminator: discriminators::SELL_IX,
                event_type: EventType::PumpFunSell,
                inner_instruction_parser: Some(Self::parse_trade_inner_instruction),
                instruction_parser: Some(Self::parse_sell_instruction),
            },
            GenericEventParseConfig {
                program_id: PUMPFUN_PROGRAM_ID,
                protocol_type: ProtocolType::PumpFun,
                inner_instruction_discriminator: discriminators::COMPLETE_PUMP_AMM_MIGRATION_EVENT,
                instruction_discriminator: discriminators::MIGRATE_IX,
                event_type: EventType::PumpFunMigrate,
                inner_instruction_parser: Some(Self::parse_migrate_inner_instruction),
                instruction_parser: Some(Self::parse_migrate_instruction),
            },
        ];

        let inner = GenericEventParser::new(vec![PUMPFUN_PROGRAM_ID], configs);

        Self { inner }
    }

    /// 解析迁移事件
    fn parse_migrate_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<PumpFunMigrateEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!("{}-{}-{}", metadata.signature, event.user, event.mint));
            Some(Box::new(PumpFunMigrateEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// 解析创建代币日志事件
    fn parse_create_token_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<PumpFunCreateTokenEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature, event.name, event.symbol, event.mint
            ));
            Some(Box::new(PumpFunCreateTokenEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// 解析交易事件
    fn parse_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        let usefuldata = if (data.len()>250) {
            data[..250].to_vec() 
        } else {
            data.to_vec()
        }; // 只截取部分字节，防止pump日后再加字段。

        if let Ok(event) = borsh::from_slice::<PumpFunTradeEvent>(&usefuldata[..]) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature, event.mint, event.user, event.is_buy
            ));
            Some(Box::new(PumpFunTradeEvent { metadata, ..event }))
        } else {
            None
        }
    }

    /// 解析创建代币指令事件
    fn parse_create_token_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 11 {
            return None;
        }
        let mut offset = 0;
        let name_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + name_len > data.len() {
            return None; // 防止越界
        }
        let name = String::from_utf8_lossy(&data[offset..offset + name_len]);
        offset += name_len;
        let symbol_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + symbol_len > data.len() {
            return None; // 防止越界
        }
        let symbol = String::from_utf8_lossy(&data[offset..offset + symbol_len]);
        offset += symbol_len;
        let uri_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if offset + uri_len > data.len() {
            return None; // 防止越界
        }
        let uri = String::from_utf8_lossy(&data[offset..offset + uri_len]);
        offset += uri_len;
        let creator = if offset + 32 <= data.len() {
            Pubkey::new_from_array(data[offset..offset + 32].try_into().ok()?)
        } else {
            Pubkey::default()
        };

        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}-{}", metadata.signature, name, symbol, accounts[0]));

        Some(Box::new(PumpFunCreateTokenEvent {
            metadata,
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            creator,
            mint: accounts[0],
            mint_authority: accounts[1],
            bonding_curve: accounts[2],
            associated_bonding_curve: accounts[3],
            user: accounts[7],
            ..Default::default()
        }))
    }

    // 解析买入指令事件
    fn parse_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 14 {
            return None;
        }
        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}-{}", metadata.signature, accounts[2], accounts[6], true));
        Some(Box::new(PumpFunTradeEvent {
            metadata,
            global: accounts[0],
            fee_recipient: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            associated_user: accounts[5],
            user: accounts[6],
            system_program: accounts[7],
            token_program: accounts[8],
            creator_vault: accounts[9],
            event_authority: accounts[10],
            program: accounts[11],
            global_volume_accumulator: accounts[12],
            user_volume_accumulator: accounts[13],
            max_sol_cost,
            amount,
            is_buy: true,
            ..Default::default()
        }))
    }

    // 解析卖出指令事件
    fn parse_sell_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 12 {
            return None;
        }
        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let min_sol_output = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut metadata = metadata;
        metadata
            .set_id(format!("{}-{}-{}-{}", metadata.signature, accounts[2], accounts[6], false));
        Some(Box::new(PumpFunTradeEvent {
            metadata,
            global: accounts[0],
            fee_recipient: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            associated_user: accounts[5],
            user: accounts[6],
            system_program: accounts[7],
            creator_vault: accounts[8],
            token_program: accounts[9],
            event_authority: accounts[10],
            program: accounts[11],
            global_volume_accumulator: *accounts.get(12).unwrap_or(&Pubkey::default()),
            user_volume_accumulator: *accounts.get(13).unwrap_or(&Pubkey::default()),
            min_sol_output,
            amount,
            is_buy: false,
            ..Default::default()
        }))
    }

    /// 解析迁移指令事件
    fn parse_migrate_instruction(
        _data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if accounts.len() < 24 {
            return None;
        }
        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, accounts[5], accounts[2]));
        Some(Box::new(PumpFunMigrateEvent {
            metadata,
            global: accounts[0],
            withdraw_authority: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            user: accounts[5],
            system_program: accounts[6],
            token_program: accounts[7],
            pump_amm: accounts[8],
            pool: accounts[9],
            pool_authority: accounts[10],
            pool_authority_mint_account: accounts[11],
            pool_authority_wsol_account: accounts[12],
            amm_global_config: accounts[13],
            wsol_mint: accounts[14],
            lp_mint: accounts[15],
            user_pool_token_account: accounts[16],
            pool_base_token_account: accounts[17],
            pool_quote_token_account: accounts[18],
            token_2022_program: accounts[19],
            associated_token_program: accounts[20],
            pump_amm_event_authority: accounts[21],
            event_authority: accounts[22],
            program: accounts[23],
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for PumpFunEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner.inner_instruction_configs()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.inner.instruction_configs()
    }
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
