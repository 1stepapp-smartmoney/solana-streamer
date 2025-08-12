use crate::streaming::event_parser::core::traits::UnifiedEvent;
use crate::streaming::event_parser::protocols::block::block_meta_event::BlockMetaEvent;

pub struct CommonEventParser {}

impl CommonEventParser {
    pub fn generate_block_meta_event(
        slot: u64,
        block_hash: &str,
        block_time_ms: i64,
    ) -> Box<dyn UnifiedEvent> {
        let block_meta_event = BlockMetaEvent::new(slot, block_hash.to_string(), block_time_ms);
        Box::new(block_meta_event)
    }
}
