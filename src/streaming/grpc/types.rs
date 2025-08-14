use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{collections::HashMap, fmt};
use yellowstone_grpc_proto::{
    geyser::{
        SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions, SubscribeUpdateAccount,
        SubscribeUpdateBlockMeta, SubscribeUpdateTransaction,
    },
    prost_types::Timestamp,
};

pub type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
pub type AccountsFilterMap = HashMap<String, SubscribeRequestFilterAccounts>;

#[derive(Clone)]
pub enum EventPretty {
    BlockMeta(BlockMetaPretty),
    Transaction(TransactionPretty),
    Account(AccountPretty),
}

#[derive(Clone)]
pub struct AccountPretty {
    pub slot: u64,
    pub signature: String,
    pub pubkey: String,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
}

impl fmt::Debug for AccountPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("pubkey", &self.pubkey)
            .field("executable", &self.executable)
            .field("lamports", &self.lamports)
            .field("owner", &self.owner)
            .field("rent_epoch", &self.rent_epoch)
            .field("data", &self.data)
            .finish()
    }
}

#[derive(Clone)]
pub struct BlockMetaPretty {
    pub slot: u64,
    pub block_hash: String,
    pub block_time: Option<Timestamp>,
}

impl fmt::Debug for BlockMetaPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockMetaPretty")
            .field("slot", &self.slot)
            .field("block_hash", &self.block_hash)
            .field("block_time", &self.block_time)
            .finish()
    }
}

#[derive(Clone)]
pub struct TransactionPretty {
    pub slot: u64,
    pub block_hash: String,
    pub block_time: Option<Timestamp>,
    pub signature: Signature,
    pub is_vote: bool,
    pub tx: EncodedTransactionWithStatusMeta,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TxWrap<'a>(&'a EncodedTransactionWithStatusMeta);
        impl<'a> fmt::Debug for TxWrap<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let serialized = serde_json::to_string(self.0).expect("failed to serialize");
                fmt::Display::fmt(&serialized, f)
            }
        }

        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("tx", &TxWrap(&self.tx))
            .finish()
    }
}

impl From<SubscribeUpdateAccount> for AccountPretty {
    fn from(account: SubscribeUpdateAccount) -> Self {
        let account_info = account.account.unwrap();
        Self {
            slot: account.slot,
            signature: bs58::encode(&account_info.txn_signature.unwrap_or_default()).into_string(),
            pubkey: bs58::encode(&account_info.pubkey).into_string(),
            executable: account_info.executable,
            lamports: account_info.lamports,
            owner: bs58::encode(&account_info.owner).into_string(),
            rent_epoch: account_info.rent_epoch,
            data: account_info.data,
        }
    }
}

impl From<(SubscribeUpdateBlockMeta, Option<Timestamp>)> for BlockMetaPretty {
    fn from(
        (SubscribeUpdateBlockMeta { slot, blockhash, .. }, block_time): (
            SubscribeUpdateBlockMeta,
            Option<Timestamp>,
        ),
    ) -> Self {
        Self { block_hash: blockhash.to_string(), block_time, slot }
    }
}

impl From<(SubscribeUpdateTransaction, Option<Timestamp>)> for TransactionPretty {
    fn from(
        (SubscribeUpdateTransaction { transaction, slot }, block_time): (
            SubscribeUpdateTransaction,
            Option<Timestamp>,
        ),
    ) -> Self {
        let tx = transaction.expect("should be defined");
        Self {
            slot,
            block_time,
            block_hash: "".to_string(),
            signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
            is_vote: tx.is_vote,
            tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
                .expect("valid tx with meta")
                .encode(UiTransactionEncoding::Base64, Some(u8::MAX), true)
                .expect("failed to encode"),
        }
    }
}
