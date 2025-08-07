use std::{collections::HashMap, fmt, time::Duration};

use chrono::Local;
use futures::{channel::mpsc, sink::Sink, SinkExt, Stream, StreamExt};
use log::{error, info};
use yellowstone_grpc_proto::prost_types::Timestamp;
use rustls::crypto::{ring::default_provider, CryptoProvider};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding};
use tonic::{transport::channel::ClientTlsConfig, Status};
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions, SubscribeRequestPing, SubscribeUpdate,
    SubscribeUpdateTransaction,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::common::AnyResult;
use crate::streaming::event_parser::{EventParserFactory, Protocol, UnifiedEvent};

type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;

// 默认配置常量
const DEFAULT_CONNECT_TIMEOUT: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT: u64 = 60;
const DEFAULT_CHANNEL_SIZE: usize = 1000;
const DEFAULT_MAX_DECODING_MESSAGE_SIZE: usize = 1024 * 1024 * 10;
const DEFAULT_BATCH_SIZE: usize = 100;
const DEFAULT_BATCH_TIMEOUT_MS: u64 = 5;

// 背压处理策略
#[derive(Debug, Clone, Copy)]
pub enum BackpressureStrategy {
    /// 阻塞等待（默认）
    Block,
    /// 丢弃消息
    Drop,
    /// 重试有限次数后丢弃
    Retry { max_attempts: usize, wait_ms: u64 },
    /// 有序处理（确保按 slot 顺序处理）
    Ordered { max_pending_slots: usize },
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
    /// 批处理超时时间（毫秒，默认：10ms）
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
    /// 通道大小（默认：10000）
    pub channel_size: usize,
    /// 背压处理策略（默认：Block）
    pub strategy: BackpressureStrategy,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            channel_size: DEFAULT_CHANNEL_SIZE,
            strategy: BackpressureStrategy::default(),
        }
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

/// 完整的客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 连接配置
    pub connection: ConnectionConfig,
    /// 批处理配置
    pub batch: BatchConfig,
    /// 背压配置
    pub backpressure: BackpressureConfig,
    /// 是否启用性能监控（默认：false）
    pub enable_metrics: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig::default(),
            backpressure: BackpressureConfig::default(),
            enable_metrics: false,
        }
    }
}

impl ClientConfig {
    /// 创建高性能配置（适合高并发场景）
    pub fn high_performance() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig {
                batch_size: 200,
                batch_timeout_ms: 5,
                enabled: true,
            },
            backpressure: BackpressureConfig {
                channel_size: 20000,
                strategy: BackpressureStrategy::Drop,
            },
            enable_metrics: true,
        }
    }

    /// 创建低延迟配置（适合实时场景）
    pub fn low_latency() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig {
                batch_size: 10,
                batch_timeout_ms: 1,
                enabled: false,
            },
            backpressure: BackpressureConfig {
                channel_size: 1000,
                strategy: BackpressureStrategy::Block,
            },
            enable_metrics: false,
        }
    }

    /// 创建有序处理配置（确保事件按顺序处理）
    pub fn ordered_processing(max_pending_slots: usize) -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig {
                batch_size: 50,
                batch_timeout_ms: 5,
                enabled: true,
            },
            backpressure: BackpressureConfig {
                channel_size: 15000,
                strategy: BackpressureStrategy::Ordered { max_pending_slots },
            },
            enable_metrics: true,
        }
    }
}

/// 性能监控指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub events_processed: u64,
    pub events_per_second: f64,
    pub average_processing_time_ms: f64,
    pub min_processing_time_ms: f64,
    pub max_processing_time_ms: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub last_update_time: std::time::Instant,
    pub events_in_window: u64,
    pub window_start_time: std::time::Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            events_processed: 0,
            events_per_second: 0.0,
            average_processing_time_ms: 0.0,
            min_processing_time_ms: f64::MAX,
            max_processing_time_ms: 0.0,
            cache_hit_rate: 0.0,
            memory_usage_mb: 0.0,
            last_update_time: now,
            events_in_window: 0,
            window_start_time: now,
        }
    }
}

/// gRPC连接池 - 简化版本
pub struct GrpcConnectionPool {
    endpoint: String,
    x_token: Option<String>,
}

