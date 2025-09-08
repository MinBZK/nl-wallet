use derive_more::From;
use derive_more::Into;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, From, Into)]
pub struct TransferSessionId(Uuid);
