use std::fmt::Debug;

use kv::Raw;
use serde::{Deserialize, Serialize};

use super::incoming::IncomingTx;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[repr(C)]
pub struct TxDetails {
    pub original_tx: IncomingTx,
    pub state: TxState,
}

impl TxDetails {
    pub fn with_state(self, state: TxState) -> Self {
        Self {
            original_tx: self.original_tx,
            state,
        }
    }
}
const TX_DETAILS_SIZE: usize = std::mem::size_of::<TxDetails>();

impl From<Raw> for TxDetails {
    fn from(raw: Raw) -> Self {
        debug_assert_eq!(raw.len(), TX_DETAILS_SIZE);

        // MaybeUninit is still quirky with arrays on stable
        // Also, zeroing out will be optimized away in --release
        let mut bytes = [0; TX_DETAILS_SIZE];
        bytes.copy_from_slice(&raw);

        // SAFETY
        // safe because TxDetails is Copy
        unsafe { std::mem::transmute(bytes) }
    }
}

impl From<TxDetails> for Raw {
    fn from(tx: TxDetails) -> Self {
        // SAFETY
        // safe because TxDetails is Copy
        let v: [u8; TX_DETAILS_SIZE] = unsafe { std::mem::transmute(tx) };
        Raw::from(&v)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(C)]
pub enum TxState {
    Complete,
    UnderDispute,
    Resolved,
    ChargedBack,
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use kv::Raw;

    use crate::tx::incoming::IncomingTx;

    use super::TxDetails;

    #[test]
    fn tx_details_to_raw_and_back() {
        let original = TxDetails {
            original_tx: IncomingTx::deposit(123, 456, "789.1112").unwrap(),
            state: super::TxState::Complete,
        };

        let raw: Raw = original.clone().into();
        assert_debug_snapshot!(&raw);

        let recovered: TxDetails = raw.into();
        assert_eq!(recovered, original);
    }
}
