use std::sync::Arc;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use tracing::info;
use tracing::instrument;
use url::Url;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::urls;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::built_info::version;
use wallet_account::messages::instructions::ConfirmTransfer;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::config::UNIVERSAL_LINK_BASE_URL;
use crate::digid::DigidClient;
use crate::errors::ChangePinError;
use crate::errors::InstructionError;
use crate::errors::UpdatePolicyError;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::TransferData;
use crate::transfer::TransferSessionId;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum TransferError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,

    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,

    #[error("wallet is locked")]
    #[category(expected)]
    Locked,

    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),

    #[error("error fetching transfer data from storage: {0}")]
    Storage(#[from] StorageError),

    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),

    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),

    #[error("transfer_session_id not found in storage")]
    #[category(critical)]
    MissingTransferSessionId,
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
    #[sentry_capture_error]
    pub async fn start_transfer(&mut self) -> Result<Url, TransferError> {
        info!("Start transfer");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(TransferError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
            return Err(TransferError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(TransferError::Locked);
        }

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        let key = crypto::utils::random_bytes(32);
        // Safe to do it the simple way since it is encoded via Base64 URL safe (base64url)
        let query = format!(
            "s={}&k={}",
            BASE64_URL_SAFE_NO_PAD.encode(transfer_data.transfer_session_id.as_ref()),
            BASE64_URL_SAFE_NO_PAD.encode(&key),
        );

        let mut url: Url = urls::transfer_base_uri(&UNIVERSAL_LINK_BASE_URL).into_inner();
        url.set_fragment(Some(query.as_str()));

        Ok(url)
    }
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
    #[sentry_capture_error]
    pub async fn confirm_transfer(&mut self, transfer_session_id: TransferSessionId) -> Result<(), TransferError> {
        info!("Confirming transfer");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(TransferError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| TransferError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(TransferError::Locked);
        }

        let config = self.config_repository.get();
        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let instruction_client = self.new_hw_signed_instruction_client(
            Arc::clone(attested_key),
            registration_data.clone(),
            config.account_server.http_config.clone(),
            instruction_result_public_key,
        );

        instruction_client
            .send(ConfirmTransfer {
                transfer_session_id: transfer_session_id.into(),
                app_version: version().clone(),
            })
            .await
            .map_err(TransferError::Instruction)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use url::Host;
    use url::form_urlencoded;
    use uuid::Uuid;

    use wallet_account::messages::instructions::HwSignedInstruction;

    use crate::storage::InstructionData;
    use crate::wallet::test::create_wp_result;

    use super::super::test::TestWalletInMemoryStorage;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;

    #[tokio::test]
    async fn test_wallet_start_transfer() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let transfer_session_id = Uuid::new_v4();

        wallet
            .mut_storage()
            .expect_fetch_data::<TransferData>()
            .returning(move || {
                Ok(Some(TransferData {
                    transfer_session_id: transfer_session_id.into(),
                }))
            });

        let url = wallet
            .start_transfer()
            .await
            .expect("Wallet start transfer should have succeeded");

        assert_eq!(url.scheme(), "walletdebuginteraction");
        assert_eq!(
            url.host().map(|h| h.to_owned()),
            Some(Host::parse("wallet.edi.rijksoverheid.nl").unwrap())
        );
        assert_eq!(url.path(), "/transfer");
        assert_eq!(url.query(), None);
        assert!(url.fragment().is_some());

        let mut pairs = form_urlencoded::parse(url.fragment().unwrap().as_bytes());

        let (key, value) = pairs.next().unwrap();
        assert_eq!(key, "s");
        assert_eq!(
            BASE64_URL_SAFE_NO_PAD
                .decode(value.as_ref())
                .map(|id| Uuid::from_slice(id.as_ref())),
            Ok(Ok(transfer_session_id))
        );

        let (key, value) = pairs.next().unwrap();
        assert_eq!(key, "k");
        assert_eq!(BASE64_URL_SAFE_NO_PAD.decode(value.as_ref()).unwrap().len(), 32);

        assert!(pairs.next().is_none());
    }

    #[tokio::test]
    async fn test_wallet_start_transfer_error_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.update_policy_repository.state = VersionState::Block;

        let error = wallet
            .start_transfer()
            .await
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_wallet_start_transfer_error_not_unlocked() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.lock();

        let error = wallet
            .start_transfer()
            .await
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::Locked);
    }

    #[tokio::test]
    async fn test_wallet_start_transfer_error_not_registered() {
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        let error = wallet
            .start_transfer()
            .await
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::NotRegistered);
    }

    #[tokio::test]
    async fn test_wallet_confirm_transfer() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let transfer_session_id = Uuid::new_v4();

        wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<ConfirmTransfer>| Ok(wp_result));

        wallet
            .confirm_transfer(transfer_session_id.into())
            .await
            .expect("Wallet confirm transfer should have succeeded");
    }
}
