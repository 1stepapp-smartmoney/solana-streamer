use std::sync::Arc;

use futures::StreamExt;
use solana_sdk::pubkey::Pubkey;

use crate::common::AnyResult;
use crate::protos::shredstream::SubscribeEntriesRequest;
use crate::streaming::common::{EventProcessor, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use crate::streaming::event_parser::{Protocol, UnifiedEvent};
use crate::streaming::shred::pool::factory;
use solana_entry::entry::Entry;

use super::ShredStreamGrpc;
use log::error;

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

        let tx_pack_cb = tx_pack_callback.map(|cb| Arc::new(cb) as Arc<dyn for<'a> Fn(Vec<Box<dyn UnifiedEvent>>) + Send + Sync>);

        // 创建事件处理器
        let mut event_processor =
            EventProcessor::new(self.metrics_manager.clone(), self.config.clone());
        event_processor.set_protocols_and_event_type_filter(
            protocols,
            event_type_filter,
            self.config.backpressure.clone(),
            Some(Arc::new(callback)),
            tx_pack_cb,
        );

        // 启动流处理
        let mut client = (*self.shredstream_client).clone();
        let request = tonic::Request::new(SubscribeEntriesRequest {});
        let mut stream = client.subscribe_entries(request).await?.into_inner();
        let event_processor_clone = event_processor.clone();
        let stream_task = tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
                            for entry in entries {
                                for transaction in entry.transactions {
                                    let transaction_with_slot =
                                        factory::create_transaction_with_slot_pooled(
                                            transaction.clone(),
                                            msg.slot,
                                            get_high_perf_clock(),
                                        );
                                    // 直接处理，背压控制在 EventProcessor 内部处理
                                    if let Err(e) = event_processor_clone
                                        .process_shred_transaction_with_metrics(
                                            transaction_with_slot,
                                            bot_wallet,
                                        )
                                        .await
                                    {
                                        error!("Error handling message: {e:?}");
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    Err(error) => {
                        error!("Stream error: {error:?}");
                        break;
                    }
                }
            }
        });

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_task, None, metrics_handle);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}
