use futures::{channel::mpsc, StreamExt};
use log::error;
use solana_entry::entry::Entry;
use tokio::task::JoinHandle;

use crate::common::AnyResult;
use crate::protos::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
};
use crate::streaming::shred::TransactionWithSlot;

/// ShredStream 流处理器
pub struct ShredStreamHandler;

impl ShredStreamHandler {
    /// 启动 ShredStream 流处理任务
    ///
    /// # 参数
    /// * `client` - ShredStream 客户端
    /// * `tx` - 事务发送通道
    /// * `channel_size` - 通道缓冲区大小
    ///
    /// # 返回值
    /// 返回 ShredStream 流处理任务句柄和事务接收通道
    pub async fn start_stream_processing(
        mut client: ShredstreamProxyClient<tonic::transport::Channel>,
        channel_size: usize,
    ) -> AnyResult<(JoinHandle<()>, mpsc::Receiver<TransactionWithSlot>)> {
        let request = tonic::Request::new(SubscribeEntriesRequest {});
        let stream = client.subscribe_entries(request).await?.into_inner();
        let (tx, rx) = mpsc::channel::<TransactionWithSlot>(channel_size);

        let stream_task = tokio::spawn(Self::process_stream_messages(stream, tx));

        Ok((stream_task, rx))
    }

    /// 处理流消息
    ///
    /// # 参数
    /// * `stream` - ShredStream 数据流
    /// * `tx` - 事务发送通道
    async fn process_stream_messages(
        mut stream: tonic::codec::Streaming<crate::protos::shredstream::Entry>,
        mut tx: mpsc::Sender<TransactionWithSlot>,
    ) {
        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    if let Err(e) = Self::handle_stream_message(msg, &mut tx).await {
                        error!("Error handling stream message: {e:?}");
                        continue;
                    }
                }
                Err(error) => {
                    error!("Stream error: {error:?}");
                    break;
                }
            }
        }
    }

    /// 处理单个流消息
    ///
    /// # 参数
    /// * `msg` - ShredStream 消息
    /// * `tx` - 事务发送通道
    async fn handle_stream_message(
        msg: crate::protos::shredstream::Entry,
        tx: &mut mpsc::Sender<TransactionWithSlot>,
    ) -> AnyResult<()> {
        if let Ok(entries) = bincode::deserialize::<Vec<Entry>>(&msg.entries) {
            for entry in entries {
                for transaction in entry.transactions {
                    let transaction_with_slot =
                        TransactionWithSlot::new(transaction.clone(), msg.slot);

                    if let Err(e) = tx.try_send(transaction_with_slot) {
                        // 如果通道满了，记录警告但不中断处理
                        if e.is_full() {
                            log::warn!("Transaction channel is full, dropping transaction");
                        } else {
                            // 通道已关闭，返回错误
                            return Err(e.into());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 启动事务处理任务
    ///
    /// # 参数
    /// * `rx` - 事务接收通道
    /// * `processor` - 事务处理器
    pub fn start_transaction_processing<F>(
        mut rx: mpsc::Receiver<TransactionWithSlot>,
        processor: F,
    ) -> JoinHandle<()>
    where
        F: Fn(TransactionWithSlot) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        tokio::spawn(async move {
            while let Some(transaction_with_slot) = rx.next().await {
                if let Err(e) = processor(transaction_with_slot) {
                    error!("Error processing transaction: {e:?}");
                }
            }
        })
    }
}