impl GrpcConnectionPool {
    pub fn new(endpoint: String, x_token: Option<String>) -> Self {
        Self {
            endpoint,
            x_token,
        }
    }

    pub async fn create_connection(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(DEFAULT_MAX_DECODING_MESSAGE_SIZE)
            .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT))
            .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT));

        Ok(builder.connect().await?)
    }
}

/// 批处理事件收集器
pub struct EventBatchCollector<F>
where
    F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
{
    pub(crate) callback: F,
    batch: Vec<Box<dyn UnifiedEvent>>,
    batch_size: usize,
    timeout_ms: u64,
    last_flush_time: std::time::Instant,
}

impl<F> EventBatchCollector<F>
where
    F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
{
    pub fn new(callback: F, batch_size: usize, timeout_ms: u64) -> Self {
        Self {
            callback,
            batch: Vec::with_capacity(batch_size),
            batch_size,
            timeout_ms,
            last_flush_time: std::time::Instant::now(),
        }
    }

    pub fn add_event(&mut self, event: Box<dyn UnifiedEvent>) {
        log::debug!("Adding event to batch: {} (type: {:?})", event.id(), event.event_type());
        self.batch.push(event);
        
        // 检查是否需要刷新批次
        if self.batch.len() >= self.batch_size || self.should_flush_by_timeout() {
            log::info!("Flushing batch: size={}, timeout={}", self.batch.len(), self.should_flush_by_timeout());
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if !self.batch.is_empty() {
            let events = std::mem::replace(&mut self.batch, Vec::with_capacity(self.batch_size));
            log::info!("Flushing {} events from batch processor", events.len());
            
            // 添加更详细的调试信息
            for (i, event) in events.iter().enumerate() {
                log::info!("Event {}: Type={:?}, ID={}", i, event.event_type(), event.id());
            }
            
            // 执行回调并捕获可能的错误
            log::info!("About to execute batch callback with {} events", events.len());
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (self.callback)(events);
            })) {
                Ok(_) => {
                    log::info!("Batch callback executed successfully");
                }
                Err(e) => {
                    log::error!("Batch callback panicked: {:?}", e);
                }
            }
            
            self.last_flush_time = std::time::Instant::now();
        } else {
            log::debug!("No events to flush");
        }
    }

    fn should_flush_by_timeout(&self) -> bool {
        self.last_flush_time.elapsed().as_millis() >= self.timeout_ms as u128
    }
}

#[derive(Clone)]
pub struct TransactionPretty {
    pub slot: u64,
    pub block_time: Option<Timestamp>,
    pub signature: Signature,
    pub is_vote: bool,
    pub tx: EncodedTransactionWithStatusMeta,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TxWrap<'a>(&'a EncodedTransactionWithStatusMeta);
        impl<'a> fmt::Debug for TxWrap<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let serialized = serde_json::to_string(self.0).expect("failed to serialize");
                fmt::Display::fmt(&serialized, f)
            }
        }

        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("tx", &TxWrap(&self.tx))
            .finish()
    }
}

impl From<(SubscribeUpdateTransaction, Option<Timestamp>)> for TransactionPretty {
    fn from(
        (SubscribeUpdateTransaction { transaction, slot }, block_time): (
            SubscribeUpdateTransaction,
            Option<Timestamp>,
        ),
    ) -> Self {
        let tx = transaction.expect("should be defined");
        Self {
            slot,
            block_time,
            signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
            is_vote: tx.is_vote,
            tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
                .expect("valid tx with meta")
                .encode(UiTransactionEncoding::Base64, Some(u8::MAX), true)
                .expect("failed to encode"),
        }
    }
}

#[derive(Clone)]
pub struct YellowstoneGrpc {
    endpoint: String,
    x_token: Option<String>,
    config: ClientConfig,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl YellowstoneGrpc {
    /// 创建客户端，使用默认配置
    pub fn new(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, ClientConfig::default())
    }

    /// 创建客户端，使用自定义配置
    pub fn new_with_config(endpoint: String, x_token: Option<String>, config: ClientConfig) -> AnyResult<Self> {
        if CryptoProvider::get_default().is_none() {
            default_provider()
                .install_default()
                .map_err(|e| anyhow::anyhow!("Failed to install crypto provider: {:?}", e))?;
        }

        Ok(Self { 
            endpoint, 
            x_token,
            config,
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
        })
    }

