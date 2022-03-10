use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use kv::{Bucket, Integer, Raw};

use crate::{
    account::{Account, AccountId},
    tx::{incoming::IncomingTx, stored::TxDetails, TxId},
};

pub struct Bank {
    tx_cache: Box<dyn TxCache>,
    accounts: BTreeMap<AccountId, Account>,
}

impl Default for Bank {
    fn default() -> Self {
        Self {
            tx_cache: Box::new(InMemoryTxCache::default()),
            accounts: Default::default(),
        }
    }
}

impl Bank {
    pub fn with_cache(tx_cache: Box<dyn TxCache>) -> Self {
        Self {
            tx_cache,
            accounts: Default::default(),
        }
    }

    pub fn apply_tx(&mut self, tx: IncomingTx) {
        let account = self.accounts.entry(tx.account).or_default();
        let prev_tx = self.tx_cache.get_by_id(tx.id);

        if let Some(new_tx_state) = account.apply_tx(prev_tx.as_ref(), &tx) {
            self.tx_cache.store(new_tx_state);
        }
    }

    pub fn into_accounts(self) -> BTreeMap<AccountId, Account> {
        self.accounts
    }
}

pub trait TxCache {
    fn get_by_id(&self, id: TxId) -> Option<TxDetails>;
    fn store(&mut self, tx: TxDetails);
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryTxCache {
    tx_by_id: HashMap<TxId, TxDetails>,
}

impl TxCache for InMemoryTxCache {
    fn get_by_id(&self, id: TxId) -> Option<TxDetails> {
        self.tx_by_id.get(&id).map(ToOwned::to_owned)
    }

    fn store(&mut self, tx: TxDetails) {
        self.tx_by_id.insert(tx.original_tx.id, tx);
    }
}

pub struct OnDiskTxCache<'c> {
    bucket: Bucket<'c, Integer, Raw>,
}

impl<'c> OnDiskTxCache<'c> {
    pub fn new(bucket: Bucket<'c, Integer, Raw>) -> Self {
        Self { bucket }
    }
}

impl<'c> TxCache for OnDiskTxCache<'c> {
    fn get_by_id(&self, id: TxId) -> Option<TxDetails> {
        let cached = self
            .bucket
            .get(&Integer::from(id.0))
            .expect("can't retrieve Tx details from the cache")?;
        Some(cached.into())
    }

    fn store(&mut self, tx: TxDetails) {
        self.bucket
            .set(&Integer::from(tx.original_tx.id.0), &tx.into())
            .expect("can't store Tx details in the cache");
    }
}
