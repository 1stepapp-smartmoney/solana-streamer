pub mod common;
pub mod core;
pub mod protocols;

pub use core::traits::UnifiedEvent;
pub use protocols::types::Protocol;

/// 宏：简化 downcast_ref 模式匹配
///
/// # 使用示例
/// ```
/// use sol_trade_sdk::event_parser::match_event;
///
/// match_event!(event, {
///     PumpSwapCreatePoolEvent => |typed_event| {
///         println!("CreatePool event: {:?}", typed_event);
///     },
///     PumpSwapDepositEvent => |typed_event| {
///         // 处理存款事件
///     },
/// });
/// ```
#[macro_export]
macro_rules! match_event {
    ($event:expr, {
        $($event_type:ty => $handler:expr),* $(,)?
    }) => {
        $(
            if let Some(typed_event) = $event.as_any().downcast_ref::<$event_type>() {
                $handler(typed_event.clone());
            } else
        )*
        {
            // 默认情况：什么都不做
        }
    };
}

// 重新导出宏以便于使用
pub use match_event;
