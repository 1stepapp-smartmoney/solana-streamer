use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::axiom2::{discriminators, AxiomPumpSwapBuyEvent},
};
use crate::streaming::event_parser::common::read_u64_le;
use crate::streaming::event_parser::protocols::pumpswap::PumpSwapBuyEvent;

/// Axiom Trading Program 1程序ID
pub const AXIOM_2_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("AxiomxSitiyXyPjKgJ9XSrdhsydtZsskZTEDam3PxKcC");

/// Axiom Trading Program 1事件解析器
pub struct Axiom2EventParser {
    inner: GenericEventParser,
}

impl Default for Axiom2EventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Axiom2EventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::AXIOM_2_PUMPSWAP_BUY_IX,
                event_type: EventType::AxiomPumpSwapBuy,
                inner_instruction_parser: Self::parse_axiom_trade_inner_instruction,
                instruction_parser: Self::parse_axiom_pumpswap_buy_instruction,
            },
        ];

        let inner = GenericEventParser::new(AXIOM_2_PROGRAM_ID, ProtocolType::AxiomTrading2, configs);

        Self { inner }
    }
    fn parse_axiom_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    // 解析pumpswap买入指令事件
    fn parse_axiom_pumpswap_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 16 || accounts.len() < 19 {
            return None;
        }
        let base_amount_out = read_u64_le(data, 8)?;
        let max_quote_amount_in = read_u64_le(data, 0)?;

        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature, accounts[1], accounts[0], base_amount_out
        ));

        Some(Box::new(AxiomPumpSwapBuyEvent {
            metadata,
            base_amount_out,
            max_quote_amount_in,
            pool: accounts[0],
            user: accounts[1],
            base_mint: accounts[3],
            quote_mint: accounts[4],
            user_base_token_account: accounts[5],
            user_quote_token_account: accounts[6],
            pool_base_token_account: accounts[7],
            pool_quote_token_account: accounts[8],
            protocol_fee_recipient: accounts[9],
            protocol_fee_recipient_token_account: accounts[10],
            coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
            coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
            global_volume_accumulator: accounts.get(19).copied().unwrap_or_default(),
            user_volume_accumulator: accounts.get(20).copied().unwrap_or_default(),
            ..Default::default()
        }))
    }

}

#[async_trait::async_trait]
impl EventParser for Axiom2EventParser {
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
