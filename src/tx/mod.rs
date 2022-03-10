use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

pub mod incoming;
pub mod stored;

#[derive(Clone, Copy, Hash, PartialEq, Eq, From, Into, Debug, Display, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TxId(pub u32);
