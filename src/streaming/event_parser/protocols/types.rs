use crate::streaming::event_parser::protocols::{
    bonk::parser::BONK_PROGRAM_ID, pumpfun::parser::PUMPFUN_PROGRAM_ID,
    pumpswap::parser::PUMPSWAP_PROGRAM_ID, raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
    raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID, raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use crate::streaming::event_parser::protocols::axiom2::parser::{AXIOM_2_PROGRAM_ID, AXIOM_2_PROGRAM_VAR_1_ID};
use crate::streaming::event_parser::protocols::axiom::parser::{AXIOM_1_PROGRAM_ID, AXIOM_1_PROGRAM_VAR_1_ID};
use crate::streaming::event_parser::protocols::meteora_dammv2::parser::METEORA_DAMM_V2_PROGRAM_ID;
use crate::streaming::event_parser::protocols::meteora_dbc::parser::METEORA_DBC_PROGRAM_ID;
use crate::streaming::event_parser::protocols::photon::parser::PHOTON_PROGRAM_ID;

/// 支持的协议
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    Phonton,
    MeteoraDBC,
    MeteoraDAMMv2,
    AxiomProgram1,
    AxiomProgram2,
}

impl Protocol {
    pub fn get_program_id(&self) -> Vec<Pubkey> {
        match self {
            Protocol::PumpSwap => vec![PUMPSWAP_PROGRAM_ID],
            Protocol::PumpFun => vec![PUMPFUN_PROGRAM_ID],
            Protocol::Bonk => vec![BONK_PROGRAM_ID],
            Protocol::RaydiumCpmm => vec![RAYDIUM_CPMM_PROGRAM_ID],
            Protocol::RaydiumClmm => vec![RAYDIUM_CLMM_PROGRAM_ID],
            Protocol::RaydiumAmmV4 => vec![RAYDIUM_AMM_V4_PROGRAM_ID],
            Protocol::Phonton => vec![PHOTON_PROGRAM_ID],
            Protocol::MeteoraDBC => vec![METEORA_DBC_PROGRAM_ID],
            Protocol::MeteoraDAMMv2 => vec![METEORA_DAMM_V2_PROGRAM_ID],
            Protocol::AxiomProgram1 => vec![AXIOM_1_PROGRAM_ID, AXIOM_1_PROGRAM_VAR_1_ID],
            Protocol::AxiomProgram2 => vec![AXIOM_2_PROGRAM_ID, AXIOM_2_PROGRAM_VAR_1_ID],

        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::PumpSwap => write!(f, "PumpSwap"),
            Protocol::PumpFun => write!(f, "PumpFun"),
            Protocol::Bonk => write!(f, "Bonk"),
            Protocol::RaydiumCpmm => write!(f, "RaydiumCpmm"),
            Protocol::RaydiumClmm => write!(f, "RaydiumClmm"),
            Protocol::RaydiumAmmV4 => write!(f, "RaydiumAmmV4"),
            Protocol::Phonton => write!(f, "Photon"),
            Protocol::MeteoraDBC => write!(f, "MeteoraDBC"),
            Protocol::MeteoraDAMMv2 => write!(f, "MeteoraDAMMv2"),
            Protocol::AxiomProgram1 => write!(f, "AxiomTradingProgram1"),
            Protocol::AxiomProgram2 => write!(f, "AxiomTradingProgram2"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pumpswap" => Ok(Protocol::PumpSwap),
            "pumpfun" => Ok(Protocol::PumpFun),
            "bonk" => Ok(Protocol::Bonk),
            "raydiumcpmm" => Ok(Protocol::RaydiumCpmm),
            "raydiumclmm" => Ok(Protocol::RaydiumClmm),
            "raydiumammv4" => Ok(Protocol::RaydiumAmmV4),
            "photon" => Ok(Protocol::Phonton),
            "meteoradbc" => Ok(Protocol::MeteoraDBC),
            "meteoradammv2" => Ok(Protocol::MeteoraDAMMv2),
            "axiomprogram1" => Ok(Protocol::AxiomProgram1),
            "axiomprogram2" => Ok(Protocol::AxiomProgram2),
            _ => Err(anyhow!("Unsupported protocol: {}", s)),
        }
    }
}
