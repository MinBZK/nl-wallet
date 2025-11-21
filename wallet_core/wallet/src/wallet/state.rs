use std::sync::Arc;

use tracing::instrument;

use error_category::ErrorCategory;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::errors::StorageError;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::TransferData;
use crate::storage::TransferKeyData;
use crate::wallet::disclosure::DisclosureError;
use crate::wallet::issuance::IssuanceError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletStateError {
    #[error("error fetching data from storage: {0}")]
    Storage(#[from] StorageError),
    #[error("error checking for active issuance session: {0}")]
    Issuance(#[from] IssuanceError),
    #[error("error checking for active disclosure session: {0}")]
    Disclosure(#[from] DisclosureError),
}

pub enum WalletState {
    Ready,
    Registration,
    Empty,
    Locked { sub_state: Box<WalletState> },
    TransferPossible,
    Transferring { role: WalletTransferRole },
    Disclosure,
    Issuance { pid: bool },
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

impl<CR, UR, S, AKH, APC, DC, IS, DCC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    IS: IssuanceSession,
    DCC: DisclosureClient,
    APC: AccountProviderClient,
{
    #[instrument(skip_all)]
    pub async fn get_state(&self) -> Result<WalletState, WalletStateError> {
        if !self.has_registration() {
            return Ok(WalletState::Registration);
        }

        let is_empty = self.storage.read().await.fetch_unique_attestations().await?.is_empty();
        let sub_state = if is_empty {
            WalletState::Empty
        } else {
            WalletState::Ready
        };

        // TODO: Implement logic for other WalletStates, this is a temp. implementation to allow the app to start.
        if self.is_locked() {
            return Ok(WalletState::Locked {
                sub_state: Box::new(sub_state),
            });
        }

        if let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? {
            return Ok(transfer_data
                .key_data
                .map(|key_data| {
                    let role = match key_data {
                        TransferKeyData::Source { .. } => WalletTransferRole::Source,
                        TransferKeyData::Destination { .. } => WalletTransferRole::Destination,
                    };
                    WalletState::Transferring { role }
                })
                .unwrap_or(WalletState::TransferPossible));
        }

        if self.has_active_disclosure_session()? {
            return Ok(WalletState::Disclosure);
        }

        if self.has_active_issuance_session()? {
            return Ok(WalletState::Issuance { pid: false });
        }

        Ok(WalletState::Ready)
    }
}
