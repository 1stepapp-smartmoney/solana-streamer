use std::sync::Arc;
use tokio::sync::Mutex;

use futures::{channel::mpsc, StreamExt};
use solana_entry::entry::Entry;
use tonic::transport::Channel;

use log::error;
use solana_sdk::transaction::VersionedTransaction;

use crate::common::AnyResult;
use crate::streaming::event_parser::{EventParserFactory, Protocol, UnifiedEvent};

use crate::protos::shredstream::shredstream_proxy_client::ShredstreamProxyClient;
use crate::protos::shredstream::SubscribeEntriesRequest;
use solana_sdk::pubkey::Pubkey;

// -------------- TODO 待重构 --------------
//
// -------------- --------------

// 默认配置常量
const DEFAULT_CHANNEL_SIZE: usize = 1000;
const DEFAULT_BATCH_SIZE: usize = 100;
const DEFAULT_BATCH_TIMEOUT_MS: u64 = 5;

/// ShredStream批处理配置
#[derive(Debug, Clone)]
pub struct ShredBatchConfig {
    /// 批处理大小（默认：100）
    pub batch_size: usize,
    /// 批处理超时时间（毫秒，默认：10ms）
    pub batch_timeout_ms: u64,
    /// 是否启用批处理（默认：true）
    pub enabled: bool,
}

impl Default for ShredBatchConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            batch_timeout_ms: DEFAULT_BATCH_TIMEOUT_MS,
            enabled: true,
        }
    }
}

/// ShredStream背压配置
#[derive(Debug, Clone)]
pub struct ShredBackpressureConfig {
    /// 通道大小（默认：10000）
    pub channel_size: usize,
}

impl Default for ShredBackpressureConfig {
    fn default() -> Self {
        Self { channel_size: DEFAULT_CHANNEL_SIZE }
    }
}

/// ShredStream完整配置
#[derive(Debug, Clone)]
pub struct ShredClientConfig {
    /// 批处理配置
    pub batch: ShredBatchConfig,
    /// 背压配置
    pub backpressure: ShredBackpressureConfig,
    /// 是否启用性能监控（默认：false）
    pub enable_metrics: bool,
}

impl Default for ShredClientConfig {
    fn default() -> Self {
        Self {
            batch: ShredBatchConfig::default(),
            backpressure: ShredBackpressureConfig::default(),
            enable_metrics: false,
        }
    }
}

impl ShredClientConfig {
    /// 创建高性能配置（适合高并发场景）
    pub fn high_performance() -> Self {
        Self {
            batch: ShredBatchConfig { batch_size: 200, batch_timeout_ms: 5, enabled: true },
            backpressure: ShredBackpressureConfig { channel_size: 20000 },
            enable_metrics: true,
        }
    }

    /// 创建低延迟配置（适合实时场景）
    pub fn low_latency() -> Self {
        Self {
            batch: ShredBatchConfig {
                batch_size: 10,
                batch_timeout_ms: 1,
                enabled: false, // 禁用批处理，即时处理
            },
            backpressure: ShredBackpressureConfig { channel_size: 1000 },
            enable_metrics: false,
        }
    }
}

/// ShredStream性能监控指标
#[derive(Debug, Clone)]
pub struct ShredPerformanceMetrics {
    pub events_processed: u64,
    pub events_per_second: f64,
    pub average_processing_time_ms: f64,
    pub min_processing_time_ms: f64,
    pub max_processing_time_ms: f64,
    pub last_update_time: std::time::Instant,
    pub events_in_window: u64,
    pub window_start_time: std::time::Instant,
}

impl Default for ShredPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ShredPerformanceMetrics {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            events_processed: 0,
            events_per_second: 0.0,
            average_processing_time_ms: 0.0,
            min_processing_time_ms: 0.0,
            max_processing_time_ms: 0.0,
            last_update_time: now,
            events_in_window: 0,
            window_start_time: now,
        }
    }
}

#[derive(Clone)]
pub struct ShredStreamGrpc {
    shredstream_client: Arc<ShredstreamProxyClient<Channel>>,
    config: ShredClientConfig,
    metrics: Arc<Mutex<ShredPerformanceMetrics>>,
}

struct TransactionWithSlot {
    transaction: VersionedTransaction,
    slot: u64,
}

/// ShredStream批处理器
pub struct ShredBatchProcessor<F>
where
    F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
{
    callback: F,
    batch: Vec<Box<dyn UnifiedEvent>>,
    batch_size: usize,
    timeout_ms: u64,
    last_flush_time: std::time::Instant,
}

impl<F> ShredBatchProcessor<F>
where
    F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
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
        self.batch.push(event);

        // 检查是否需要刷新批次
        if self.batch.len() >= self.batch_size || self.should_flush_by_timeout() {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if !self.batch.is_empty() {
            let events = std::mem::replace(&mut self.batch, Vec::with_capacity(self.batch_size));
            (self.callback)(events);
            self.last_flush_time = std::time::Instant::now();
        }
    }

    fn should_flush_by_timeout(&self) -> bool {
        self.last_flush_time.elapsed().as_millis() >= self.timeout_ms as u128
    }
}

