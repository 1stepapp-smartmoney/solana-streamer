use std::collections::HashMap;
use std::sync::OnceLock;

use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::{EventMetadata, EventType, ProtocolType};
use crate::streaming::event_parser::core::traits::UnifiedEvent;
use crate::streaming::event_parser::protocols::bonk::parser::BONK_PROGRAM_ID;
use crate::streaming::event_parser::protocols::pumpfun::parser::PUMPFUN_PROGRAM_ID;
use crate::streaming::event_parser::protocols::pumpswap::parser::PUMPSWAP_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID;
use crate::streaming::event_parser::protocols::raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID;
use crate::streaming::event_parser::Protocol;
use crate::streaming::grpc::AccountPretty;

/// 通用事件解析器配置
#[derive(Debug, Clone)]
pub struct AccountEventParseConfig {
    pub program_id: Pubkey,
    pub protocol_type: ProtocolType,
    pub event_type: EventType,
    pub account_discriminator: &'static [u8],
    pub account_parser: AccountEventParserFn,
}

/// 账户事件解析器
pub type AccountEventParserFn =
    fn(account: &AccountPretty, metadata: EventMetadata) -> Option<Box<dyn UnifiedEvent>>;

static PROTOCOL_CONFIGS_CACHE: OnceLock<HashMap<Protocol, Vec<AccountEventParseConfig>>> =
    OnceLock::new();

pub struct AccountEventParser {}

impl AccountEventParser {
    pub fn configs(protocols: Vec<Protocol>, event_type_filter: Option<EventTypeFilter>) -> Vec<AccountEventParseConfig> {
        let protocols_map = PROTOCOL_CONFIGS_CACHE.get_or_init(|| {
            let mut map: HashMap<Protocol, Vec<AccountEventParseConfig>> = HashMap::new();
            map.insert(Protocol::PumpSwap, vec![
                AccountEventParseConfig {
                    program_id: PUMPSWAP_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpSwap,
                    event_type: EventType::AccountPumpSwapGlobalConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpswap::discriminators::GLOBAL_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpswap::types::global_config_parser,
                },
                AccountEventParseConfig {
                    program_id: PUMPSWAP_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpSwap,
                    event_type: EventType::AccountPumpSwapPool,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpswap::discriminators::POOL_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpswap::types::pool_parser,
                },
            ]);
            map.insert(Protocol::PumpFun, vec![
                AccountEventParseConfig {
                    program_id: PUMPFUN_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpFun,
                    event_type: EventType::AccountPumpFunBondingCurve,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpfun::discriminators::BONDING_CURVE_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpfun::types::bonding_curve_parser,
                },
                AccountEventParseConfig {
                    program_id: PUMPFUN_PROGRAM_ID,
                    protocol_type: ProtocolType::PumpFun,
                    event_type: EventType::AccountPumpFunGlobal,
                    account_discriminator: crate::streaming::event_parser::protocols::pumpfun::discriminators::GLOBAL_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::pumpfun::types::global_parser,
                },
            ]);
            map.insert(Protocol::Bonk, vec![
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::POOL_STATE_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::pool_state_parser,
                },
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkGlobalConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::GLOBAL_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::global_config_parser,
                },
                AccountEventParseConfig {
                    program_id: BONK_PROGRAM_ID,
                    protocol_type: ProtocolType::Bonk,
                    event_type: EventType::AccountBonkPlatformConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::bonk::discriminators::PLATFORM_CONFIG_ACCOUNT,
                    account_parser: crate::streaming::event_parser::protocols::bonk::types::platform_config_parser,
                },
            ]);
            map.insert(Protocol::RaydiumCpmm, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_CPMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumCpmm,
                    event_type: EventType::AccountRaydiumCpmmAmmConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_cpmm::discriminators::AMM_CONFIG,
                    account_parser: crate::streaming::event_parser::protocols::raydium_cpmm::types::amm_config_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CPMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumCpmm,
                    event_type: EventType::AccountRaydiumCpmmPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_cpmm::discriminators::POOL_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_cpmm::types::pool_state_parser,
                },
            ]);
            map.insert(Protocol::RaydiumClmm, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmAmmConfig,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::AMM_CONFIG,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::amm_config_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmPoolState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::POOL_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::pool_state_parser,
                },
                AccountEventParseConfig {
                    program_id: RAYDIUM_CLMM_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumClmm,
                    event_type: EventType::AccountRaydiumClmmTickArrayState,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_clmm::discriminators::TICK_ARRAY_STATE,
                    account_parser: crate::streaming::event_parser::protocols::raydium_clmm::types::tick_array_state_parser,
                },
            ]);
            map.insert(Protocol::RaydiumAmmV4, vec![
                AccountEventParseConfig {
                    program_id: RAYDIUM_AMM_V4_PROGRAM_ID,
                    protocol_type: ProtocolType::RaydiumAmmV4,
                    event_type: EventType::AccountRaydiumAmmV4AmmInfo,
                    account_discriminator: crate::streaming::event_parser::protocols::raydium_amm_v4::discriminators::AMM_INFO,
                    account_parser: crate::streaming::event_parser::protocols::raydium_amm_v4::types::amm_info_parser,
                },
            ]);
            map
        });

        let mut configs = vec![];
        for protocol in protocols {
            let protocol_configs = protocols_map.get(&protocol).unwrap_or(&vec![]).clone();
            let filtered_configs: Vec<AccountEventParseConfig> = protocol_configs.into_iter().filter(|config| {
                event_type_filter.as_ref().map(|filter| filter.include.contains(&config.event_type)).unwrap_or(true)
            }).collect();
            configs.extend(filtered_configs);
        }
        configs
    }

    pub fn parse_account_event(
        protocols: Vec<Protocol>,
        account: AccountPretty,
        program_received_time_ms: i64,
        event_type_filter: Option<EventTypeFilter>,
    ) -> Option<Box<dyn UnifiedEvent>> {
        let configs = Self::configs(protocols, event_type_filter);
        for config in configs {
            if account.owner == config.program_id.to_string()
                && account.data[..config.account_discriminator.len()]
                    == *config.account_discriminator
            {
                let event = (config.account_parser)(
                    &account,
                    EventMetadata {
                        slot: account.slot,
                        signature: account.signature.clone(),
                        protocol: config.protocol_type,
                        event_type: config.event_type,
                        program_id: config.program_id,
                        program_received_time_ms,
                        ..Default::default()
                    },
                );
                if let Some(mut event) = event {
                    event.set_program_handle_time_consuming_ms(
                        chrono::Utc::now().timestamp_millis() - program_received_time_ms,
                    );
                    return Some(event);
                }
            }
        }
        None
    }
}