    /// 创建高性能客户端（适合高并发场景）
    pub fn new_high_performance(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, ClientConfig::high_performance())
    }

    /// 创建低延迟客户端（适合实时场景）
    pub fn new_low_latency(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, ClientConfig::low_latency())
    }

    /// 创建有序处理客户端（确保事件按顺序处理）
    pub fn new_ordered_processing(endpoint: String, x_token: Option<String>, max_pending_slots: usize) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, ClientConfig::ordered_processing(max_pending_slots))
    }

    /// 创建简化的即时处理客户端（推荐用于简单场景）
    pub fn new_immediate(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        let mut config = ClientConfig::low_latency();
        config.enable_metrics = false; // 即时模式默认关闭性能监控
        Self::new_with_config(endpoint, x_token, config)
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ClientConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ClientConfig) {
        self.config = config;
    }

    /// 获取性能指标
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// 启用或禁用性能监控
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }



    /// 打印性能指标
    pub async fn print_metrics(&self) {
        let metrics = self.get_metrics().await;
        println!("📊 Performance Metrics:");
        println!("   Events Processed: {}", metrics.events_processed);
        println!("   Events/Second: {:.2}", metrics.events_per_second);
        println!("   Avg Processing Time: {:.2}ms", metrics.average_processing_time_ms);
        println!("   Min Processing Time: {:.2}ms", metrics.min_processing_time_ms);
        println!("   Max Processing Time: {:.2}ms", metrics.max_processing_time_ms);
        println!("   Cache Hit Rate: {:.2}%", metrics.cache_hit_rate * 100.0);
        println!("   Memory Usage: {:.2}MB", metrics.memory_usage_mb);
        println!("---");
    }

    /// 启动自动性能监控任务
    pub async fn start_auto_metrics_monitoring(&self) {
        // 检查是否启用性能监控
        if !self.config.enable_metrics {
            return; // 如果未启用性能监控，不启动监控任务
        }

        let grpc_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                grpc_clone.print_metrics().await;
            }
        });
    }

    /// 更新性能指标
    async fn update_metrics(&self, events_processed: u64, processing_time_ms: f64) {
        // 检查是否启用性能监控
        if !self.config.enable_metrics {
            return; // 如果未启用性能监控，直接返回
        }

        let mut metrics = self.metrics.lock().await;
        let now = std::time::Instant::now();
        
        metrics.events_processed += events_processed;
        metrics.events_in_window += events_processed;
        metrics.last_update_time = now;
        
        // 更新最快和最慢处理时间
        if processing_time_ms < metrics.min_processing_time_ms {
            metrics.min_processing_time_ms = processing_time_ms;
        }
        if processing_time_ms > metrics.max_processing_time_ms {
            metrics.max_processing_time_ms = processing_time_ms;
        }
        
        // 计算平均处理时间
        if metrics.events_processed > 0 {
            metrics.average_processing_time_ms = 
                (metrics.average_processing_time_ms * (metrics.events_processed - events_processed) as f64 + processing_time_ms) 
                / metrics.events_processed as f64;
        }
        
        // 基于时间窗口计算每秒处理事件数（5秒窗口）
        let window_duration = std::time::Duration::from_secs(5);
        if now.duration_since(metrics.window_start_time) >= window_duration {
            let window_seconds = now.duration_since(metrics.window_start_time).as_secs_f64();
            if window_seconds > 0.0 && metrics.events_in_window > 0 {
                metrics.events_per_second = metrics.events_in_window as f64 / window_seconds;
            } else {
                // 如果窗口内没有事件，保持之前的速率或设为0
                metrics.events_per_second = 0.0;
            }
            
            // 重置窗口
            metrics.events_in_window = 0;
            metrics.window_start_time = now;
        } else {
            // 如果窗口还没满，不更新 events_per_second，保持之前的计算值
            // 这样可以避免因为单次批处理时间波动导致的指标跳跃
        }
        
        // 估算内存使用（基于处理的事件数量）
        metrics.memory_usage_mb = metrics.events_processed as f64 * 0.001; // 每个事件约1KB
    }

    pub async fn connect(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(self.config.connection.max_decoding_message_size)
            .connect_timeout(Duration::from_secs(self.config.connection.connect_timeout))
            .timeout(Duration::from_secs(self.config.connection.request_timeout));

        Ok(builder.connect().await?)
    }

    pub async fn subscribe_with_request(
        &self,
        transactions: TransactionsFilterMap,
        commitment: Option<CommitmentLevel>,
    ) -> AnyResult<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        let subscribe_request = SubscribeRequest {
            transactions,
            commitment: if let Some(commitment) = commitment {
                Some(commitment as i32)
            } else {
                Some(CommitmentLevel::Processed.into())
            },
            ..Default::default()
        };

        let mut client = self.connect().await?;
        let (sink, stream) = client
            .subscribe_with_request(Some(subscribe_request))
            .await?;
        Ok((sink, stream))
    }

    pub fn get_subscribe_request_filter(
        &self,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
    ) -> TransactionsFilterMap {
        let mut transactions = HashMap::new();
        transactions.insert(
            "client".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include,
                account_exclude,
                account_required,
            },
        );
        transactions
    }

    pub async fn handle_stream_message(
        msg: SubscribeUpdate,
        tx: &mut mpsc::Sender<TransactionPretty>,
        subscribe_tx: &mut (impl Sink<SubscribeRequest, Error = mpsc::SendError> + Unpin),
        backpressure_strategy: BackpressureStrategy,
    ) -> AnyResult<()> {
        let created_at = msg.created_at;
        match msg.update_oneof {
            Some(UpdateOneof::Transaction(sut)) => {
                let transaction_pretty = TransactionPretty::from((sut, created_at));
                log::info!("Received transaction: {} at slot {}", transaction_pretty.signature, transaction_pretty.slot);
                
                // 根据背压策略处理发送
                match backpressure_strategy {
                    BackpressureStrategy::Block => {
                        // 阻塞等待，直到有空间
                        if let Err(e) = tx.send(transaction_pretty).await {
                            log::error!("Failed to send transaction to channel: {:?}", e);
                            return Err(anyhow::anyhow!("Channel send failed: {:?}", e));
                        }
                    }
                    BackpressureStrategy::Drop => {
                        // 尝试发送，如果失败则丢弃
                        if let Err(e) = tx.try_send(transaction_pretty) {
                            if e.is_full() {
                                log::warn!("Channel is full, dropping transaction");
                            } else {
                                log::error!("Channel is closed: {:?}", e);
                                return Err(anyhow::anyhow!("Channel is closed: {:?}", e));
                            }
                        }
                    }
                    BackpressureStrategy::Retry { max_attempts, wait_ms } => {
                        // 重试有限次数
                        let mut retry_count = 0;
                        loop {
                            match tx.try_send(transaction_pretty.clone()) {
                                Ok(_) => break,
                                Err(e) => {
                                    if e.is_full() {
                                        retry_count += 1;
                                        if retry_count >= max_attempts {
                                            log::warn!("Channel is full after {} attempts, dropping transaction", retry_count);
                                            break;
                                        }
                                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
                                    } else {
                                        log::error!("Channel is closed: {:?}", e);
                                        return Err(anyhow::anyhow!("Channel is closed: {:?}", e));
                                    }
                                }
                            }
                        }
                    }
                    BackpressureStrategy::Ordered { max_pending_slots: _ } => {
                        // 有序处理策略 - 这里暂时使用阻塞策略，实际的有序处理在接收端实现
                        if let Err(e) = tx.send(transaction_pretty).await {
                            log::error!("Failed to send transaction to channel: {:?}", e);
                            return Err(anyhow::anyhow!("Channel send failed: {:?}", e));
                        }
                    }
                }
            }
            Some(UpdateOneof::Ping(_)) => {
                subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id: 1 }),
                        ..Default::default()
                    })
                    .await?;
                info!("service is ping: {}", Local::now());
            }
            Some(UpdateOneof::Pong(_)) => {
                info!("service is pong: {}", Local::now());
            }
            _ => {
                log::debug!("Received other message type");
            }
        }
        Ok(())
    }

    /// Subscribe to Yellowstone GRPC service events with advanced filtering options
    ///
    /// This method allows subscribing to specific protocol events with more granular account filtering.
    /// It processes transactions in real-time and calls the provided callback function when matching events are found.
    ///
    /// # Parameters
    ///
    /// * `protocols` - List of protocols to parse (e.g., PumpFun, PumpSwap, Bonk, RaydiumCpmm)
    /// * `bot_wallet` - Optional bot wallet address. If passed: in PumpFunTradeEvent if user is in the address, is_bot=true will be set. In BonkTradeEvent if payer is in the address, is_bot=true will be set. Default is false.
    /// * `account_include` - List of account addresses to include in the subscription
    /// * `account_exclude` - List of account addresses to exclude from the subscription
    /// * `account_required` - List of account addresses that must be present in transactions
    /// * `commitment` - Optional commitment level for the subscription
    /// * `callback` - Function to call when matching events are found
    #[allow(clippy::too_many_arguments)]
    pub async fn subscribe_events_v2<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            self.start_auto_metrics_monitoring().await;
        }
        
        // 默认使用即时处理模式
        self.subscribe_events_immediate(
            protocols,
            bot_wallet,
            account_include,
            account_exclude,
            account_required,
            commitment,
            callback,
        )
        .await
    }

    /// 简化的即时事件订阅（推荐用于简单场景）
    pub async fn subscribe_events_immediate<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            self.start_auto_metrics_monitoring().await;
        }
        
        if account_include.is_empty() && account_exclude.is_empty() && account_required.is_empty() {
            return Err(anyhow::anyhow!(
                "account_include or account_exclude or account_required cannot be empty"
            ));
        }

        let transactions =
            self.get_subscribe_request_filter(account_include, account_exclude, account_required);
        
        // 订阅事件
        let (mut subscribe_tx, mut stream) = self
            .subscribe_with_request(transactions, commitment)
            .await?;

        // 创建通道，使用配置中的通道大小
        let (mut tx, mut rx) = mpsc::channel::<TransactionPretty>(self.config.backpressure.channel_size);

        // 启动流处理任务
        let backpressure_strategy = self.config.backpressure.strategy;
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::handle_stream_message(msg, &mut tx, &mut subscribe_tx, backpressure_strategy).await
                        {
                            error!("Error handling message: {e:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // 即时处理交易，无批处理
        let self_clone = self.clone();
        tokio::spawn(async move {
            while let Some(transaction_pretty) = rx.next().await {            
                if let Err(e) = self_clone.process_event_transaction_with_metrics(
                    transaction_pretty,
                    &callback,
                    bot_wallet,
                    protocols.clone(),
                )
                .await
                {
                    error!("Error processing transaction: {e:?}");
                }
            }
        });

        //tokio::signal::ctrl_c().await?;
        Ok(())
    }

    /// 高级模式订阅（包含批处理和背压处理）
    pub async fn subscribe_events_advanced<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            self.start_auto_metrics_monitoring().await;
        }
        
        if account_include.is_empty() && account_exclude.is_empty() && account_required.is_empty() {
            return Err(anyhow::anyhow!(
                "account_include or account_exclude or account_required cannot be empty"
            ));
        }

        let transactions =
            self.get_subscribe_request_filter(account_include, account_exclude, account_required);
        // Subscribe to events
        let (mut subscribe_tx, mut stream) = self
            .subscribe_with_request(transactions, commitment)
            .await?;

        // Create channel
        let (mut tx, mut rx) = mpsc::channel::<TransactionPretty>(self.config.backpressure.channel_size);

        // 创建批处理器，将单个事件回调转换为批量回调
        let batch_callback = move |events: Vec<Box<dyn UnifiedEvent>>| {
            for event in events {
                callback(event);
            }
        };
        
        let mut batch_processor = EventBatchCollector::new(
            batch_callback, 
            self.config.batch.batch_size, 
            self.config.batch.batch_timeout_ms
        );

        // Start task to process the stream
        let backpressure_strategy = self.config.backpressure.strategy;
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::handle_stream_message(msg, &mut tx, &mut subscribe_tx, backpressure_strategy).await
                        {
                            error!("Error handling message: {e:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // Process transactions with batch processing
        let self_clone = self.clone();
        
        // 根据背压策略选择处理方式
        match self.config.backpressure.strategy {
            BackpressureStrategy::Ordered { max_pending_slots: _ } => {
                // 使用有序处理 - 暂时使用普通的批处理方式
                tokio::spawn(async move {
                    while let Some(transaction_pretty) = rx.next().await {            
                        if let Err(e) = self_clone.process_event_transaction_with_batch(
                            transaction_pretty,
                            &mut batch_processor,
                            bot_wallet,
                            protocols.clone(),
                        )
                        .await
                        {
                            error!("Error processing transaction: {e:?}");
                        }
                    }
                    
                    // 处理剩余的事件
                    batch_processor.flush();
                });
            }
            _ => {
                // 使用原有的批处理方式
                tokio::spawn(async move {
                    while let Some(transaction_pretty) = rx.next().await {            
                        if let Err(e) = self_clone.process_event_transaction_with_batch(
                            transaction_pretty,
                            &mut batch_processor,
                            bot_wallet,
                            protocols.clone(),
                        )
                        .await
                        {
                            error!("Error processing transaction: {e:?}");
                        }
                    }
                    
                    // 处理剩余的事件
                    batch_processor.flush();
                });
            }
        }

        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    /// 订阅事件
    #[deprecated(
        since = "0.1.5",
        note = "This method will be removed, please use the new API: subscribe_events_v2"
    )]
    #[allow(clippy::too_many_arguments)]
    pub async fn subscribe_events<F>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        account_include: Option<Vec<String>>,
        account_exclude: Option<Vec<String>>,
        account_required: Option<Vec<String>>,
        commitment: Option<CommitmentLevel>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            self.start_auto_metrics_monitoring().await;
        }
        
        // 创建过滤器
        let protocol_accounts = protocols
            .iter()
            .flat_map(|p| p.get_program_id())
            .map(|p| p.to_string())
            .collect::<Vec<String>>();
        let mut account_include = account_include.unwrap_or_default();
        let account_exclude = account_exclude.unwrap_or_default();
        let account_required = account_required.unwrap_or_default();

        account_include.extend(protocol_accounts.clone());

        let transactions =
            self.get_subscribe_request_filter(account_include, account_exclude, account_required);

        // 订阅事件
        let (mut subscribe_tx, mut stream) = self
            .subscribe_with_request(transactions, commitment)
            .await?;

        // 创建通道
        let (mut tx, mut rx) = mpsc::channel::<TransactionPretty>(self.config.backpressure.channel_size);

        // 创建回调函数，使用 Arc 包装以便在多个任务中共享
        let callback = std::sync::Arc::new(Box::new(callback));

        // 启动处理流的任务
        let backpressure_strategy = self.config.backpressure.strategy;
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) =
                            Self::handle_stream_message(msg, &mut tx, &mut subscribe_tx, backpressure_strategy).await
                        {
                            error!("Error handling message: {e:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // 处理交易
        let self_clone = self.clone();
        tokio::spawn(async move {
            while let Some(transaction_pretty) = rx.next().await {
                if let Err(e) = self_clone.process_event_transaction_with_metrics(
                    transaction_pretty,
                    &**callback,
                    bot_wallet,
                    protocols.clone(),
                )
                .await
                {
                    error!("Error processing transaction: {e:?}");
                }
            }
        });

        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    async fn process_event_transaction_with_metrics<F>(
        &self,
        transaction_pretty: TransactionPretty,
        callback: &F,
        bot_wallet: Option<Pubkey>,
        protocols: Vec<Protocol>,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_pretty.slot;
        let signature = transaction_pretty.signature.to_string();
        
        // 预分配向量容量，避免动态扩容
        let mut futures = Vec::with_capacity(protocols.len());
        
        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol);
            // 在异步任务中需要克隆值
            let tx_clone = transaction_pretty.tx.clone();
            let signature_clone = signature.clone();
            let bot_wallet_clone = bot_wallet;

            futures.push(tokio::spawn(async move {
                parser
                    .parse_transaction(
                        tx_clone,
                        &signature_clone,
                        Some(slot),
                        transaction_pretty.block_time.map(|ts| prost_types::Timestamp {
                            seconds: ts.seconds,
                            nanos: ts.nanos,
                        }),
                        program_received_time_ms,
                        bot_wallet_clone,
                    )
                    .await
                    .unwrap_or_else(|_e| vec![])
            }));
        }

        let results = futures::future::join_all(futures).await;
        
        // 收集所有事件
        let mut all_events = Vec::new();
        for events in results.into_iter().flatten() {
            all_events.extend(events);
        }
        
        // 保存事件数量用于日志记录
        let event_count = all_events.len();
        
        // 批量处理事件
        if !all_events.is_empty() {
            for event in all_events {
                callback(event);
            }
        }
        
        // 更新性能指标
        let processing_time = start_time.elapsed();
        let processing_time_ms = processing_time.as_millis() as f64;
        
        // 更新性能指标（如果启用）
        if self.config.enable_metrics {
            self.update_metrics(event_count as u64, processing_time_ms).await;
        }
        
        // 记录慢处理操作
        if processing_time_ms > 10.0 {
            log::warn!("Slow event processing: {processing_time_ms}ms for {event_count} events");
        }
        
        Ok(())
    }

    async fn process_event_transaction_with_batch<F>(
        &self,
        transaction_pretty: TransactionPretty,
        batch_processor: &mut EventBatchCollector<F>,
        bot_wallet: Option<Pubkey>,
        protocols: Vec<Protocol>,
    ) -> AnyResult<()>
    where
        F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        let start_time = std::time::Instant::now();
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_pretty.slot;
        let signature = transaction_pretty.signature.to_string();
        
        // 预分配向量容量，避免动态扩容
        let mut futures: Vec<tokio::task::JoinHandle<Result<Vec<Box<dyn UnifiedEvent>>, anyhow::Error>>> = Vec::with_capacity(protocols.len());
        
        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol.clone());
            // 在异步任务中需要克隆值
            let tx_clone = transaction_pretty.tx.clone();
            let signature_clone = signature.clone();
            let bot_wallet_clone = bot_wallet;
            let protocol_clone = protocol.clone();

            futures.push(tokio::spawn(async move {
                let result = parser
                    .parse_transaction(
                        tx_clone,
                        &signature_clone,
                        Some(slot),
                        transaction_pretty.block_time.map(|ts| prost_types::Timestamp {
                            seconds: ts.seconds,
                            nanos: ts.nanos,
                        }),
                        program_received_time_ms,
                        bot_wallet_clone,
                    )
                    .await;
                
                match result {
                    Ok(events) => {
                        if !events.is_empty() {
                            log::info!("Parsed {} events for protocol {:?}", events.len(), protocol_clone);
                        }
                        Ok(events)
                    }
                    Err(e) => {
                        log::warn!("Failed to parse transaction for protocol {:?}: {:?}", protocol_clone, e);
                        Ok(vec![])
                    }
                }
            }));
        }

        let results = futures::future::join_all(futures).await;
        
        // 收集所有事件并使用批处理器
        let mut total_events = 0;
        for result in results {
            match result {
                Ok(parse_result) => {
                    match parse_result {
                        Ok(events) => {
                            total_events += events.len();
                            log::info!("Adding {} events to batch processor", events.len());
                            for event in events {
                                if self.config.batch.enabled {
                                    batch_processor.add_event(event);
                                } else {
                                    // 如果批处理被禁用，直接调用回调
                                    // 这里需要将单个事件包装成Vec来调用批处理回调
                                    let single_event_batch = vec![event];
                                    (batch_processor.callback)(single_event_batch);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to parse transaction: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get events from async task: {:?}", e);
                }
            }
        }
        
        // 添加调试信息
        if total_events > 0 {
            log::info!("Total events parsed: {} for transaction {}", total_events, signature);
        }
        
        // 更新性能指标
        let processing_time = start_time.elapsed();
        let processing_time_ms = processing_time.as_millis() as f64;
        
        // 实际调用性能指标更新
        self.update_metrics(total_events as u64, processing_time_ms).await;
        
        // 记录慢处理操作
        if processing_time_ms > 10.0 {
            log::warn!("Slow event processing: {processing_time_ms}ms for {total_events} events");
        }
        
        Ok(())
    }
}



