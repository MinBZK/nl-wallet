use std::sync::Arc;

use josekit::JoseError;
use josekit::jwk::KeyPair;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
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
use wallet_account::messages::instructions::CancelTransfer;
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
use crate::transfer::uri::TransferQuery;
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

    #[error("jose error: {0}")]
    #[category(critical)]
    JoseError(#[from] JoseError),

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
    pub async fn init_transfer(&mut self) -> Result<(EcKeyPair, Url), TransferError> {
        info!("Init transfer");

        self.validate_transfer_allowed()?;

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        let key_pair = EcKeyPair::generate(EcCurve::P256)?;

        let query = TransferQuery {
            session_id: transfer_data.transfer_session_id,
            public_key: key_pair.to_jwk_public_key(),
        };

        let url: Url = query.try_into()?;

        Ok((key_pair, url))
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn confirm_transfer(&mut self, uri: Url) -> Result<(), TransferError> {
        info!("Confirming transfer");

        self.validate_transfer_allowed()?;

        let transfer_query: TransferQuery = uri.try_into()?;

        self.storage
            .write()
            .await
            .insert_data(&TransferData {
                transfer_session_id: transfer_query.session_id,
            })
            .await?;

        self.send_transfer_instruction(ConfirmTransfer {
            transfer_session_id: transfer_query.session_id.into(),
            app_version: version().clone(),
        })
        .await
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_transfer(&mut self) -> Result<(), TransferError> {
        info!("Canceling transfer status");

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        self.send_transfer_instruction(CancelTransfer {
            transfer_session_id: transfer_data.transfer_session_id.into(),
        })
        .await
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_transfer_status(&mut self) -> Result<TransferSessionState, TransferError> {
        info!("Retrieving transfer status");

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        self.send_transfer_instruction(GetTransferStatus {
            transfer_session_id: transfer_data.transfer_session_id.into(),
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
    use url::Host;
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
            .expect_err("Wallet validate transfer should have resulted in error");

        assert_matches!(error, TransferError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_transfer_error_not_unlocked() {
        let mut wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.lock();

        let error = wallet
            .init_transfer()
            .await
            .expect_err("Wallet validate transfer should have resulted in error");

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
            .expect_err("Wallet validate transfer should have resulted in error");

        assert_matches!(error, TransferError::IllegalWalletState);
    }

    #[tokio::test]
    async fn test_init_transfer() {
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

        let (key_pair, url) = wallet
            .init_transfer()
            .await
            .expect("Wallet init transfer should have succeeded");

        assert_eq!(url.scheme(), "walletdebuginteraction");
        assert_eq!(
            url.host().map(|h| h.to_owned()),
            Some(Host::parse("wallet.edi.rijksoverheid.nl").unwrap())
        );
        assert_eq!(url.path(), "/transfer");
        assert_eq!(url.query(), None);
        assert!(url.fragment().is_some());

        let query: TransferQuery = serde_urlencoded::from_str(url.fragment().unwrap()).unwrap();
        assert_eq!(query.session_id, transfer_session_id.into());
        assert_eq!(query.public_key.key_type(), "EC");
        assert_eq!(query.public_key.curve(), Some("P-256"));
        assert_eq!(query.public_key, key_pair.to_jwk_public_key());
    }

    #[tokio::test]
    async fn test_confirm_transfer() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let transfer_session_id = Uuid::new_v4();

        wallet
            .mut_storage()
            .expect_insert_data::<TransferData>()
            .returning(|_| Ok(()));

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

        let key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();

        let transfer_uri = TransferQuery {
            session_id: transfer_session_id.into(),
            public_key: key_pair.to_jwk_public_key(),
        };

        wallet
            .confirm_transfer(transfer_uri.try_into().unwrap())
            .await
            .expect("Wallet confirm transfer should have succeeded");
    }

    #[tokio::test]
    async fn test_cancel_transfer() {
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
            .expect_fetch_data::<TransferData>()
            .returning(move || {
                Ok(Some(TransferData {
                    transfer_session_id: transfer_session_id.into(),
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
            .return_once(move |_, _: HwSignedInstruction<CancelTransfer>| Ok(wp_result));

        wallet
            .cancel_transfer()
            .await
            .expect("Wallet cancel transfer should have succeeded");
    }

    #[tokio::test]
    async fn test_get_transfer_status() {
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
            .expect_fetch_data::<TransferData>()
            .returning(move || {
                Ok(Some(TransferData {
                    transfer_session_id: transfer_session_id.into(),
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
            .get_transfer_status()
            .await
            .expect("Wallet get transfer status should have succeeded");

        assert_eq!(result, TransferSessionState::ReadyForTransfer)
    }

    #[tokio::test]
    async fn test_start_and_cancel_from_source_and_destination() {
        let mut destination_wallet =
            TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let mut source_wallet = TestWalletInMemoryStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let transfer_session_id = Uuid::new_v4();

        // First, init the transfer on the destination wallet

        destination_wallet
            .mut_storage()
            .insert_data(&TransferData {
                transfer_session_id: transfer_session_id.into(),
            })
            .await
            .unwrap();

        let (_, transfer_url) = destination_wallet
            .init_transfer()
            .await
            .expect("Wallet init transfer should have succeeded");

        // Then, confirm the transfer on the source wallet

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<ConfirmTransfer>| Ok(wp_result));

        source_wallet
            .confirm_transfer(transfer_url)
            .await
            .expect("Wallet confirm transfer should have succeeded");

        // Now both source and destination wallets should be able to request the session state

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(TransferSessionState::ReadyForTransfer);

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<GetTransferStatus>| Ok(wp_result));

        let result = destination_wallet
            .get_transfer_status()
            .await
            .expect("Wallet get transfer status should have succeeded");

        assert_eq!(result, TransferSessionState::ReadyForTransfer);

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(TransferSessionState::ReadyForTransfer);

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<GetTransferStatus>| Ok(wp_result));

        let result = source_wallet
            .get_transfer_status()
            .await
            .expect("Wallet get transfer status should have succeeded");

        assert_eq!(result, TransferSessionState::ReadyForTransfer);

        // And both can cancel the transfer

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<CancelTransfer>| Ok(wp_result));

        destination_wallet
            .cancel_transfer()
            .await
            .expect("Wallet cancel transfer should have succeeded");

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<CancelTransfer>| Ok(wp_result));

        source_wallet
            .cancel_transfer()
            .await
            .expect("Wallet cancel transfer should have succeeded");
    }
}
