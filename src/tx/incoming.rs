use crate::{account::AccountId, Money};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::TxId;

#[derive(Error, Debug)]
pub enum IncomingTxError {
    #[error("error parsing amount: {0}")]
    ParseAmount(#[from] rust_decimal::Error),
    #[error("negative amount")]
    NegativeAmount,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[repr(C)]
pub enum IncomingTxDetails {
    Deposit(Money),
    Withdrawal(Money),
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[repr(C)]
pub struct IncomingTx {
    pub id: TxId,
    pub account: AccountId,
    pub details: IncomingTxDetails,
}

impl IncomingTxDetails {
    pub fn balance_effect(&self) -> Option<Money> {
        match self {
            IncomingTxDetails::Deposit(amount) => Some(*amount),
            IncomingTxDetails::Withdrawal(amount) => Some(-amount),
            _ => None,
        }
    }
}

impl IncomingTx {
    pub fn deposit(
        id: impl Into<TxId>,
        account: impl Into<AccountId>,
        amount: &str,
    ) -> Result<Self, IncomingTxError> {
        let amount = Money::from_str_exact(amount)?.ensure_non_negative()?;
        Ok(Self {
            id: id.into(),
            account: account.into(),
            details: IncomingTxDetails::Deposit(amount),
        })
    }

    pub fn withdrawal(
        id: impl Into<TxId>,
        account: impl Into<AccountId>,
        amount: &str,
    ) -> Result<Self, IncomingTxError> {
        let amount = Money::from_str_exact(amount)?.ensure_non_negative()?;
        Ok(Self {
            id: id.into(),
            account: account.into(),
            details: IncomingTxDetails::Withdrawal(amount),
        })
    }

    pub fn dispute(id: impl Into<TxId>, account: impl Into<AccountId>) -> Self {
        Self {
            id: id.into(),
            account: account.into(),
            details: IncomingTxDetails::Dispute,
        }
    }

    pub fn resolve(id: impl Into<TxId>, account: impl Into<AccountId>) -> Self {
        Self {
            id: id.into(),
            account: account.into(),
            details: IncomingTxDetails::Resolve,
        }
    }

    pub fn chargeback(id: impl Into<TxId>, account: impl Into<AccountId>) -> Self {
        Self {
            id: id.into(),
            account: account.into(),
            details: IncomingTxDetails::Chargeback,
        }
    }
}

trait MoneyExt: Sized {
    fn ensure_non_negative(self) -> Result<Self, IncomingTxError>;
}

impl MoneyExt for Money {
    fn ensure_non_negative(mut self) -> Result<Self, IncomingTxError> {
        if self.is_sign_negative() && !self.is_zero() {
            Err(IncomingTxError::NegativeAmount)
        } else {
            self.set_sign_positive(true);
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Money;

    use super::MoneyExt;

    #[test]
    fn money_ensure_non_negative_makes_zero_positive() {
        assert_eq!(
            Money::from_str_exact("-0.0")
                .unwrap()
                .ensure_non_negative()
                .unwrap(),
            Money::from_str_exact("0.0").unwrap()
        );
    }

    #[test]
    fn money_ensure_non_negative_fails_on_negative() {
        assert_eq!(
            Money::from_str_exact("-1.0")
                .unwrap()
                .ensure_non_negative()
                .ok(),
            None
        );
    }

    #[test]
    fn money_ensure_non_negative_leaves_positive_untouched() {
        assert_eq!(
            Money::from_str_exact("1.0")
                .unwrap()
                .ensure_non_negative()
                .unwrap(),
            Money::from_str_exact("1.0").unwrap()
        );
    }
}
