use chrono::Local;
use futures::{channel::mpsc, sink::Sink, SinkExt};
use log::info;
use yellowstone_grpc_proto::geyser::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestPing, SubscribeUpdate,
};

use super::types::{BlockMetaPretty, EventPretty, TransactionPretty};
use crate::common::AnyResult;
use crate::streaming::common::BackpressureStrategy;

/// 流消息处理器
pub struct StreamHandler;

impl StreamHandler {
    /// 处理单个流消息
    pub async fn handle_stream_message(
        msg: SubscribeUpdate,
        tx: &mut mpsc::Sender<EventPretty>,
        subscribe_tx: &mut (impl Sink<SubscribeRequest, Error = mpsc::SendError> + Unpin),
        backpressure_strategy: BackpressureStrategy,
    ) -> AnyResult<()> {
        let created_at = msg.created_at;
        match msg.update_oneof {
            Some(UpdateOneof::BlockMeta(sut)) => {
                let block_meta_pretty = BlockMetaPretty::from((sut, created_at));
                log::info!("Received block meta: {:?}", block_meta_pretty);
                Self::handle_backpressure(
                    tx,
                    EventPretty::BlockMeta(block_meta_pretty),
                    backpressure_strategy,
                )
                .await?;
            }
            Some(UpdateOneof::Transaction(sut)) => {
                let transaction_pretty = TransactionPretty::from((sut, created_at));
                log::info!(
                    "Received transaction: {} at slot {}",
                    transaction_pretty.signature,
                    transaction_pretty.slot
                );

                // 根据背压策略处理发送
                Self::handle_backpressure(
                    tx,
                    EventPretty::Transaction(transaction_pretty),
                    backpressure_strategy,
                )
                .await?;
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

    /// 处理背压策略
    async fn handle_backpressure(
        tx: &mut mpsc::Sender<EventPretty>,
        event_pretty: EventPretty,
        backpressure_strategy: BackpressureStrategy,
    ) -> AnyResult<()> {
        match backpressure_strategy {
            BackpressureStrategy::Block => {
                // 阻塞等待，直到有空间
                if let Err(e) = tx.send(event_pretty).await {
                    log::error!("Failed to send transaction to channel: {:?}", e);
                    return Err(anyhow::anyhow!("Channel send failed: {:?}", e));
                }
            }
            BackpressureStrategy::Drop => {
                // 尝试发送，如果失败则丢弃
                if let Err(e) = tx.try_send(event_pretty) {
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
                    match tx.try_send(event_pretty.clone()) {
                        Ok(_) => break,
                        Err(e) => {
                            if e.is_full() {
                                retry_count += 1;
                                if retry_count >= max_attempts {
                                    log::warn!(
                                        "Channel is full after {} attempts, dropping transaction",
                                        retry_count
                                    );
                                    break;
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms))
                                    .await;
                            } else {
                                log::error!("Channel is closed: {:?}", e);
                                return Err(anyhow::anyhow!("Channel is closed: {:?}", e));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
