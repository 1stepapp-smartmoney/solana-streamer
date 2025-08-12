use futures::{channel::mpsc, sink::Sink, Stream};
use maplit::hashmap;
use std::{collections::HashMap, time::Duration};
use tonic::{transport::channel::ClientTlsConfig, Status};
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterBlocksMeta,
    SubscribeRequestFilterTransactions, SubscribeUpdate,
};

use super::types::TransactionsFilterMap;
use crate::common::AnyResult;
use crate::streaming::common::StreamClientConfig as ClientConfig;

/// 订阅管理器
#[derive(Clone)]
pub struct SubscriptionManager {
    endpoint: String,
    x_token: Option<String>,
    config: ClientConfig,
}

impl SubscriptionManager {
    /// 创建新的订阅管理器
    pub fn new(endpoint: String, x_token: Option<String>, config: ClientConfig) -> Self {
        Self { endpoint, x_token, config }
    }

    /// 创建 gRPC 连接
    pub async fn connect(&self) -> AnyResult<GeyserGrpcClient<impl Interceptor>> {
        let builder = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .max_decoding_message_size(self.config.connection.max_decoding_message_size)
            .connect_timeout(Duration::from_secs(self.config.connection.connect_timeout))
            .timeout(Duration::from_secs(self.config.connection.request_timeout));
        Ok(builder.connect().await?)
    }

    /// 创建订阅请求并返回流
    pub async fn subscribe_with_request(
        &self,
        transactions: TransactionsFilterMap,
        commitment: Option<CommitmentLevel>,
    ) -> AnyResult<(
        impl Sink<SubscribeRequest, Error = mpsc::SendError>,
        impl Stream<Item = Result<SubscribeUpdate, Status>>,
    )> {
        let subscribe_request = SubscribeRequest {
            transactions,
            blocks_meta: hashmap! { "".to_owned() => SubscribeRequestFilterBlocksMeta {} },
            commitment: if let Some(commitment) = commitment {
                Some(commitment as i32)
            } else {
                Some(CommitmentLevel::Processed.into())
            },
            ..Default::default()
        };

        let mut client = self.connect().await?;
        let (sink, stream) = client.subscribe_with_request(Some(subscribe_request)).await?;
        Ok((sink, stream))
    }

    /// 生成订阅请求过滤器
    pub fn get_subscribe_request_filter(
        &self,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
    ) -> TransactionsFilterMap {
        let mut transactions = HashMap::new();
        transactions.insert(
            "client".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include,
                account_exclude,
                account_required,
            },
        );
        transactions
    }

    /// 验证订阅参数
    pub fn validate_subscription_params(
        &self,
        account_include: &[String],
        account_exclude: &[String],
        account_required: &[String],
    ) -> AnyResult<()> {
        if account_include.is_empty() && account_exclude.is_empty() && account_required.is_empty() {
            return Err(anyhow::anyhow!(
                "account_include or account_exclude or account_required cannot be empty"
            ));
        }
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self) -> &ClientConfig {
        &self.config
    }
}
