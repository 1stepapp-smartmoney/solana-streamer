use crate::impl_unified_event;
use crate::streaming::event_parser::common::{types::EventType, EventMetadata};
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};

/// Block元数据事件
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct BlockMetaEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub slot: u64,
    pub block_hash: String,
}

impl BlockMetaEvent {
    pub fn new(slot: u64, block_hash: String, block_time_ms: i64) -> Self {
        let metadata = EventMetadata::new(
            format!("block_{}_{}", slot, block_hash),
            "".to_string(),
            slot,
            block_time_ms / 1000,
            block_time_ms,
            crate::streaming::event_parser::common::types::ProtocolType::Common,
            EventType::BlockMeta,
            solana_sdk::pubkey::Pubkey::default(),
            "".to_string(),
            chrono::Utc::now().timestamp_millis(),
        );
        Self { metadata, slot, block_hash }
    }
}

// 使用macro生成UnifiedEvent实现
impl_unified_event!(BlockMetaEvent,);
