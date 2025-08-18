use solana_sdk::transaction::VersionedTransaction;

/// 携带槽位信息的交易
#[derive(Debug, Clone)]
pub struct TransactionWithSlot {
    pub transaction: VersionedTransaction,
    pub slot: u64,
}

impl TransactionWithSlot {
    /// 创建新的带槽位的交易
    pub fn new(transaction: VersionedTransaction, slot: u64) -> Self {
        Self { transaction, slot }
    }

    /// 获取交易签名
    pub fn signature(&self) -> String {
        self.transaction.signatures[0].to_string()
    }
}
