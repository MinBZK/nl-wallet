pub enum TransferSessionState {
    Created,
    ReadyForTransfer,
    ReadyForDownload,
    Success,
    Cancelled,
    Error,
}

impl From<wallet::TransferSessionState> for TransferSessionState {
    fn from(value: wallet::TransferSessionState) -> Self {
        match value {
            wallet::TransferSessionState::Created => TransferSessionState::Created,
            wallet::TransferSessionState::ReadyForTransfer => TransferSessionState::ReadyForTransfer,
            wallet::TransferSessionState::ReadyForDownload => TransferSessionState::ReadyForDownload,
            wallet::TransferSessionState::Success => TransferSessionState::Success,
            wallet::TransferSessionState::Canceled => TransferSessionState::Cancelled,
            wallet::TransferSessionState::Error => TransferSessionState::Error,
        }
    }
}
