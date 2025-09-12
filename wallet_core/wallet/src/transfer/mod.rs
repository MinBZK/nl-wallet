use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use derive_more::Into;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, From, Into, AsRef, Serialize, Deserialize, FromStr, Display)]
pub struct TransferSessionId(Uuid);
