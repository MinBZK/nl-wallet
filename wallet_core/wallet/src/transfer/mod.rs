use derive_more::AsRef;
use derive_more::From;
use derive_more::Into;
use josekit::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;
use serde_with::json::JsonString;
use serde_with::serde_as;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, From, Into, AsRef, Serialize, Deserialize)]
pub struct TransferSessionId(Uuid);

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferQuery {
    #[serde(rename = "s")]
    pub session_id: TransferSessionId,

    #[serde(rename = "k")]
    #[serde_as(as = "JsonString")]
    pub public_key: Jwk,
}
