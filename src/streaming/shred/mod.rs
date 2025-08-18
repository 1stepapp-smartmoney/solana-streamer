// ShredStream 相关模块
pub mod connection;
pub mod types;
pub mod stream_handler;
pub mod event_processor;

// 重新导出主要类型
pub use connection::*;
pub use types::*;
pub use stream_handler::*;
pub use event_processor::*;

// 从公用模块重新导出
pub use crate::streaming::common::{
    BackpressureConfig, BackpressureStrategy, BatchConfig, ConnectionConfig, EventBatchProcessor,
    MetricsEventType, MetricsManager, PerformanceMetrics, StreamClientConfig,
};
