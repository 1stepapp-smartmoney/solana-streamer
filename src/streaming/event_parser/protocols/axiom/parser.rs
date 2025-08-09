use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::axiom::{discriminators, AxiomPumpFunTradeEvent},
};

/// Axiom Trading Program 1程序ID
pub const AXIOM_1_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("AxiomfHaWDemCFBLBayqnEnNwE6b7B2Qz3UmzMpgbMG6");

/// Axiom Trading Program 1事件解析器
pub struct AxiomEventParser {
    inner: GenericEventParser,
}

impl Default for AxiomEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AxiomEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::AXIOM_PUMPFUN_BUY_IX,
                event_type: EventType::AxiomPumpFunBuy,
                inner_instruction_parser: Self::parse_axiom_trade_inner_instruction,
                instruction_parser: Self::parse_axiom_pumpfun_buy_instruction,
            },
        ];

        let inner = GenericEventParser::new(AXIOM_1_PROGRAM_ID, ProtocolType::AxiomTrading1, configs);

        Self { inner }
    }
    fn parse_axiom_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    // 解析pumpfun买入指令事件
    fn parse_axiom_pumpfun_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 16 || accounts.len() < 14 {
            return None;
        }

        let amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[0..8].try_into().unwrap()) + 1;
        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            accounts[2],
            accounts[6],
            true
        ));

        Some(Box::new(AxiomPumpFunTradeEvent {
            metadata,
            fee_recipient: accounts[1],
            mint: accounts[2],
            bonding_curve: accounts[3],
            associated_bonding_curve: accounts[4],
            associated_user: accounts[5],
            user: accounts[6],
            creator_vault: accounts[9],
            global_volume_accumulator: accounts[12],
            user_volume_accumulator: accounts[13],
            max_sol_cost,
            amount,
            is_buy: true,
            ..Default::default()
        }))


    }

}

#[async_trait::async_trait]
impl EventParser for AxiomEventParser {
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
