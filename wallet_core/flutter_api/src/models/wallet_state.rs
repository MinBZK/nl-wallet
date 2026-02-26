pub enum WalletState {
    Blocked {
        reason: BlockedReason,
        can_register_new_account: bool,
    },
    Unregistered,
    Locked {
        sub_state: Box<WalletState>,
    },
    // The following variants may appear in `Locked { sub_state }`
    Empty,
    TransferPossible,
    Transferring {
        role: TransferRole,
    },
    InDisclosureFlow,
    InIssuanceFlow,
    InPinChangeFlow,
    InPinRecoveryFlow,
    Ready,
}

pub enum BlockedReason {
    RequiresAppUpdate,
    BlockedByWalletProvider,
}

pub enum TransferRole {
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
            wallet::WalletState::Unregistered => WalletState::Unregistered,
            wallet::WalletState::InDisclosureFlow => WalletState::InDisclosureFlow,
            wallet::WalletState::InIssuanceFlow => WalletState::InIssuanceFlow,
            wallet::WalletState::InPinChangeFlow => WalletState::InPinChangeFlow,
            wallet::WalletState::InPinRecoveryFlow => WalletState::InPinRecoveryFlow,
            wallet::WalletState::Blocked {
                reason,
                can_register_new_account,
            } => WalletState::Blocked {
                reason: reason.into(),
                can_register_new_account,
            },
            wallet::WalletState::Empty => WalletState::Empty,
        }
    }
}

impl From<wallet::TransferRole> for TransferRole {
    fn from(source: wallet::TransferRole) -> Self {
        match source {
            wallet::TransferRole::Source => TransferRole::Source,
            wallet::TransferRole::Destination => TransferRole::Destination,
        }
    }
}
impl From<wallet::BlockedReason> for BlockedReason {
    fn from(source: wallet::BlockedReason) -> Self {
        match source {
            wallet::BlockedReason::RequiresAppUpdate => BlockedReason::RequiresAppUpdate,
            wallet::BlockedReason::BlockedByWalletProvider => BlockedReason::BlockedByWalletProvider,
        }
    }
}
