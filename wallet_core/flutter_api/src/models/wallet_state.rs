pub enum WalletState {
    Ready,
    Locked,
    TransferPossible,
    Transferring { role: WalletTransferRole },
    Registration { has_pin: bool },
    Disclosure,
    Issuance,
    PinChange,
    PinRecovery,
    WalletBlocked { reason: WalletBlockedReason },
}

pub enum WalletBlockedReason {
    RequiresAppUpdate,
    BlockedByWalletProvider,
}

pub enum WalletTransferRole {
    Source,
    Destination,
}

impl From<wallet::WalletState> for WalletState {
    fn from(source: wallet::WalletState) -> Self {
        match source {
            wallet::WalletState::Ready => WalletState::Ready,
            wallet::WalletState::Locked => WalletState::Locked,
            wallet::WalletState::TransferPossible => WalletState::TransferPossible,
            wallet::WalletState::Transferring { role } => WalletState::Transferring { role: role.into() },
            wallet::WalletState::Registration { has_pin } => WalletState::Registration { has_pin: has_pin },
            wallet::WalletState::Disclosure => WalletState::Disclosure,
            wallet::WalletState::Issuance => WalletState::Issuance,
            wallet::WalletState::PinChange => WalletState::PinChange,
            wallet::WalletState::PinRecovery => WalletState::PinRecovery,
            wallet::WalletState::WalletBlocked { reason } => WalletState::WalletBlocked { reason: reason.into() },
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
impl From<wallet::WalletBlockedReason> for WalletBlockedReason {
    fn from(source: wallet::WalletBlockedReason) -> Self {
        match source {
            wallet::WalletBlockedReason::RequiresAppUpdate => WalletBlockedReason::RequiresAppUpdate,
            wallet::WalletBlockedReason::BlockedByWalletProvider => WalletBlockedReason::BlockedByWalletProvider,
        }
    }
}
