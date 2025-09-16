pub mod types;
pub mod utils;
pub mod filter;
pub mod high_performance_clock;

/// 自动生成UnifiedEvent trait实现的宏
#[macro_export]
macro_rules! impl_unified_event {
    // 带有自定义ID表达式的版本
    ($struct_name:ident, $($field:ident),*) => {
        impl $crate::streaming::event_parser::core::traits::UnifiedEvent for $struct_name {
            fn event_type(&self) -> $crate::streaming::event_parser::common::types::EventType {
                self.metadata.event_type.clone()
            }

            fn signature(&self) -> &solana_sdk::signature::Signature {
                &self.metadata.signature
            }

            fn slot(&self) -> u64 {
                self.metadata.slot
            }

            fn recv_us(&self) -> i64 {
                self.metadata.recv_us
            }

            fn handle_us(&self) -> i64 {
                self.metadata.handle_us
            }

            fn set_handle_us(&mut self, handle_us: i64) {
                self.metadata.handle_us = handle_us;
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn $crate::streaming::event_parser::core::traits::UnifiedEvent> {
                Box::new(self.clone())
            }

            fn merge(&mut self, other: &dyn $crate::streaming::event_parser::core::traits::UnifiedEvent) {
                if let Some(_e) = other.as_any().downcast_ref::<$struct_name>() {
                    $(
                        self.$field = _e.$field.clone();
                    )*
                }
            }

            fn set_swap_data(&mut self, swap_data: $crate::streaming::event_parser::common::types::SwapData) {
                self.metadata.set_swap_data(swap_data);
            }

            fn swap_data_is_parsed(&self) -> bool {
                self.metadata.swap_data.is_some()
            }

            fn outer_index(&self) -> i64 {
                self.metadata.outer_index
            }

            fn inner_index(&self) -> Option<i64> {
                self.metadata.inner_index
            }
            fn transaction_index(&self) -> Option<u64> {
                self.metadata.transaction_index
            }
        }
    };
}

pub use types::*;
pub use utils::*;
