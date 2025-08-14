use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use super::types::EventPretty;
use crate::common::AnyResult;
use crate::streaming::common::{
    EventBatchProcessor as EventBatchCollector, MetricsManager, StreamClientConfig as ClientConfig,
};
use crate::streaming::event_parser::core::account_event_parser::AccountEventParser;
use crate::streaming::event_parser::core::common_event_parser::CommonEventParser;
use crate::streaming::event_parser::EventParser;
use crate::streaming::event_parser::{
    core::traits::UnifiedEvent, protocols::mutil::parser::MutilEventParser, Protocol,
};

/// 事件处理器
pub struct EventProcessor {
    pub(crate) metrics_manager: MetricsManager,
    pub(crate) config: ClientConfig,
}

impl EventProcessor {
    /// 创建新的事件处理器
    pub fn new(metrics_manager: MetricsManager, config: ClientConfig) -> Self {
        Self { metrics_manager, config }
    }

    /// 使用性能监控处理事件交易
    pub async fn process_event_transaction_with_metrics<F>(
        &self,
        event_pretty: EventPretty,
        callback: &F,
        bot_wallet: Option<Pubkey>,
        protocols: Vec<Protocol>,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync,
    {
        match event_pretty {
            EventPretty::Account(account_pretty) => {
                self.metrics_manager.add_process_count().await;
                let start_time = std::time::Instant::now();
                let program_received_time_ms = chrono::Utc::now().timestamp_millis();
                let account_event = AccountEventParser::parse_account_event(
                    protocols.clone(),
                    account_pretty,
                    program_received_time_ms,
                );
                if let Some(event) = account_event {
                    callback(event);
                    // 更新性能指标
                    let processing_time = start_time.elapsed();
                    let processing_time_ms = processing_time.as_millis() as f64;
                    // 更新性能指标（如果启用）
                    self.metrics_manager.update_metrics(1, processing_time_ms).await;
                    // 记录慢处理操作
                    self.metrics_manager.log_slow_processing(processing_time_ms, 1);
                }
            }
            EventPretty::Transaction(transaction_pretty) => {
                self.metrics_manager.add_process_count().await;
                let start_time = std::time::Instant::now();
                let program_received_time_ms = chrono::Utc::now().timestamp_millis();
                let slot = transaction_pretty.slot;
                let signature = transaction_pretty.signature.to_string();

                // 直接创建解析器并处理事务
                let parser: Arc<dyn EventParser> =
                    Arc::new(MutilEventParser::new(protocols.clone()));
                let all_events = parser
                    .parse_transaction(
                        transaction_pretty.tx.clone(),
                        &signature,
                        Some(slot),
                        transaction_pretty.block_time.map(|ts| prost_types::Timestamp {
                            seconds: ts.seconds,
                            nanos: ts.nanos,
                        }),
                        program_received_time_ms,
                        bot_wallet,
                    )
                    .await
                    .unwrap_or_else(|_e| vec![]);

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
                self.metrics_manager.update_metrics(event_count as u64, processing_time_ms).await;
                // 记录慢处理操作
                self.metrics_manager.log_slow_processing(processing_time_ms, event_count);
            }
            EventPretty::BlockMeta(block_meta_pretty) => {
                let start_time = std::time::Instant::now();
                self.metrics_manager.add_process_count().await;
                let block_time_ms = block_meta_pretty
                    .block_time
                    .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
                let block_meta_event = CommonEventParser::generate_block_meta_event(
                    block_meta_pretty.slot,
                    &block_meta_pretty.block_hash,
                    block_time_ms,
                );
                callback(block_meta_event);
                // 更新性能指标
                let processing_time = start_time.elapsed();
                let processing_time_ms = processing_time.as_millis() as f64;
                // 更新性能指标（如果启用）
                self.metrics_manager.update_metrics(1, processing_time_ms).await;
                // 记录慢处理操作
                self.metrics_manager.log_slow_processing(processing_time_ms, 1);
            }
        }

        Ok(())
    }

    /// 使用批处理处理事件交易
    pub async fn process_event_transaction_with_batch<F>(
        &self,
        event_pretty: EventPretty,
        batch_processor: &mut EventBatchCollector<F>,
        bot_wallet: Option<Pubkey>,
        protocols: Vec<Protocol>,
    ) -> AnyResult<()>
    where
        F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        match event_pretty {
            EventPretty::Account(account_pretty) => {
                self.metrics_manager.add_process_count().await;
                let start_time = std::time::Instant::now();
                let program_received_time_ms = chrono::Utc::now().timestamp_millis();
                let account_event = AccountEventParser::parse_account_event(
                    protocols.clone(),
                    account_pretty,
                    program_received_time_ms,
                );
                if let Some(event) = account_event {
                    (batch_processor.callback)(vec![event]);
                    // 更新性能指标
                    let processing_time = start_time.elapsed();
                    let processing_time_ms = processing_time.as_millis() as f64;
                    // 实际调用性能指标更新
                    self.metrics_manager.update_metrics(1, processing_time_ms).await;
                    // 记录慢处理操作
                    self.metrics_manager.log_slow_processing(processing_time_ms, 1);
                }
            }
            EventPretty::Transaction(transaction_pretty) => {
                self.metrics_manager.add_process_count().await;
                let start_time = std::time::Instant::now();
                let program_received_time_ms = chrono::Utc::now().timestamp_millis();
                let slot = transaction_pretty.slot;
                let signature = transaction_pretty.signature.to_string();

                // 直接创建解析器并处理事务
                let parser: Arc<dyn EventParser> =
                    Arc::new(MutilEventParser::new(protocols.clone()));
                let result = parser
                    .parse_transaction(
                        transaction_pretty.tx.clone(),
                        &signature,
                        Some(slot),
                        transaction_pretty.block_time.map(|ts| prost_types::Timestamp {
                            seconds: ts.seconds,
                            nanos: ts.nanos,
                        }),
                        program_received_time_ms,
                        bot_wallet,
                    )
                    .await;

                // 处理解析结果并使用批处理器
                let total_events = match result {
                    Ok(events) => {
                        let event_count = events.len();
                        if !events.is_empty() {
                            log::debug!("Parsed {} events", event_count);
                            log::debug!("Adding {} events to batch processor", event_count);
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
                        event_count
                    }
                    Err(e) => {
                        log::warn!("Failed to parse transaction: {:?}", e);
                        0
                    }
                };

                // 添加调试信息
                if total_events > 0 {
                    log::debug!(
                        "Total events parsed: {} for transaction {}",
                        total_events,
                        signature
                    );
                }

                // 更新性能指标
                let processing_time = start_time.elapsed();
                let processing_time_ms = processing_time.as_millis() as f64;

                // 实际调用性能指标更新
                self.metrics_manager.update_metrics(total_events as u64, processing_time_ms).await;

                // 记录慢处理操作
                self.metrics_manager.log_slow_processing(processing_time_ms, total_events);
            }
            EventPretty::BlockMeta(block_meta_pretty) => {
                let start_time = std::time::Instant::now();
                let block_time_ms = block_meta_pretty
                    .block_time
                    .map(|ts| ts.seconds * 1000 + ts.nanos as i64 / 1_000_000)
                    .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
                let block_meta_event = CommonEventParser::generate_block_meta_event(
                    block_meta_pretty.slot,
                    &block_meta_pretty.block_hash,
                    block_time_ms,
                );
                (batch_processor.callback)(vec![block_meta_event]);
                // 更新性能指标
                let processing_time = start_time.elapsed();
                let processing_time_ms = processing_time.as_millis() as f64;
                // 更新性能指标（如果启用）
                self.metrics_manager.update_metrics(1, processing_time_ms).await;
                // 记录慢处理操作
                self.metrics_manager.log_slow_processing(processing_time_ms, 1);
            }
        }

        Ok(())
    }
}
