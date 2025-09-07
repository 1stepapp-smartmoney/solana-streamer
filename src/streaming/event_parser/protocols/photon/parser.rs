use std::collections::HashMap;
use prost_types::Timestamp;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::UiCompiledInstruction;
use spl_associated_token_account::get_associated_token_address;
use crate::impl_event_parser_delegate;
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::traits::{EventParser, GenericEventParseConfig, GenericEventParser, UnifiedEvent},
    protocols::photon::{discriminators, PhotonPumpFunTradeEvent},
};
use crate::streaming::event_parser::common::read_u64_le;
use crate::streaming::event_parser::protocols::meteora_dbc::types::TradeDirection;
use crate::streaming::event_parser::protocols::photon::PhotonPumpSwapTradeEvent;
use crate::streaming::event_parser::protocols::pumpfun::PumpFunTradeEvent;
use crate::streaming::event_parser::protocols::pumpswap::{PumpSwapBuyEvent, PumpSwapSellEvent};
use crate::streaming::event_parser::protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID;

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
                program_id: PHOTON_PROGRAM_ID,
                protocol_type: ProtocolType::PhotonProtocol,
                inner_instruction_discriminator: &[],
                instruction_discriminator: discriminators::PHOTON_PUMPFUN_BUY_IX,
                event_type: EventType::PhotonPumpFunBuy,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_photon_pumpfun_buy_instruction),
            },
            GenericEventParseConfig {
                program_id: PHOTON_PROGRAM_ID,
                protocol_type: ProtocolType::PhotonProtocol,
                inner_instruction_discriminator: &[],
                instruction_discriminator: discriminators::PHOTON_PUMPFUN_SELL_IX,
                event_type: EventType::PhotonPumpFunSell,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_photon_pumpfun_sell_instruction),
            },
            GenericEventParseConfig {
                program_id: PHOTON_PROGRAM_ID,
                protocol_type: ProtocolType::PhotonProtocol,
                inner_instruction_discriminator: &[],
                instruction_discriminator: discriminators::PHOTON_PUMPSWAP_TRADE_IX,
                event_type: EventType::PhotonPumpSwapTrade,
                inner_instruction_parser: None,
                instruction_parser: Some(Self::parse_photon_pumpswap_trade_instruction),
            },
        ];

        let inner = GenericEventParser::new(vec![PHOTON_PROGRAM_ID], configs);

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

        if data.len() < 32 || accounts.len() < 17 {
            return None;
        }

        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let max_sol_cost = u64::from_le_bytes(data[8..16].try_into().unwrap());

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

        if data.len() < 32 || accounts.len() < 15 {
            return None;
        }
        let amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let min_sol_output = u64::from_le_bytes(data[16..24].try_into().unwrap());

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

    fn parse_pumpswap_inner_instruction(
        data: &[u8],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        None
    }


    fn parse_photon_pumpswap_trade_instruction(
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        if data.len() < 16 || accounts.len() < 21 {
            return None;
        }

        let user = accounts[1].clone();
        let base_mint = accounts[3].clone();
        let quote_mint = accounts[4].clone();

        let input_token_account = accounts[5];
        let output_token_account = accounts[6];

        let calculated_user_base_token_account = get_associated_token_address(&user,&base_mint);
        // let calculated_quote_token_account = get_associated_token_address(&payer,&quote_mint);

        let is_buy = calculated_user_base_token_account == output_token_account;


        if (is_buy) {
            let base_amount_out = read_u64_le(data, 8)?;
            let max_quote_amount_in = read_u64_le(data, 0)?;

            let buyevt = PhotonPumpSwapTradeEvent {
                metadata,
                base_amount_out,
                max_quote_amount_in,
                is_buy,
                pool: accounts[0],
                user,
                base_mint,
                quote_mint,
                user_base_token_account: output_token_account,
                user_quote_token_account: input_token_account,
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

            let sellevt = PhotonPumpSwapTradeEvent {
                metadata,
                base_amount_in,
                min_quote_amount_out,
                is_buy,
                pool: accounts[0],
                user,
                base_mint,
                quote_mint,
                user_base_token_account: input_token_account,
                user_quote_token_account: output_token_account,
                pool_base_token_account: accounts[7],
                pool_quote_token_account: accounts[8],
                protocol_fee_recipient: accounts[9],
                protocol_fee_recipient_token_account: accounts[10],
                coin_creator_vault_ata: accounts.get(17).copied().unwrap_or_default(),
                coin_creator_vault_authority: accounts.get(18).copied().unwrap_or_default(),
                global_volume_accumulator: accounts.get(19).copied().unwrap_or_default(),
                user_volume_accumulator: accounts.get(20).copied().unwrap_or_default(),
                ..Default::default()
            };
            Some(Box::new(sellevt))
        }

    }

}

impl_event_parser_delegate!(PhotonEventParser);