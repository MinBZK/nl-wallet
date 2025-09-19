use derive_more::AsRef;
use derive_more::From;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;

use uuid::Uuid;

pub mod uri;

#[derive(Debug, Copy, Clone, PartialEq, From, Into, AsRef, Serialize, Deserialize)]
pub struct TransferSessionId(Uuid);
