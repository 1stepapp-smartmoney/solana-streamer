use solana_sdk::pubkey::Pubkey;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use crate::common::AnyResult;
use crate::streaming::common::{
    EventBatchProcessor, MetricsEventType, MetricsManager, StreamClientConfig,
};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::protocols::MutilEventParser;
use crate::streaming::event_parser::{EventParser, Protocol, UnifiedEvent};
use crate::streaming::shred::TransactionWithSlot;

/// ShredStream 事件处理器
pub struct ShredEventProcessor {
    pub(crate) metrics_manager: MetricsManager,
    pub(crate) config: StreamClientConfig,
    pub(crate) parser_cache: Arc<Mutex<Option<Arc<dyn EventParser>>>>,
}

impl ShredEventProcessor {
    /// 创建新的事件处理器
    pub fn new(metrics_manager: MetricsManager, config: StreamClientConfig) -> Self {
        Self { metrics_manager, config, parser_cache: Arc::new(Mutex::new(None)) }
    }

    /// 获取或创建解析器，使用缓存机制避免重复创建
    fn get_or_create_parser(
        &self,
        protocols: Vec<Protocol>,
        event_type_filter: Option<EventTypeFilter>,
    ) -> Arc<dyn EventParser> {
        let mut cache = self.parser_cache.lock().unwrap();
        if let Some(cached_parser) = cache.clone() {
            return cached_parser.clone();
        }
        let parser: Arc<dyn EventParser> =
            Arc::new(MutilEventParser::new(protocols.clone(), event_type_filter.clone()));
        *cache = Some(parser.clone());
        parser
    }

    /// 即时处理单个交易
    pub async fn process_transaction_immediate<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: &F,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        self.metrics_manager.add_tx_process_count().await;
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        if (versioned_tx.signatures.len() == 0) {
            return Err(Error::msg("Malformed Transaction has no signatures"));
        }
        let signature = versioned_tx.signatures[0];

        // 获取缓存的解析器
        let parser = self.get_or_create_parser(protocols, event_type_filter);

        let all_events = parser
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
        self.metrics_manager.log_slow_processing(processing_time_ms, event_count);

        Ok(())
    }

    pub async fn process_transaction_immediate_in_tx_pack<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: &F,
    ) -> AnyResult<()>
    where
        F: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        self.metrics_manager.add_tx_process_count().await;
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        if (versioned_tx.signatures.len() == 0) {
            return Err(Error::msg("Malformed Transaction has no signatures"));
        }
        let signature = versioned_tx.signatures[0];

        // 获取缓存的解析器
        let parser = self.get_or_create_parser(protocols, event_type_filter);

        let all_events = parser
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

        // 保存事件数量用于日志记录
        let event_count = all_events.len();

        // 即时处理事件
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
        self.metrics_manager.log_slow_processing(processing_time_ms, event_count);

        Ok(())
    }

    /// 批处理模式处理单个交易
    pub async fn process_transaction_with_batch<F>(
        &self,
        transaction_with_slot: TransactionWithSlot,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        batch_processor: &mut EventBatchProcessor<F>,
        event_type_filter: Option<EventTypeFilter>,
    ) -> AnyResult<()>
    where
        F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        let start_time = std::time::Instant::now();
        self.metrics_manager.add_tx_process_count().await;
        let program_received_time_ms = chrono::Utc::now().timestamp_millis();
        let slot = transaction_with_slot.slot;
        let versioned_tx = transaction_with_slot.transaction;
        let signature = versioned_tx.signatures[0];

        // 获取缓存的解析器
        let parser = self.get_or_create_parser(protocols, event_type_filter);

        let all_events = parser
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
        self.metrics_manager.log_slow_processing(processing_time_ms, event_count);

        Ok(())
    }

    /// 更新性能指标
    async fn update_metrics(&self, events_processed: u64, processing_time_ms: f64) {
        // 使用统一的指标管理器，这里假设 ShredStream 主要处理交易事件
        self.metrics_manager
            .update_metrics(MetricsEventType::Tx, events_processed, processing_time_ms)
            .await;
    }
}

// 实现 Clone trait 以支持模块间共享
impl Clone for ShredEventProcessor {
    fn clone(&self) -> Self {
        Self {
            metrics_manager: self.metrics_manager.clone(),
            config: self.config.clone(),
            parser_cache: self.parser_cache.clone(),
        }
    }
}