impl ShredStreamGrpc {
    /// 创建客户端，使用默认配置
    pub async fn new(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, ShredClientConfig::default()).await
    }

    /// 创建客户端，使用自定义配置
    pub async fn new_with_config(endpoint: String, config: ShredClientConfig) -> AnyResult<Self> {
        let shredstream_client = ShredstreamProxyClient::connect(endpoint.clone()).await?;
        Ok(Self {
            shredstream_client: Arc::new(shredstream_client),
            config,
            metrics: Arc::new(Mutex::new(ShredPerformanceMetrics::new())),
        })
    }

    /// 创建高性能客户端（适合高并发场景）
    pub async fn new_high_performance(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, ShredClientConfig::high_performance()).await
    }

    /// 创建低延迟客户端（适合实时场景）
    pub async fn new_low_latency(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, ShredClientConfig::low_latency()).await
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &ShredClientConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ShredClientConfig) {
        self.config = config;
    }

    /// 获取性能指标
    pub async fn get_metrics(&self) -> ShredPerformanceMetrics {
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
        println!("📊 ShredStream Performance Metrics:");
        println!("   Events Processed: {}", metrics.events_processed);
        println!("   Events/Second: {:.2}", metrics.events_per_second);
        println!("   Avg Processing Time: {:.2}ms", metrics.average_processing_time_ms);
        println!("   Min Processing Time: {:.2}ms", metrics.min_processing_time_ms);
        println!("   Max Processing Time: {:.2}ms", metrics.max_processing_time_ms);
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
        if processing_time_ms < metrics.min_processing_time_ms || metrics.min_processing_time_ms == 0.0 {
            metrics.min_processing_time_ms = processing_time_ms;
        }
        if processing_time_ms > metrics.max_processing_time_ms {
            metrics.max_processing_time_ms = processing_time_ms;
        }

        // 计算平均处理时间
        if metrics.events_processed > 0 {
            metrics.average_processing_time_ms = (metrics.average_processing_time_ms
                * (metrics.events_processed - events_processed) as f64
                + processing_time_ms)
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

    }

    /// 订阅ShredStream事件（支持批处理和即时处理）
    pub async fn shredstream_subscribe<F,FP>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: F,
        tx_pack_callback: Option<FP>
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
        FP: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            self.start_auto_metrics_monitoring().await;
        }

        let request = tonic::Request::new(SubscribeEntriesRequest {});
        let mut client = (*self.shredstream_client).clone();
        let stream = client.subscribe_entries(request).await?.into_inner();
        let (tx, rx) = mpsc::channel::<TransactionWithSlot>(self.config.backpressure.channel_size);

        // 根据配置选择处理模式
        if self.config.batch.enabled {
            // 批处理模式
            self.process_with_batch(stream, tx, rx, protocols, bot_wallet, callback).await
        } else {
            self.process_immediate_in_tx_pack(stream, tx, rx, protocols, bot_wallet, callback, tx_pack_callback).await

        }
    }

    /// 批处理模式
    async fn process_with_batch<F>(
        &self,
        mut stream: tonic::codec::Streaming<crate::protos::shredstream::Entry>,
        mut tx: mpsc::Sender<TransactionWithSlot>,
        mut rx: mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        // 创建批处理器，将单个事件回调转换为批量回调
        let batch_callback = move |events: Vec<Box<dyn UnifiedEvent>>| {
            for event in events {
                callback(event);
            }
        };

        let mut batch_processor = ShredBatchProcessor::new(
            batch_callback,
            self.config.batch.batch_size,
            self.config.batch.batch_timeout_ms,
        );

        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
                            for entry in entries {
                                for transaction in entry.transactions {
                                    let _ = tx.try_send(TransactionWithSlot {
                                        transaction: transaction.clone(),
                                        slot: msg.slot,
                                    });
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        let self_clone = self.clone();
        while let Some(transaction_with_slot) = rx.next().await {
            if let Err(e) = self_clone
                .process_transaction_with_batch(
                    transaction_with_slot,
                    protocols.clone(),
                    bot_wallet,
                    &mut batch_processor,
                )
                .await
            {
                error!("Error processing transaction: {e:?}");
            }
        }

        // 处理剩余的事件
        batch_processor.flush();

        Ok(())
    }

    /// 即时处理模式
    async fn process_immediate<F>(
        &self,
        mut stream: tonic::codec::Streaming<crate::protos::shredstream::Entry>,
        mut tx: mpsc::Sender<TransactionWithSlot>,
        mut rx: mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
                            for entry in entries {
                                for transaction in entry.transactions {
                                    let _ = tx.try_send(TransactionWithSlot {
                                        transaction: transaction.clone(),
                                        slot: msg.slot,
                                    });
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        let self_clone = self.clone();
        while let Some(transaction_with_slot) = rx.next().await {
            if let Err(e) = self_clone
                .process_transaction_immediate(
                    transaction_with_slot,
                    protocols.clone(),
                    bot_wallet,
                    &callback,
                )
                .await
            {
                error!("Error processing transaction: {e:?}");
            }
        }

        Ok(())
    }

    async fn process_immediate_in_tx_pack<F, FP>(
        &self,
        mut stream: tonic::codec::Streaming<crate::protos::shredstream::Entry>,
        mut tx: mpsc::Sender<TransactionWithSlot>,
        mut rx: mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: F,
        tx_pack_callback: Option<FP>,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
        FP: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
                            for entry in entries {
                                for transaction in entry.transactions {
                                    let _ = tx.try_send(TransactionWithSlot {
                                        transaction: transaction.clone(),
                                        slot: msg.slot,
                                    });
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        let self_clone = self.clone();
        if let Some(pack_callback) = tx_pack_callback {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = self_clone.process_transaction_immediate_in_tx_pack(
                    transaction_with_slot,
                    protocols.clone(),
                    bot_wallet,
                    &pack_callback,
                )
                    .await
                {
                    error!("Error processing transaction: {e:?}");
                }
            }
        } else {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = self_clone.process_transaction_immediate(
                    transaction_with_slot,
                    protocols.clone(),
                    bot_wallet,
                    &callback,
                )
                    .await
                {
                    error!("Error processing transaction: {e:?}");
                }
            }
        }


        Ok(())
    }

    /// 即时处理单个交易
    async fn process_transaction_immediate<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: &F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        if versioned_tx.signatures.is_empty() {
            return Ok(()); // 如果没有签名，直接返回
        }
        let signature = versioned_tx.signatures[0];

        // 预分配向量容量
        let mut all_events = Vec::with_capacity(protocols.len() * 2);

        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol.clone());
            let events = parser
                .parse_versioned_transaction(
                    &versioned_tx,
                    &signature.to_string(),
                    Some(slot),
                    None,
                    program_received_time_ms,
                    bot_wallet,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
            all_events.extend(events);
        }

        // 保存事件数量用于日志记录
        let event_count = all_events.len();

        // 即时处理事件
        for event in all_events {
            callback(event);
        }

        // 更新性能指标
        let processing_time = start_time.elapsed();
        let processing_time_ms = processing_time.as_millis() as f64;

        // 实际调用性能指标更新
        self.update_metrics(event_count as u64, processing_time_ms).await;

        // 记录慢处理操作
        if processing_time_ms > 5.0 {
            log::warn!(
                "ShredStream transaction processing took {}ms for {} events",
                processing_time_ms,
                event_count
            );
        }

        Ok(())
    }

    /// 单个Tx的事件打包返回
    async fn process_transaction_immediate_in_tx_pack<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        callback: &F,
    ) -> AnyResult<()>
    where
        F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        if versioned_tx.signatures.is_empty() {
            return Ok(()); // 如果没有签名，直接返回
        }
        let signature = versioned_tx.signatures[0];

        // 预分配向量容量
        let mut all_events = Vec::with_capacity(protocols.len() * 2);

        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol.clone());
            let events = parser
                .parse_versioned_transaction(
                    &versioned_tx,
                    &signature.to_string(),
                    Some(slot),
                    None,
                    program_received_time_ms,
                    bot_wallet,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
            all_events.extend(events);
        }

        // 保存事件数量用于日志记录
        let event_count = all_events.len();

        // // 即时处理事件
        // for event in all_events {
        //     callback(event);
        // }

        callback(all_events);

        // 更新性能指标
        let processing_time = start_time.elapsed();
        let processing_time_ms = processing_time.as_millis() as f64;

        // 实际调用性能指标更新
        self.update_metrics(event_count as u64, processing_time_ms).await;

        // 记录慢处理操作
        if processing_time_ms > 5.0 {
            log::warn!("ShredStream transaction processing took {}ms for {} events",
                      processing_time_ms, event_count);
        }

        Ok(())
    }

    async fn process_transaction_with_batch<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        batch_processor: &mut ShredBatchProcessor<F>,
    ) -> AnyResult<()>
    where
        F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        let start_time = std::time::Instant::now();
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        let signature = versioned_tx.signatures[0];

        // 预分配向量容量
        let mut all_events = Vec::with_capacity(protocols.len() * 2);

        for protocol in protocols {
            let parser = EventParserFactory::create_parser(protocol.clone());
            let events = parser
                .parse_versioned_transaction(
                    &versioned_tx,
                    &signature.to_string(),
                    Some(slot),
                    None,
                    program_received_time_ms,
                    bot_wallet,
                )
                .await
                .unwrap_or_else(|_e| vec![]);
            all_events.extend(events);
        }

        // 保存事件数量用于日志记录
        let event_count = all_events.len();

        // 使用批处理器处理事件
        for event in all_events {
            batch_processor.add_event(event);
        }

        // 更新性能指标
        let processing_time = start_time.elapsed();
        let processing_time_ms = processing_time.as_millis() as f64;

        // 实际调用性能指标更新
        self.update_metrics(event_count as u64, processing_time_ms).await;

        // 记录慢处理操作
        if processing_time_ms > 5.0 {
            log::warn!(
                "ShredStream transaction processing took {}ms for {} events",
                processing_time_ms,
                event_count
            );
        }

        Ok(())
    }
}
