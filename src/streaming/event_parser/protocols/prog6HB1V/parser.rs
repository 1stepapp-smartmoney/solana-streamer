use std::collections::HashMap;
use prost_types::Timestamp;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiCompiledInstruction;
use crate::impl_event_parser_delegate;
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
};
use crate::streaming::event_parser::common::read_u64_le;
use crate::streaming::event_parser::protocols::prog6HB1V::{discriminators, Prog6HB1VPumpSwapBuyEvent};

/// Program 6HB1V ID  疑似Axiom2
pub const PROG6HB1V_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("6HB1VBBS8LrdQiR9MZcXV5VdpKFb7vjTMZuQQEQEPioC");

/// Program 6HB1V 事件解析器
pub struct Prog6HB1VEventParser {
    inner: GenericEventParser,
}

impl Default for Prog6HB1VEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Prog6HB1VEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                program_id: PROG6HB1V_PROGRAM_ID,
                protocol_type: ProtocolType::Program6HB1V,
                inner_instruction_discriminator: &[],
                instruction_discriminator: discriminators::PROG6HB1V_PUMPSWAP_BUY_IX,
                event_type: EventType::Prog6HB1VPumpSwapBuy,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_prog6hb1v_pumpswap_buy_instruction),
            },
        ];

        let inner = GenericEventParser::new(vec![PROG6HB1V_PROGRAM_ID], configs);

        Self { inner }
    }
    fn parse_prog6hb1v_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    // 解析pumpswap买入指令事件
    fn parse_prog6hb1v_pumpswap_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 16 || accounts.len() < 19 {
            return None;
        }
        let base_amount_out = read_u64_le(data, 8)?;
        let max_quote_amount_in = read_u64_le(data, 0)?;


        Some(Box::new(Prog6HB1VPumpSwapBuyEvent {
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

impl_event_parser_delegate!(Prog6HB1VEventParser);