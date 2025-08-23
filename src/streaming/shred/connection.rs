use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::common::AnyResult;
use crate::protos::shredstream::shredstream_proxy_client::ShredstreamProxyClient;
use crate::streaming::common::{
    MetricsManager, PerformanceMetrics, StreamClientConfig, SubscriptionHandle,
};

/// ShredStream gRPC 客户端
#[derive(Clone)]
pub struct ShredStreamGrpc {
    pub shredstream_client: Arc<ShredstreamProxyClient<Channel>>,
    pub config: StreamClientConfig,
    pub metrics: Arc<Mutex<PerformanceMetrics>>,
    pub metrics_manager: MetricsManager,
    pub subscription_handle: Arc<Mutex<Option<SubscriptionHandle>>>,
}

impl ShredStreamGrpc {
    /// 创建客户端，使用默认配置
    pub async fn new(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, StreamClientConfig::default()).await
    }

    /// 创建客户端，使用自定义配置
    pub async fn new_with_config(endpoint: String, config: StreamClientConfig) -> AnyResult<Self> {
        let shredstream_client = ShredstreamProxyClient::connect(endpoint.clone()).await?;
        let metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
        let config_arc = Arc::new(config.clone());

        let metrics_manager =
            MetricsManager::new(metrics.clone(), config_arc, "ShredStream".to_string());

        Ok(Self {
            shredstream_client: Arc::new(shredstream_client),
            config,
            metrics,
            metrics_manager,
            subscription_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// 创建高性能客户端（适合高并发场景）
    pub async fn new_high_performance(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, StreamClientConfig::high_performance()).await
    }

    /// 创建低延迟客户端（适合实时场景）
    pub async fn new_low_latency(endpoint: String) -> AnyResult<Self> {
        Self::new_with_config(endpoint, StreamClientConfig::low_latency()).await
    }

    /// 获取当前配置
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

    /// 启用或禁用性能监控
    pub fn set_enable_metrics(&mut self, enabled: bool) {
        self.config.enable_metrics = enabled;
    }

    /// 打印性能指标
    pub async fn print_metrics(&self) {
        self.metrics_manager.print_metrics().await;
    }

    /// 启动自动性能监控任务
    pub async fn start_auto_metrics_monitoring(&self) {
        self.metrics_manager.start_auto_monitoring().await;
    }

    /// 停止当前订阅
    pub async fn stop(&self) {
        let mut handle_guard = self.subscription_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop();
        }
    }
}
