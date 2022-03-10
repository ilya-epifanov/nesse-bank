use clap::{ArgEnum, Parser};
use kv::{Config, Integer, Raw, Store};
use nesse_bank::{
    bank::{InMemoryTxCache, OnDiskTxCache, TxCache},
    util::{historic_run, write_state},
};
use std::{fmt::Debug, fs::File, path::PathBuf};
use tempdir::TempDir;

/// This program does historic run over a list of transactions and outputs the final state of accounts
#[derive(Debug, Parser)]
struct Args {
    // transaction cache backend
    #[clap(arg_enum, short, long, default_value = "disk")]
    cache_backend: TxCacheBackend,
    /// input csv file with columns: type, client, tx, amount
    input_file: PathBuf,
}

#[derive(ArgEnum, Clone, Debug)]
#[clap(rename_all = "lower")]
enum TxCacheBackend {
    Memory,
    Disk,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let cache: Box<dyn TxCache> = match args.cache_backend {
        TxCacheBackend::Memory => Box::new(InMemoryTxCache::default()),
        TxCacheBackend::Disk => Box::new({
            let temp_dir = TempDir::new("nesse-bank")?;
            let store_cfg = Config::new(temp_dir.path());
            let store = Store::new(store_cfg)?;
            let bucket = store.bucket::<Integer, Raw>(Some("tx"))?;
            OnDiskTxCache::new(bucket)
        }),
    };

    let state = historic_run(File::open(args.input_file)?, cache)?;
    write_state(state, std::io::stdout())?;

    Ok(())
}
