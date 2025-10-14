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

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WalletStateError {
    #[error("error fetching data from storage: {0}")]
    Storage(#[from] StorageError),
}

pub enum WalletState {
    Ready,
    Transferring { role: WalletTransferRole },
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
                .unwrap_or(WalletState::Ready));
        }

        Ok(WalletState::Ready)
    }
}
