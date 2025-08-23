use std::collections::HashMap;

use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{read_u64_le, EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::raydium_cpmm::{
        discriminators, RaydiumCpmmDepositEvent, RaydiumCpmmInitializeEvent, RaydiumCpmmSwapEvent,
        RaydiumCpmmWithdrawEvent,
    },
};

/// Raydium CPMM程序ID
pub const RAYDIUM_CPMM_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

/// Raydium CPMM事件解析器
pub struct RaydiumCpmmEventParser {
    inner: GenericEventParser,
}

impl Default for RaydiumCpmmEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RaydiumCpmmEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_IN,
                event_type: EventType::RaydiumCpmmSwapBaseInput,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_swap_base_input_instruction),
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::SWAP_BASE_OUT,
                event_type: EventType::RaydiumCpmmSwapBaseOutput,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_swap_base_output_instruction),
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::DEPOSIT,
                event_type: EventType::RaydiumCpmmDeposit,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_deposit_instruction),
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::INITIALIZE,
                event_type: EventType::RaydiumCpmmInitialize,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_initialize_instruction),
            },
            GenericEventParseConfig {
                program_id: RAYDIUM_CPMM_PROGRAM_ID,
                protocol_type: ProtocolType::RaydiumCpmm,
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::WITHDRAW,
                event_type: EventType::RaydiumCpmmWithdraw,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_withdraw_instruction),
            },
        ];

        let inner = GenericEventParser::new(vec![RAYDIUM_CPMM_PROGRAM_ID], configs);

        Self { inner }
    }

    /// 解析提款指令事件
    fn parse_withdraw_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 14 {
            return None;
        }
        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, accounts[0], accounts[1]));
        Some(Box::new(RaydiumCpmmWithdrawEvent {
            metadata,
            lp_token_amount: read_u64_le(data, 0)?,
            minimum_token0_amount: read_u64_le(data, 8)?,
            minimum_token1_amount: read_u64_le(data, 16)?,
            owner: accounts[0],
            authority: accounts[1],
            pool_state: accounts[2],
            owner_lp_token: accounts[3],
            token0_account: accounts[4],
            token1_account: accounts[5],
            token0_vault: accounts[6],
            token1_vault: accounts[7],
            token_program: accounts[8],
            token_program2022: accounts[9],
            vault0_mint: accounts[10],
            vault1_mint: accounts[11],
            lp_mint: accounts[12],
            memo_program: accounts[13],
        }))
    }

    /// 解析初始化指令事件
    fn parse_initialize_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 20 {
            return None;
        }
        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, accounts[0], accounts[1]));
        Some(Box::new(RaydiumCpmmInitializeEvent {
            metadata,
            init_amount0: read_u64_le(data, 0)?,
            init_amount1: read_u64_le(data, 8)?,
            open_time: read_u64_le(data, 16)?,
            creator: accounts[0],
            amm_config: accounts[1],
            authority: accounts[2],
            pool_state: accounts[3],
            token0_mint: accounts[4],
            token1_mint: accounts[5],
            lp_mint: accounts[6],
            creator_token0: accounts[7],
            creator_token1: accounts[8],
            creator_lp_token: accounts[9],
            token0_vault: accounts[10],
            token1_vault: accounts[11],
            create_pool_fee: accounts[12],
            observation_state: accounts[13],
            token_program: accounts[14],
            token0_program: accounts[15],
            token1_program: accounts[16],
            associated_token_program: accounts[17],
            system_program: accounts[18],
            rent: accounts[19],
        }))
    }

    /// 解析存款指令事件
    fn parse_deposit_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 24 || accounts.len() < 13 {
            return None;
        }
        let mut metadata = metadata;
        metadata.set_id(format!("{}-{}-{}", metadata.signature, accounts[0], accounts[1]));
        Some(Box::new(RaydiumCpmmDepositEvent {
            metadata,
            lp_token_amount: read_u64_le(data, 0)?,
            maximum_token0_amount: read_u64_le(data, 8)?,
            maximum_token1_amount: read_u64_le(data, 16)?,
            owner: accounts[0],
            authority: accounts[1],
            pool_state: accounts[2],
            owner_lp_token: accounts[3],
            token0_account: accounts[4],
            token1_account: accounts[5],
            token0_vault: accounts[6],
            token1_vault: accounts[7],
            token_program: accounts[8],
            token_program2022: accounts[9],
            vault0_mint: accounts[10],
            vault1_mint: accounts[11],
            lp_mint: accounts[12],
        }))
    }

    /// 解析买入指令事件
    fn parse_swap_base_input_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 13 {
            return None;
        }

        let amount_in = read_u64_le(data, 0)?;
        let minimum_amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[3], accounts[10], accounts[11]
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata,
            amount_in,
            minimum_amount_out,
            payer: accounts[0],
            authority: accounts[1],
            amm_config: accounts[2],
            pool_state: accounts[3],
            input_token_account: accounts[4],
            output_token_account: accounts[5],
            input_vault: accounts[6],
            output_vault: accounts[7],
            input_token_program: accounts[8],
            output_token_program: accounts[9],
            input_token_mint: accounts[10],
            output_token_mint: accounts[11],
            observation_state: accounts[12],
            ..Default::default()
        }))
    }

    fn parse_swap_base_output_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 13 {
            return None;
        }

        let max_amount_in = read_u64_le(data, 0)?;
        let amount_out = read_u64_le(data, 8)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[3], accounts[10], accounts[11]
        ));

        Some(Box::new(RaydiumCpmmSwapEvent {
            metadata,
            max_amount_in,
            amount_out,
            payer: accounts[0],
            authority: accounts[1],
            amm_config: accounts[2],
            pool_state: accounts[3],
            input_token_account: accounts[4],
            output_token_account: accounts[5],
            input_vault: accounts[6],
            output_vault: accounts[7],
            input_token_program: accounts[8],
            output_token_program: accounts[9],
            input_token_mint: accounts[10],
            output_token_mint: accounts[11],
            observation_state: accounts[12],
            ..Default::default()
        }))
    }
}

#[async_trait::async_trait]
impl EventParser for RaydiumCpmmEventParser {
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
