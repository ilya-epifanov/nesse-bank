use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use crate::{
    tx::{
        incoming::{IncomingTx, IncomingTxDetails},
        stored::{TxDetails, TxState},
    },
    Money,
};

#[derive(
    Copy, Clone, PartialEq, Eq, From, Debug, Display, PartialOrd, Ord, Deserialize, Serialize,
)]
#[serde(transparent)]
pub struct AccountId(pub u16);

#[cfg_attr(test, derive(Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountState {
    Active,
    Frozen,
}

impl Default for AccountState {
    fn default() -> Self {
        Self::Active
    }
}

#[cfg_attr(test, derive(Serialize))]
#[derive(Clone, Debug, Default)]
pub struct Account {
    pub balance: Money,
    pub held: Money,
    pub state: AccountState,
}

impl Account {
    pub fn apply_tx(&mut self, prev_tx: Option<&TxDetails>, tx: &IncomingTx) -> Option<TxDetails> {
        match (prev_tx, &tx.details) {
            (None, IncomingTxDetails::Deposit(amount)) => {
                if self.state == AccountState::Frozen {
                    return None;
                }
                self.balance += amount;
                Some(TxDetails {
                    original_tx: *tx,
                    state: TxState::Complete,
                })
            }
            (None, IncomingTxDetails::Withdrawal(amount)) => {
                if self.state == AccountState::Frozen {
                    return None;
                }
                if &self.balance < amount {
                    return None;
                }
                self.balance -= amount;
                Some(TxDetails {
                    original_tx: *tx,
                    state: TxState::Complete,
                })
            }
            (
                Some(
                    prev_tx @ TxDetails {
                        original_tx,
                        state: TxState::Complete,
                    },
                ),
                IncomingTxDetails::Dispute,
            ) => {
                // assumption: even if the account is frozen, some other tx might be disputed
                let balance_effect = original_tx.details.balance_effect().unwrap();
                self.balance -= balance_effect;
                self.held += balance_effect;
                Some(prev_tx.with_state(TxState::UnderDispute))
            }
            (
                Some(
                    prev_tx @ TxDetails {
                        original_tx,
                        state: TxState::UnderDispute,
                    },
                ),
                IncomingTxDetails::Resolve,
            ) => {
                // assumption: even if the account is frozen, some other tx might be disputed
                let balance_effect = original_tx.details.balance_effect().unwrap();
                self.balance += balance_effect;
                self.held -= balance_effect;
                Some(prev_tx.with_state(TxState::Resolved))
            }
            (
                Some(
                    prev_tx @ TxDetails {
                        original_tx,
                        state: TxState::UnderDispute,
                    },
                ),
                IncomingTxDetails::Chargeback,
            ) => {
                // assumption: even if the account is frozen, some other tx might be disputed
                let balance_effect = original_tx.details.balance_effect().unwrap();
                self.held -= balance_effect;
                self.state = AccountState::Frozen;
                Some(prev_tx.with_state(TxState::ChargedBack))
            }
            _ => None,
        }
    }
}
