use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;

use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::photon::{discriminators, PhotonPumpFunTradeEvent},
};
use crate::streaming::event_parser::common::read_u64_le;
use crate::streaming::event_parser::protocols::photon::PhotonPumpSwapTradeEvent;
use crate::streaming::event_parser::protocols::pumpfun::PumpFunTradeEvent;
use crate::streaming::event_parser::protocols::pumpswap::{PumpSwapBuyEvent, PumpSwapSellEvent};

/// PumpFun程序ID
pub const PHOTON_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("BSfD6SHZigAfDWSjzD5Q41jw8LmKwtmjskPH9XW1mrRW");

/// PumpFun事件解析器
pub struct PhotonEventParser {
    inner: GenericEventParser,
}

impl Default for PhotonEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PhotonEventParser {
    pub fn new() -> Self {
        // 配置所有事件类型
        let configs = vec![
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::PHOTON_PUMPFUN_BUY_IX,
                event_type: EventType::PhotonPumpFunBuy,
                inner_instruction_parser: Self::parse_pumpfun_trade_inner_instruction,
                instruction_parser: Self::parse_photon_pumpfun_buy_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::PHOTON_PUMPFUN_SELL_IX,
                event_type: EventType::PhotonPumpFunSell,
                inner_instruction_parser: Self::parse_pumpfun_trade_inner_instruction,
                instruction_parser: Self::parse_photon_pumpfun_sell_instruction,
            },
            GenericEventParseConfig {
                inner_instruction_discriminator: "",
                instruction_discriminator: discriminators::PHOTON_PUMPSWAP_TRADE_IX,
                event_type: EventType::PhotonPumpSwapTrade,
                inner_instruction_parser: Self::parse_pumpswap_buy_inner_instruction,
                instruction_parser: Self::parse_photon_pumpswap_trade_instruction,
            },
        ];

        let inner = GenericEventParser::new(PHOTON_PROGRAM_ID, ProtocolType::PhotonProtocol, configs);

        Self { inner }
    }
    fn parse_pumpfun_trade_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    // 解析pumpfun买入指令事件
    fn parse_photon_pumpfun_buy_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 32 {
            return None;
        }


        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            accounts[2],
            accounts[6],
            true
        ));
        Some(Box::new(PhotonPumpFunTradeEvent {
            metadata,
            fee_recipient: accounts[1],
            mint: accounts[3],
            bonding_curve: accounts[4],
            associated_bonding_curve: accounts[5],
            associated_user: accounts[6],
            user: accounts[7],
            creator_vault: accounts[14],
            global_volume_accumulator: accounts[15],
            user_volume_accumulator: accounts[16],
            max_sol_cost,
            amount,
            is_buy: true,
            ..Default::default()
        }))
    }

    fn parse_photon_pumpfun_sell_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {

        if data.len() < 32 {
            return None;
        }
        let amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let min_sol_output = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let mut metadata = metadata;
        metadata.set_id(format!(
            "{}-{}-{}-{}",
            metadata.signature,
            accounts[2],
            accounts[6],
            false
        ));
        Some(Box::new(PumpFunTradeEvent {
            metadata,
            fee_recipient: accounts[1],
            mint: accounts[3],
            bonding_curve: accounts[4],
            associated_bonding_curve: accounts[5],
            associated_user: accounts[6],
            user: accounts[7],
            creator_vault: accounts[14],
            min_sol_output,
            amount,
            is_buy: false,
            ..Default::default()
        }))
    }

    fn parse_pumpswap_buy_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }

    /// 解析卖出日志事件
    fn parse_pumpswap_sell_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if let Ok(event) = borsh::from_slice::<PumpSwapSellEvent>(data) {
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature, event.user, event.pool, event.base_amount_in
            ));
            Some(Box::new(PumpSwapSellEvent {
                metadata,
                ..event
            }))
        } else {
            None
        }
    }

    fn parse_photon_pumpswap_trade_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 19 {
            return None;
        }


        if (accounts.len() > 19) {
            let base_amount_out = read_u64_le(data, 8)?;
            let max_quote_amount_in = read_u64_le(data, 0)?;
            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature,
                accounts[2],
                accounts[6],
                true
            ));
            let buyevt = PhotonPumpSwapTradeEvent {
                metadata,
                base_amount_out,
                max_quote_amount_in,
                is_buy: true,
                pool: accounts[0],
                user: accounts[1],
                base_mint: accounts[3],
                quote_mint: accounts[4],
                user_base_token_account: accounts[6],
                user_quote_token_account: accounts[5],
                pool_base_token_account: accounts[7],
                pool_quote_token_account: accounts[8],
                protocol_fee_recipient: accounts[9],
                protocol_fee_recipient_token_account: accounts[10],
                coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
                coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
                ..Default::default()
            };
            Some(Box::new(buyevt))

        } else {
            let base_amount_in = read_u64_le(data, 0)?;
            let min_quote_amount_out = read_u64_le(data, 8)?;

            let mut metadata = metadata;
            metadata.set_id(format!(
                "{}-{}-{}-{}",
                metadata.signature, accounts[1], accounts[0], base_amount_in
            ));

            let sellevt = PhotonPumpSwapTradeEvent {
                metadata,
                base_amount_in,
                min_quote_amount_out,
                is_buy: false,
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
                ..Default::default()
            };
            Some(Box::new(sellevt))
        }

    }

}

#[async_trait::async_trait]
impl EventParser for PhotonEventParser {
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
