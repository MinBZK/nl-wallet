use std::sync::Arc;

use tracing::info;
use tracing::instrument;
use url::Url;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::issuance_session::IssuanceSession;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::built_info::version;
use wallet_account::messages::instructions::ConfirmTransfer;
use wallet_account::messages::instructions::GetTransferStatus;
use wallet_account::messages::instructions::InstructionAndResult;
use wallet_account::messages::transfer::TransferSessionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::Wallet;
use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::errors::ChangePinError;
use crate::errors::InstructionError;
use crate::errors::UpdatePolicyError;
use crate::repository::Repository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::storage::TransferData;
use crate::transfer::TransferSessionId;
use crate::transfer::uri::TransferUri;
use crate::transfer::uri::TransferUriError;

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

    #[error("wallet is in an invalid state for a transfer")]
    #[category(critical)]
    IllegalWalletState,

    #[error("invalid transfer uri: {0}")]
    #[category(pd)]
    TransferUri(#[from] TransferUriError),
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
    pub async fn init_transfer(&mut self) -> Result<Url, TransferError> {
        info!("Init transfer");

        self.validate_transfer_allowed()?;

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        let transfer_uri = TransferUri {
            transfer_session_id: transfer_data.transfer_session_id,
            key: crypto::utils::random_bytes(32),
        };

        Ok(transfer_uri.try_into()?)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn confirm_transfer(&mut self, uri: Url) -> Result<(), TransferError> {
        info!("Confirming transfer");

        let transfer_uri: TransferUri = uri.try_into()?;

        self.send_transfer_instruction(ConfirmTransfer {
            transfer_session_id: transfer_uri.transfer_session_id.into(),
            app_version: version().clone(),
        })
        .await
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_transfer_status(
        &mut self,
        transfer_session_id: TransferSessionId,
    ) -> Result<TransferSessionState, TransferError> {
        info!("Retrieving transfer status");

        self.send_transfer_instruction(GetTransferStatus {
            transfer_session_id: transfer_session_id.into(),
        })
        .await
    }

    fn validate_transfer_allowed(&self) -> Result<(), TransferError> {
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

        info!("Checking if there is no active issuance or disclosure session");
        if self.session.is_some() {
            return Err(TransferError::IllegalWalletState);
        }

        Ok(())
    }

    async fn send_transfer_instruction<I>(&self, instruction: I) -> Result<I::Result, TransferError>
    where
        I: InstructionAndResult + 'static,
    {
        self.validate_transfer_allowed()?;

        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| TransferError::NotRegistered)?;

        let config = self.config_repository.get();
        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let instruction_client = self.new_hw_signed_instruction_client(
            Arc::clone(attested_key),
            registration_data.clone(),
            config.account_server.http_config.clone(),
            instruction_result_public_key,
        );

        let result = instruction_client
            .send(instruction)
            .await
            .map_err(TransferError::Instruction)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use uuid::Uuid;

    use wallet_account::messages::instructions::HwSignedInstruction;

    use crate::digid::MockDigidSession;
    use crate::storage::InstructionData;
    use crate::wallet::Session;
    use crate::wallet::test::create_wp_result;

    use super::super::test::TestWalletInMemoryStorage;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::*;

    #[tokio::test]
    async fn test_transfer_error_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.update_policy_repository.state = VersionState::Block;

        let error = wallet
            .init_transfer()
            .await
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_transfer_error_not_unlocked() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.lock();

        let error = wallet
            .init_transfer()
            .await
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::Locked);
    }

    #[tokio::test]
    async fn test_transfer_error_not_registered() {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        let error = wallet
            .validate_transfer_allowed()
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::NotRegistered);
    }

    #[tokio::test]
    async fn test_transfer_error_issuance_session_active() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.session = Some(Session::Digid(MockDigidSession::new()));

        let error = wallet
            .validate_transfer_allowed()
            .expect_err("Wallet start transfer should have resulted in error");

        assert_matches!(error, TransferError::IllegalWalletState);
    }

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
            .init_transfer()
            .await
            .expect("Wallet start transfer should have succeeded");

        let transfer_uri: TransferUri = url.try_into().expect("URL should be a transfer uri");

        assert_eq!(transfer_uri.transfer_session_id.as_ref(), &transfer_session_id);
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

        let transfer_uri = TransferUri {
            transfer_session_id: transfer_session_id.into(),
            key: crypto::utils::random_bytes(32),
        };

        wallet
            .confirm_transfer(transfer_uri.try_into().unwrap())
            .await
            .expect("Wallet confirm transfer should have succeeded");
    }

    #[tokio::test]
    async fn test_wallet_get_transfer_status() {
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

        let wp_result = create_wp_result(TransferSessionState::ReadyForTransfer);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<GetTransferStatus>| Ok(wp_result));

        let result = wallet
            .get_transfer_status(transfer_session_id.into())
            .await
            .expect("Wallet confirm transfer should have succeeded");

        assert_eq!(result, TransferSessionState::ReadyForTransfer)
    }
}
