use borsh_derive::BorshDeserialize;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum TradeDirection {
    #[default]
    Sell,
    Buy,
}