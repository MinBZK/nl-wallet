pub enum WalletState {
    Ready,
    Registration,
    Empty,
    Locked { sub_state: Box<WalletState> },
    TransferPossible,
    Transferring { role: WalletTransferRole },
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
            wallet::WalletState::Locked { sub_state } => WalletState::Locked {
                sub_state: Box::new((*sub_state).into()),
            },
            wallet::WalletState::TransferPossible => WalletState::TransferPossible,
            wallet::WalletState::Transferring { role } => WalletState::Transferring { role: role.into() },
            wallet::WalletState::Registration => WalletState::Registration,
            wallet::WalletState::Disclosure => WalletState::Disclosure,
            wallet::WalletState::Issuance => WalletState::Issuance,
            wallet::WalletState::PinChange => WalletState::PinChange,
            wallet::WalletState::PinRecovery => WalletState::PinRecovery,
            wallet::WalletState::WalletBlocked { reason } => WalletState::WalletBlocked { reason: reason.into() },
            wallet::WalletState::Empty => WalletState::Empty,
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
