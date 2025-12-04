pub enum RevocationStatus {
    Valid,
    Revoked,
    Undetermined,
    Corrupted,
}

impl From<wallet::RevocationStatus> for RevocationStatus {
    fn from(value: wallet::RevocationStatus) -> Self {
        match value {
            wallet::RevocationStatus::Valid => RevocationStatus::Valid,
            wallet::RevocationStatus::Revoked => RevocationStatus::Revoked,
            wallet::RevocationStatus::Undetermined => RevocationStatus::Undetermined,
            wallet::RevocationStatus::Corrupted => RevocationStatus::Corrupted,
        }
    }
}
