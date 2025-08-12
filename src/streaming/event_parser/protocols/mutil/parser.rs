use std::collections::HashMap;

use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    EventParserFactory, Protocol,
};

pub struct MutilEventParser {
    inner: GenericEventParser,
}

impl MutilEventParser {
    pub fn new(protocols: Vec<Protocol>) -> Self {
        let mut inner = GenericEventParser::new(vec![], vec![]);
        // Configure all event types
        for protocol in protocols {
            let parse = EventParserFactory::create_parser(protocol);

            // Merge inner_instruction_configs, append configurations to existing Vec
            for (key, configs) in parse.inner_instruction_configs() {
                inner.inner_instruction_configs.entry(key).or_insert_with(Vec::new).extend(configs);
            }

            // Merge instruction_configs, append configurations to existing Vec
            for (key, configs) in parse.instruction_configs() {
                inner.instruction_configs.entry(key).or_insert_with(Vec::new).extend(configs);
            }

            // Append program_ids (this is already appending)
            inner.program_ids.extend(parse.supported_program_ids().clone());
        }
        Self { inner }
    }
}

#[async_trait::async_trait]
impl EventParser for MutilEventParser {
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
