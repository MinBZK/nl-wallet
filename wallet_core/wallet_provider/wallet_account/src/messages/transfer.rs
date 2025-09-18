use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TransferSessionState {
    Created,
    ReadyForTransfer,
    ReadyForDownload,
    Success,
    Cancelled,
    Error,
}
