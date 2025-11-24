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
use crate::pin::change::ChangePinStorage;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::TransferData;
use crate::storage::TransferKeyData;
use crate::wallet::Session;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletStateError {
    #[error("error fetching data from storage: {0}")]
    Storage(#[from] StorageError),
}

pub enum WalletState {
    WalletBlocked { reason: WalletBlockedReason },
    Registration,
    Empty,
    Locked { sub_state: Box<WalletState> },
    TransferPossible,
    Transferring { role: WalletTransferRole },
    Disclosure,
    Issuance,
    PinChange,
    PinRecovery,
    Ready,
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
        if self.is_blocked() {
            return Ok(WalletState::WalletBlocked {
                reason: WalletBlockedReason::RequiresAppUpdate,
            });
        }

        if !self.has_registration() {
            return Ok(WalletState::Registration);
        }

        let flow_state = self.get_flow_state().await?;

        if self.is_locked() {
            Ok(WalletState::Locked {
                sub_state: Box::new(flow_state),
            })
        } else {
            Ok(flow_state)
        }
    }

    async fn get_flow_state(&self) -> Result<WalletState, WalletStateError> {
        let read_storage = self.storage.read().await;

        let is_empty = read_storage.fetch_unique_attestations().await?.is_empty();

        if let Some(transfer_data) = read_storage.fetch_data::<TransferData>().await? {
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

        if let Some(session) = &self.session {
            return match session {
                Session::Digid(_) => Ok(WalletState::Issuance),
                Session::Issuance(_) => Ok(WalletState::Issuance),
                Session::Disclosure(_) => Ok(WalletState::Disclosure),
                Session::PinRecovery { .. } => Ok(WalletState::PinRecovery),
            };
        }
        if self.storage.get_change_pin_state().await?.is_some() {
            return Ok(WalletState::PinChange);
        }

        if is_empty {
            return Ok(WalletState::Empty);
        }

        Ok(WalletState::Ready)
    }
}
