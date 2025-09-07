
use solana_sdk::pubkey::Pubkey;
use crate::impl_event_parser_delegate;
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::ProgF5tfv::{discriminators, ProgF5tfvPumpFunTradeEvent},
};

/// F5tfv Program ID  疑似Axiom
pub const PROGF5TFV_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("F5tfvbLog9VdGUPqBDTT8rgXvTTcq7e5UiGnupL1zvBq");

/// F5tfv  Program 事件解析器
pub struct F5tfvEventParser {
    inner: GenericEventParser,
}

impl Default for F5tfvEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl F5tfvEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                program_id: PROGF5TFV_PROGRAM_ID,
                protocol_type: ProtocolType::ProgramF5tfv,
                inner_instruction_discriminator: &[],
                instruction_discriminator: discriminators::PROGF5TFV_PUMPFUN_BUY_IX,
                event_type: EventType::F5tfvPumpFunBuy,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_prog_f5tfv_pumpfun_buy_instruction),
            },
        ];

        let inner = GenericEventParser::new(vec![PROGF5TFV_PROGRAM_ID], configs);

        Self { inner }
    }
    fn parse_prog_f5tfv_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    // 解析pumpfun买入指令事件
    fn parse_prog_f5tfv_pumpfun_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 16 || accounts.len() < 14 {
            return None;
        }

        let amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[0..8].try_into().unwrap()) + 1;


        Some(Box::new(ProgF5tfvPumpFunTradeEvent {
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

impl_event_parser_delegate!(F5tfvEventParser);