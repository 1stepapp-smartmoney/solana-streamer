use std::sync::Arc;
use tokio::sync::Mutex;

use super::config::StreamClientConfig;
use super::constants::*;

/// 通用性能监控指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub start_time: std::time::Instant,
    pub process_count: u64,
    pub events_processed: u64,
    pub events_per_second: f64,
    pub average_processing_time_ms: f64,
    pub min_processing_time_ms: f64,
    pub max_processing_time_ms: f64,
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
            start_time: std::time::Instant::now(),
            process_count: 0,
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

/// 通用性能监控管理器
pub struct MetricsManager {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    config: Arc<StreamClientConfig>,
    stream_name: String,
}

impl MetricsManager {
    /// 创建新的性能监控管理器
    pub fn new(
        metrics: Arc<Mutex<PerformanceMetrics>>,
        config: Arc<StreamClientConfig>,
        stream_name: String,
    ) -> Self {
        Self { metrics, config, stream_name }
    }

    /// 获取性能指标
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// 打印性能指标
    pub async fn print_metrics(&self) {
        let metrics = self.get_metrics().await;
        println!("📊 {} Performance Metrics:", self.stream_name);
        println!("   Run Time: {:?}", metrics.start_time.elapsed());
        println!("   Process Count: {}", metrics.process_count);
        println!("   Events Processed: {}", metrics.events_processed);
        println!("   Events/Second: {:.2}", metrics.events_per_second);
        println!("   Avg Processing Time: {:.2}ms", metrics.average_processing_time_ms);
        println!("   Min Processing Time: {:.2}ms", metrics.min_processing_time_ms);
        println!("   Max Processing Time: {:.2}ms", metrics.max_processing_time_ms);
        println!("---");
    }

    /// 启动自动性能监控任务
    pub async fn start_auto_monitoring(&self) {
        // 检查是否启用性能监控
        if !self.config.enable_metrics {
            return; // 如果未启用性能监控，不启动监控任务
        }

        let metrics_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                DEFAULT_METRICS_PRINT_INTERVAL_SECONDS,
            ));
            loop {
                interval.tick().await;
                metrics_manager.print_metrics().await;
            }
        });
    }

    /// 更新处理次数
    pub async fn add_process_count(&self) {
        if !self.config.enable_metrics {
            return;
        }
        let mut metrics = self.metrics.lock().await;
        metrics.process_count += 1;
    }

    /// 更新性能指标
    pub async fn update_metrics(&self, events_processed: u64, processing_time_ms: f64) {
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
        if processing_time_ms < metrics.min_processing_time_ms
            || metrics.min_processing_time_ms == 0.0
        {
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

        // 基于时间窗口计算每秒处理事件数
        let window_duration = std::time::Duration::from_secs(DEFAULT_METRICS_WINDOW_SECONDS);
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
        }
    }

    /// 记录慢处理操作
    pub fn log_slow_processing(&self, processing_time_ms: f64, event_count: usize) {
        if processing_time_ms > SLOW_PROCESSING_THRESHOLD_MS {
            log::warn!(
                "{} slow processing: {processing_time_ms}ms for {event_count} events",
                self.stream_name
            );
        }
    }
}

impl Clone for MetricsManager {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            config: self.config.clone(),
            stream_name: self.stream_name.clone(),
        }
    }
}
