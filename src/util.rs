use std::io::{Read, Write};

use kv::{Config, Integer, Raw, Store};
use tempdir::TempDir;
use thiserror::Error;

use crate::{
    account::AccountState,
    bank::{Bank, InMemoryTxCache, OnDiskTxCache, TxCache},
    io::{csv_reader, ParseError},
};

#[derive(Error, Debug)]
pub enum HistoricRunError {
    #[error("error parsing transactions")]
    ParseError(#[from] ParseError),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Cache error: {0}")]
    Cache(#[from] kv::Error),
}

pub fn historic_run(input: impl Read, cache: Box<dyn TxCache>) -> Result<Bank, HistoricRunError> {
    let mut state = Bank::with_cache(cache);

    for tx in csv_reader(input) {
        let tx = tx?;
        state.apply_tx(tx);
    }

    Ok(state)
}

pub fn historic_run_small(input: impl Read) -> Result<Bank, HistoricRunError> {
    historic_run(input, Box::new(InMemoryTxCache::default()))
}

pub fn historic_run_large(input: impl Read) -> Result<(Bank, TempDir), HistoricRunError> {
    let temp_dir = TempDir::new("nesse-bank")?;
    let store_cfg = Config::new(temp_dir.path());
    let store = Store::new(store_cfg)?;
    let tx_bucket = store.bucket::<Integer, Raw>(Some("tx"))?;

    Ok((
        historic_run(input, Box::new(OnDiskTxCache::new(tx_bucket)))?,
        temp_dir,
    ))
}

pub fn write_state(state: Bank, output: impl Write) -> Result<(), csv::Error> {
    let mut out = csv::WriterBuilder::new().from_writer(output);
    out.write_record(&["client", "available", "held", "total", "locked"])?;

    for (account_id, account) in state.into_accounts() {
        out.write_record(&[
            account_id.to_string().as_str(),
            account.balance.to_string().as_str(),
            account.held.to_string().as_str(),
            (account.balance + account.held).to_string().as_str(),
            if account.state == AccountState::Frozen {
                "true"
            } else {
                "false"
            },
        ])?;
    }

    Ok(())
}
