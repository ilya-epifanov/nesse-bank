use std::{io::Read, num::ParseIntError};

use csv::{StringRecordsIntoIter, Trim};
use thiserror::Error;

use crate::{
    account::AccountId,
    tx::{
        incoming::{IncomingTx, IncomingTxError},
        TxId,
    },
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("error parsing CSV")]
    Csv(#[from] csv::Error),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("error parsing integer")]
    IntField(#[from] ParseIntError),
    #[error("transaction error: {0}")]
    IncomingTransaction(#[from] IncomingTxError),
    #[error("unknown transaction type `{0}`")]
    UnknownTransactionType(String),
}

pub struct RecordsIter<R: Read> {
    inner: StringRecordsIntoIter<R>,
}

impl<R: Read> Iterator for RecordsIter<R> {
    type Item = Result<IncomingTx, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            // Skip the last empty line
            .filter(|r| {
                r.as_ref()
                    .map(|r| !r.is_empty() && r.get(0) != Some(""))
                    .unwrap_or(true)
            })
            .map(|record| {
                // I decided to parse fields manually because csv's serde implementation is wonky at times
                // Also there would be more of the supporting code spread across multiple places
                let record = record?;
                let r#type = record.get(0).ok_or(ParseError::MissingField("type"))?;
                let account = AccountId(
                    record
                        .get(1)
                        .ok_or(ParseError::MissingField("client"))?
                        .parse()?,
                );
                let id = TxId(
                    record
                        .get(2)
                        .ok_or(ParseError::MissingField("tx"))?
                        .parse()?,
                );
                match r#type {
                    "deposit" => {
                        let amount = record.get(3).ok_or(ParseError::MissingField("amount"))?;
                        Ok(IncomingTx::deposit(id, account, amount)?)
                    }
                    "withdrawal" => {
                        let amount = record.get(3).ok_or(ParseError::MissingField("amount"))?;
                        Ok(IncomingTx::withdrawal(id, account, amount)?)
                    }
                    "dispute" => Ok(IncomingTx::dispute(id, account)),
                    "resolve" => Ok(IncomingTx::resolve(id, account)),
                    "chargeback" => Ok(IncomingTx::chargeback(id, account)),
                    unknown_type => {
                        Err(ParseError::UnknownTransactionType(unknown_type.to_owned()))
                    }
                }
            })
    }
}

pub fn csv_reader<R: Read>(reader: R) -> RecordsIter<R> {
    RecordsIter {
        inner: csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .flexible(true) // Otherwise we get errors on empty lines
            .from_reader(reader)
            .into_records(),
    }
}
