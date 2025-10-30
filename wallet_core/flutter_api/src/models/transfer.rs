pub enum TransferSessionState {
    Created,
    Paired,
    Confirmed,
    Uploaded,
    Success,
    Canceled,
    Error,
}

impl From<wallet::TransferSessionState> for TransferSessionState {
    fn from(value: wallet::TransferSessionState) -> Self {
        match value {
            wallet::TransferSessionState::Created => TransferSessionState::Created,
            wallet::TransferSessionState::Paired => TransferSessionState::Paired,
            wallet::TransferSessionState::Confirmed => TransferSessionState::Confirmed,
            wallet::TransferSessionState::Uploaded => TransferSessionState::Uploaded,
            wallet::TransferSessionState::Success => TransferSessionState::Success,
            wallet::TransferSessionState::Canceled => TransferSessionState::Canceled,
            wallet::TransferSessionState::Error => TransferSessionState::Error,
        }
    }
}
