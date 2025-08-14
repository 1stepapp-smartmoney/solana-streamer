use super::constants::*;

/// 背压处理策略
#[derive(Debug, Clone, Copy)]
pub enum BackpressureStrategy {
    /// 阻塞等待（默认）
    Block,
    /// 丢弃消息
    Drop,
    /// 重试有限次数后丢弃
    Retry { max_attempts: usize, wait_ms: u64 },
}

impl Default for BackpressureStrategy {
    fn default() -> Self {
        Self::Block
    }
}

/// 批处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批处理大小（默认：100）
    pub batch_size: usize,
    /// 批处理超时时间（毫秒，默认：5ms）
    pub batch_timeout_ms: u64,
    /// 是否启用批处理（默认：true）
    pub enabled: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            batch_timeout_ms: DEFAULT_BATCH_TIMEOUT_MS,
            enabled: true,
        }
    }
}

/// 背压配置
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// 通道大小（默认：1000）
    pub channel_size: usize,
    /// 背压处理策略（默认：Block）
    pub strategy: BackpressureStrategy,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self { channel_size: DEFAULT_CHANNEL_SIZE, strategy: BackpressureStrategy::default() }
    }
}

/// 连接配置
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// 连接超时时间（秒，默认：10）
    pub connect_timeout: u64,
    /// 请求超时时间（秒，默认：60）
    pub request_timeout: u64,
    /// 最大解码消息大小（字节，默认：10MB）
    pub max_decoding_message_size: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_decoding_message_size: DEFAULT_MAX_DECODING_MESSAGE_SIZE,
        }
    }
}

/// 通用客户端配置
#[derive(Debug, Clone)]
pub struct StreamClientConfig {
    /// 连接配置
    pub connection: ConnectionConfig,
    /// 批处理配置
    pub batch: BatchConfig,
    /// 背压配置
    pub backpressure: BackpressureConfig,
    /// 是否启用性能监控（默认：false）
    pub enable_metrics: bool,
}

impl Default for StreamClientConfig {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig::default(),
            backpressure: BackpressureConfig::default(),
            enable_metrics: false,
        }
    }
}

impl StreamClientConfig {
    /// 创建高性能配置（适合高并发场景）
    pub fn high_performance() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig { batch_size: 200, batch_timeout_ms: 5, enabled: true },
            backpressure: BackpressureConfig {
                channel_size: 20000,
                strategy: BackpressureStrategy::Drop,
            },
            enable_metrics: false,
        }
    }

    /// 创建低延迟配置（适合实时场景）
    pub fn low_latency() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig {
                batch_size: 10,
                batch_timeout_ms: 1,
                enabled: false, // 禁用批处理，即时处理
            },
            backpressure: BackpressureConfig {
                channel_size: 1000,
                strategy: BackpressureStrategy::Block,
            },
            enable_metrics: false,
        }
    }
}
