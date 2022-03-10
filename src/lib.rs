use rust_decimal::Decimal;

pub mod account;
pub mod bank;
pub mod io;
pub mod tx;
pub mod util;

#[cfg(test)]
mod tests;

pub type Money = Decimal;
