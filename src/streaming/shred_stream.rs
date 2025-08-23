use solana_sdk::pubkey::Pubkey;

use crate::common::AnyResult;
use crate::streaming::common::{EventBatchProcessor, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::{Protocol, UnifiedEvent};
use crate::streaming::shred::{ShredEventProcessor, ShredStreamHandler, TransactionWithSlot};

use super::ShredStreamGrpc;

impl ShredStreamGrpc {
    /// 订阅ShredStream事件（支持批处理和即时处理）
    pub async fn shredstream_subscribe<F,FP>(
        &self,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
        tx_pack_callback: Option<FP>
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
        FP: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        // 如果已有活跃订阅，先停止它
        self.stop().await;

        let mut metrics_handle = None;
        // 启动自动性能监控（如果启用）
        if self.config.enable_metrics {
            metrics_handle = self.metrics_manager.start_auto_monitoring().await;
        }

        // 启动流处理
        let client = (*self.shredstream_client).clone();
        let (stream_task, rx) = ShredStreamHandler::start_stream_processing(
            client,
            self.config.backpressure.channel_size,
        )
        .await?;

        // 根据配置选择处理模式并获取事件处理任务句柄
        let event_handle = if self.config.batch.enabled {
            // 批处理模式
            self.process_with_batch(rx, protocols, bot_wallet, event_type_filter, callback).await?
        } else {
            // 即时处理模式
            self.process_immediate_in_tx_pack(rx, protocols, bot_wallet, event_type_filter, callback, tx_pack_callback).await?
        };

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_task, event_handle, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }

    /// 批处理模式
    async fn process_with_batch<F>(
        &self,
        mut rx: futures::channel::mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
    ) -> AnyResult<tokio::task::JoinHandle<()>>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        use futures::StreamExt;

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

        // 创建事件处理器
        let event_processor =
            ShredEventProcessor::new(self.metrics_manager.clone(), self.config.clone());

        let event_handle = tokio::spawn(async move {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = event_processor
                    .process_transaction_with_batch(
                        transaction_with_slot,
                        protocols.clone(),
                        bot_wallet,
                        &mut batch_processor,
                        event_type_filter.clone(),
                    )
                    .await
                {
                    log::error!("Error processing transaction: {e:?}");
                }
            }

            // 处理剩余的事件
            batch_processor.flush();
        });

        Ok(event_handle)
    }

    /// 即时处理模式
    async fn process_immediate<F>(
        &self,
        mut rx: futures::channel::mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
    ) -> AnyResult<tokio::task::JoinHandle<()>>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
    {
        use futures::StreamExt;

        // 创建事件处理器
        let event_processor =
            ShredEventProcessor::new(self.metrics_manager.clone(), self.config.clone());

        let event_handle = tokio::spawn(async move {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = event_processor
                    .process_transaction_immediate(
                        transaction_with_slot,
                        protocols.clone(),
                        bot_wallet,
                        event_type_filter.clone(),
                        &callback,
                    )
                    .await
                {
                    log::error!("Error processing transaction: {e:?}");
                }
            }
        });

        Ok(event_handle)
    }

    async fn process_immediate_in_tx_pack<F,FP>(
        &self,
        mut rx: futures::channel::mpsc::Receiver<TransactionWithSlot>,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
        tx_pack_callback: Option<FP>,
    ) -> AnyResult<()>
    where
        F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
        FP: Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
    {
        use futures::StreamExt;

        // 创建事件处理器
        let event_processor =
            ShredEventProcessor::new(self.metrics_manager.clone(), self.config.clone());

        let self_clone = self.clone();
        if let Some(pack_callback) = tx_pack_callback {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = event_processor
                    .process_transaction_immediate_in_tx_pack(
                        transaction_with_slot,
                        protocols.clone(),
                        bot_wallet,
                        event_type_filter.clone(),
                        &pack_callback,
                    )
                    .await
                {
                    log::error!("Error processing transaction: {e:?}");
                }
            }

        } else {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = event_processor
                    .process_transaction_immediate(
                        transaction_with_slot,
                        protocols.clone(),
                        bot_wallet,
                        event_type_filter.clone(),
                        &callback,
                    )
                    .await
                {
                    log::error!("Error processing transaction: {e:?}");
                }
            }

        }
        Ok(())
    }

}
