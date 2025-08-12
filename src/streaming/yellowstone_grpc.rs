use futures::{channel::mpsc, StreamExt};
use log::error;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::Mutex;
use yellowstone_grpc_proto::geyser::CommitmentLevel;

use crate::common::AnyResult;
use crate::streaming::common::{
    EventBatchProcessor, MetricsManager, PerformanceMetrics, StreamClientConfig,
};
use crate::streaming::event_parser::{Protocol, UnifiedEvent};
use crate::streaming::grpc::{EventPretty, EventProcessor, StreamHandler, SubscriptionManager};

#[derive(Clone)]
pub struct YellowstoneGrpc {
    pub endpoint: String,
    pub x_token: Option<String>,
    pub config: StreamClientConfig,
    pub metrics: Arc<Mutex<PerformanceMetrics>>,
    pub subscription_manager: SubscriptionManager,
    pub metrics_manager: MetricsManager,
    pub event_processor: EventProcessor,
}

impl YellowstoneGrpc {
    /// 创建客户端，使用默认配置
    pub fn new(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::default())
    }

    /// 创建客户端，使用自定义配置
    pub fn new_with_config(
        endpoint: String,
        x_token: Option<String>,
        config: StreamClientConfig,
    ) -> AnyResult<Self> {
        let _ = rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| anyhow::anyhow!("Failed to install rustls crypto provider"))?;
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
        let config_arc = Arc::new(config.clone());

        let subscription_manager =
            SubscriptionManager::new(endpoint.clone(), x_token.clone(), config.clone());
        let metrics_manager =
            MetricsManager::new(metrics.clone(), config_arc.clone(), "YellowstoneGrpc".to_string());
        let event_processor = EventProcessor::new(metrics_manager.clone(), config.clone());

        Ok(Self {
            endpoint,
            x_token,
            config,
            metrics,
            subscription_manager,
            metrics_manager,
            event_processor,
        })
    }

    /// 创建高性能客户端
    pub fn new_high_performance(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::high_performance())
    }

    /// 创建低延迟客户端
    pub fn new_low_latency(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        Self::new_with_config(endpoint, x_token, StreamClientConfig::low_latency())
    }

    /// 创建即时处理客户端
    pub fn new_immediate(endpoint: String, x_token: Option<String>) -> AnyResult<Self> {
        let mut config = StreamClientConfig::low_latency();
        config.enable_metrics = false;
        Self::new_with_config(endpoint, x_token, config)
    }

    /// 获取配置
    pub fn get_config(&self) -> &StreamClientConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: StreamClientConfig) {
        self.config = config;
    }

    /// 获取性能指标
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics_manager.get_metrics().await
    }

    /// 打印性能指标
    pub async fn print_metrics(&self) {
        self.metrics_manager.print_metrics().await;
    }

    /// 启用或禁用性能监控
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
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
            self.metrics_manager.start_auto_monitoring().await;
        }

        // 验证订阅参数
        self.subscription_manager.validate_subscription_params(
            &account_include,
            &account_exclude,
            &account_required,
        )?;

        let transactions = self.subscription_manager.get_subscribe_request_filter(
            account_include,
            account_exclude,
            account_required,
        );

        // 订阅事件
        let (mut subscribe_tx, mut stream) =
            self.subscription_manager.subscribe_with_request(transactions, commitment).await?;

        // 创建通道，使用配置中的通道大小
        let (mut tx, mut rx) = mpsc::channel::<EventPretty>(self.config.backpressure.channel_size);

        // 启动流处理任务
        let backpressure_strategy = self.config.backpressure.strategy;
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) = StreamHandler::handle_stream_message(
                            msg,
                            &mut tx,
                            &mut subscribe_tx,
                            backpressure_strategy,
                        )
                        .await
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
        let event_processor = self.event_processor.clone();
        tokio::spawn(async move {
            while let Some(event_pretty) = rx.next().await {
                if let Err(e) = event_processor
                    .process_event_transaction_with_metrics(
                        event_pretty,
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
            self.metrics_manager.start_auto_monitoring().await;
        }

        // 验证订阅参数
        self.subscription_manager.validate_subscription_params(
            &account_include,
            &account_exclude,
            &account_required,
        )?;

        let transactions = self.subscription_manager.get_subscribe_request_filter(
            account_include,
            account_exclude,
            account_required,
        );

        // Subscribe to events
        let (mut subscribe_tx, mut stream) =
            self.subscription_manager.subscribe_with_request(transactions, commitment).await?;

        // Create channel
        let (mut tx, mut rx) = mpsc::channel::<EventPretty>(self.config.backpressure.channel_size);

        // 创建批处理器，将单个事件回调转换为批量回调
        let batch_callback = move |events: Vec<Box<dyn UnifiedEvent>>| {
            for event in events {
                callback(event);
            }
        };

        let mut batch_processor = EventBatchProcessor::new(
            batch_callback,
            self.config.batch.batch_size,
            self.config.batch.batch_timeout_ms,
        );

        // Start task to process the stream
        let backpressure_strategy = self.config.backpressure.strategy;
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(e) = StreamHandler::handle_stream_message(
                            msg,
                            &mut tx,
                            &mut subscribe_tx,
                            backpressure_strategy,
                        )
                        .await
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
        let event_processor = self.event_processor.clone();
        tokio::spawn(async move {
            while let Some(event_pretty) = rx.next().await {
                if let Err(e) = event_processor
                    .process_event_transaction_with_batch(
                        event_pretty,
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

        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    /// 默认订阅方法 - 委托给即时处理模式
    #[deprecated(since = "0.1.12", note = "Use subscribe_events_immediate instead")]
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
}

// 实现 Clone trait 以支持模块间共享
impl Clone for EventProcessor {
    fn clone(&self) -> Self {
        Self { metrics_manager: self.metrics_manager.clone(), config: self.config.clone() }
    }
}
