pub enum WalletState {
    Ready,
    Transferring { role: WalletTransferRole },
}

pub enum WalletTransferRole {
    Source,
    Destination,
}

impl From<wallet::WalletState> for WalletState {
    fn from(source: wallet::WalletState) -> Self {
        match source {
            wallet::WalletState::Ready => WalletState::Ready,
            wallet::WalletState::Transferring { role } => WalletState::Transferring { role: role.into() },
        }
    }
}

impl From<wallet::WalletTransferRole> for WalletTransferRole {
    fn from(source: wallet::WalletTransferRole) -> Self {
        match source {
            wallet::WalletTransferRole::Source => WalletTransferRole::Source,
            wallet::WalletTransferRole::Destination => WalletTransferRole::Destination,
        }
    }
}
