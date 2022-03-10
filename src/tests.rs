use std::{
    fs::File,
    path::{Path, PathBuf},
};

use insta::{assert_snapshot, glob};
use itertools::Itertools;

use crate::{io::csv_reader, tx::incoming::IncomingTx, util::write_state, Money};

#[test]
fn historic_runs() {
    glob!("test-data/historic-runs/*.csv", |path| {
        assert_snapshot!(historic_run_small(path));
    });
}

fn historic_run_small(path: impl AsRef<Path>) -> String {
    let state = crate::util::historic_run_small(File::open(path).unwrap()).unwrap();
    let mut buf = Vec::new();
    write_state(state, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

fn historic_run_large(path: impl AsRef<Path>) -> String {
    let (state, temp_dir) = crate::util::historic_run_large(File::open(path).unwrap()).unwrap();
    let mut buf = Vec::new();
    write_state(state, &mut buf).unwrap();
    temp_dir.close().unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn read_sample_input_with_newline() {
    let input = r#"type, client, tx, amount
            deposit, 1, 1, 1.0
            deposit, 2, 2, 2.0
            deposit, 1, 3, 2.0
            withdrawal, 1, 4, 1.5
            withdrawal, 2, 5, 3.0
            "#
    .as_bytes();

    let records = csv_reader(input).map(Result::ok).collect_vec();

    assert_eq!(
        records,
        vec![
            Some(IncomingTx::deposit(1, 1, "1.0").unwrap()),
            Some(IncomingTx::deposit(2, 2, "2.0").unwrap()),
            Some(IncomingTx::deposit(3, 1, "2.0").unwrap()),
            Some(IncomingTx::withdrawal(4, 1, "1.5").unwrap()),
            Some(IncomingTx::withdrawal(5, 2, "3.0").unwrap()),
        ]
    );
}

#[test]
fn read_sample_input_without_newline() {
    let input = r#"type, client, tx, amount
            deposit, 1, 1, 1.0
            deposit, 2, 2, 2.0
            deposit, 1, 3, 2.0
            withdrawal, 1, 4, 1.5
            withdrawal, 2, 5, 3.0"#
        .as_bytes();

    let records = csv_reader(input).map(Result::ok).collect_vec();

    assert_eq!(
        records,
        vec![
            Some(IncomingTx::deposit(1, 1, "1.0").unwrap()),
            Some(IncomingTx::deposit(2, 2, "2.0").unwrap()),
            Some(IncomingTx::deposit(3, 1, "2.0").unwrap()),
            Some(IncomingTx::withdrawal(4, 1, "1.5").unwrap()),
            Some(IncomingTx::withdrawal(5, 2, "3.0").unwrap()),
        ]
    );
}

#[ignore = "requires 2.6GiB of disk space and runs for tens of seconds with --release"]
#[test]
fn handle_10mil_transactions() {
    let tx_path = PathBuf::from("10mil-transactions.csv");

    {
        let mut out = csv::WriterBuilder::new().from_writer(File::create(&tx_path).unwrap());
        out.write_record(&["type", "client", "tx", "amount"])
            .unwrap();

        for i in 0..10_000_000 {
            out.write_record(&[
                "deposit",
                "1",
                i.to_string().as_str(),
                (Money::new(i as i64, 4)).to_string().as_str(),
            ])
            .unwrap();
        }
    }

    assert_snapshot!(historic_run_large(tx_path));
}
