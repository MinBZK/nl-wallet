use std::sync::Arc;

use josekit::JoseError;
use josekit::jwk::KeyPair;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use tracing::info;
use tracing::instrument;
use url::Url;
use uuid::Uuid;

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
use wallet_account::messages::instructions::ReceiveWalletPayload;
use wallet_account::messages::instructions::SendWalletPayload;
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
use crate::storage::TransferKeyData;
use crate::transfer::database_payload::DatabasePayloadError;
use crate::transfer::database_payload::WalletDatabasePayload;
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

    #[error("transfer public key not found in storage")]
    #[category(critical)]
    MissingTransferPublicKey,

    #[error("wallet is in an invalid state for a transfer")]
    #[category(critical)]
    IllegalWalletState,

    #[error("error generating transfer key pair: {0}")]
    #[category(critical)]
    TransferKeyPairGeneration(#[from] JoseError),

    #[error("database payload error: {0}")]
    #[category(pd)]
    DatabasePayload(#[from] DatabasePayloadError),

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

        let Some(mut transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        let key_pair = EcKeyPair::generate(EcCurve::P256)?;

        transfer_data.key_data = Some(TransferKeyData::Destination {
            private_key: key_pair.to_jwk_private_key(),
        });
        self.storage.write().await.upsert_data(&transfer_data).await?;

        let query = TransferQuery {
            session_id: transfer_data.transfer_session_id,
            public_key: key_pair.to_jwk_public_key(),
        };

        let url: Url = query.try_into()?;

        Ok(url)
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
                key_data: Some(TransferKeyData::Source {
                    public_key: transfer_query.public_key,
                }),
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

        let status = self
            .send_transfer_instruction(GetTransferStatus {
                transfer_session_id: transfer_data.transfer_session_id.into(),
            })
            .await?;

        if status == TransferSessionState::ReadyForDownload {
            self.receive_wallet_payload(transfer_data).await?;

            // TODO: PVW-4599: send complete_transfer instruction

            return Ok(TransferSessionState::Success);
        }

        Ok(status)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn send_wallet_payload(&mut self, pin: String) -> Result<(), TransferError> {
        info!("Send wallet payload");

        self.validate_transfer_allowed()?;

        let Some(transfer_data) = self.storage.read().await.fetch_data::<TransferData>().await? else {
            return Err(TransferError::MissingTransferSessionId);
        };

        let Some(TransferKeyData::Source { public_key }) = transfer_data.key_data else {
            return Err(TransferError::MissingTransferPublicKey);
        };

        let database_export = self.storage.write().await.export().await?;
        let database_payload = WalletDatabasePayload::new(database_export);

        let transfer_session_id: Uuid = transfer_data.transfer_session_id.into();

        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| TransferError::NotRegistered)?;

        let config = self.config_repository.get();
        let instruction_result_public_key = config.account_server.instruction_result_public_key.as_inner().into();

        let remote_instruction = self
            .new_instruction_client(
                pin,
                Arc::clone(attested_key),
                registration_data.clone(),
                config.account_server.http_config.clone(),
                instruction_result_public_key,
            )
            .await?;

        remote_instruction
            .send(SendWalletPayload {
                transfer_session_id,
                payload: database_payload.encrypt(&public_key)?,
            })
            .await
            .map_err(TransferError::Instruction)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn receive_wallet_payload(&mut self, transfer_data: TransferData) -> Result<(), TransferError> {
        let Some(TransferKeyData::Destination { private_key }) = transfer_data.key_data else {
            return Err(TransferError::IllegalWalletState);
        };

        let result = self
            .send_transfer_instruction(ReceiveWalletPayload {
                transfer_session_id: transfer_data.transfer_session_id.into(),
            })
            .await?;

        let database_payload = WalletDatabasePayload::decrypt(&result.payload, &private_key)?;
        self.storage.write().await.import(database_payload.into()).await?;

        Ok(())
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
    use crypto::utils::random_bytes;
    use josekit::jwk::Jwk;
    use parking_lot::Mutex;
    use url::Host;
    use uuid::Uuid;

    use wallet_account::messages::instructions::HwSignedInstruction;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::ReceiveWalletPayloadResult;

    use crate::digid::MockDigidSession;
    use crate::storage::ChangePinData;
    use crate::storage::DatabaseExport;
    use crate::storage::InstructionData;
    use crate::storage::test::SqlCipherKey;
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
                    key_data: None,
                }))
            });

        wallet
            .mut_storage()
            .expect_upsert_data::<TransferData>()
            .returning(|_| Ok(()));

        let url = wallet
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
            .returning(|_, _| Ok(random_bytes(32)));

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
                    key_data: None,
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
            .returning(|_, _| Ok(random_bytes(32)));

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
                    key_data: None,
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
            .returning(|_, _| Ok(random_bytes(32)));

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
                key_data: None,
            })
            .await
            .unwrap();

        let transfer_url = destination_wallet
            .init_transfer()
            .await
            .expect("Wallet init transfer should have succeeded");

        // Then, confirm the transfer on the source wallet

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(random_bytes(32)));

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
            .returning(|_, _| Ok(random_bytes(32)));

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
            .returning(|_, _| Ok(random_bytes(32)));

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
            .returning(|_, _| Ok(random_bytes(32)));

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
            .returning(|_, _| Ok(random_bytes(32)));

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

    #[tokio::test]
    async fn test_send_wallet_payload() {
        let mut destination_wallet =
            TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let mut source_wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let transfer_session_id = Uuid::new_v4();

        // Init the transfer on the destination wallet

        destination_wallet
            .mut_storage()
            .expect_fetch_data::<TransferData>()
            .returning(move || {
                Ok(Some(TransferData {
                    transfer_session_id: transfer_session_id.into(),
                    key_data: None,
                }))
            });

        let private_key_param: Arc<Mutex<Option<Jwk>>> = Arc::new(Mutex::new(None));
        let private_key_param_clone = Arc::clone(&private_key_param);

        destination_wallet
            .mut_storage()
            .expect_upsert_data::<TransferData>()
            .withf(move |transfer_data| {
                if let Some(TransferKeyData::Destination { private_key }) = &transfer_data.key_data {
                    private_key_param_clone.lock().replace(private_key.clone());
                }

                true
            })
            .returning(|_| Ok(()));

        let transfer_url = destination_wallet
            .init_transfer()
            .await
            .expect("Wallet init transfer should have succeeded");

        // Then, confirm the transfer on the source wallet

        source_wallet
            .mut_storage()
            .expect_insert_data::<TransferData>()
            .returning(|_| Ok(()));

        source_wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        source_wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(random_bytes(32)));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<ConfirmTransfer>| Ok(wp_result));

        source_wallet
            .confirm_transfer(transfer_url.clone())
            .await
            .expect("Wallet confirm transfer should have succeeded");

        // Send the wallet payload from the source

        let transfer_query: TransferQuery = transfer_url.try_into().unwrap();
        let public_key = transfer_query.public_key;
        source_wallet
            .mut_storage()
            .expect_fetch_data::<TransferData>()
            .returning(move || {
                Ok(Some(TransferData {
                    transfer_session_id: transfer_session_id.into(),
                    key_data: Some(TransferKeyData::Source {
                        public_key: public_key.clone(),
                    }),
                }))
            });

        let database_export_bytes = random_bytes(32);
        let database_export_key = SqlCipherKey::new_random_with_salt();
        let expected_database_export = DatabaseExport::new(database_export_key, database_export_bytes.clone());
        source_wallet
            .mut_storage()
            .expect_export()
            .returning(move || Ok(DatabaseExport::new(database_export_key, database_export_bytes.clone())));

        source_wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

        source_wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        source_wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(random_bytes(32)));

        let wp_result = create_wp_result(());

        let payload_param: Arc<Mutex<Option<SendWalletPayload>>> = Arc::new(Mutex::new(None));
        let payload_param_clone = Arc::clone(&payload_param);

        Arc::get_mut(&mut source_wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .withf(move |_, instruction| {
                payload_param_clone
                    .lock()
                    .replace(instruction.instruction.dangerous_parse_unverified().unwrap().payload);
                true
            })
            .once()
            .return_once(move |_, _: Instruction<SendWalletPayload>| Ok(wp_result));

        source_wallet
            .send_wallet_payload(String::from("12345"))
            .await
            .expect("Wallet send payload should have succeeded");

        // Receive payload on the destination
        destination_wallet
            .mut_storage()
            .expect_fetch_data::<InstructionData>()
            .returning(|| {
                Ok(Some(InstructionData {
                    instruction_sequence_number: 0,
                }))
            });

        destination_wallet
            .mut_storage()
            .expect_upsert_data::<InstructionData>()
            .returning(|_| Ok(()));

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .once()
            .returning(|_, _| Ok(random_bytes(32)));

        let payload = payload_param.lock().as_ref().unwrap().payload.clone();

        let wp_result = create_wp_result(ReceiveWalletPayloadResult { payload });

        Arc::get_mut(&mut destination_wallet.account_provider_client)
            .unwrap()
            .expect_hw_signed_instruction()
            .once()
            .return_once(move |_, _: HwSignedInstruction<ReceiveWalletPayload>| Ok(wp_result));

        destination_wallet.mut_storage().expect_import().returning(|_| Ok(()));

        let private_key = private_key_param.lock().as_ref().unwrap().clone();
        destination_wallet
            .receive_wallet_payload(TransferData {
                transfer_session_id: transfer_session_id.into(),
                key_data: Some(TransferKeyData::Destination { private_key }),
            })
            .await
            .expect("Wallet receive payload should have succeeded");

        let send_wallet_payload_instruction = payload_param.lock();
        let payload = send_wallet_payload_instruction.as_ref().unwrap();
        assert_eq!(transfer_session_id, payload.transfer_session_id);

        let decrypted_database_export =
            WalletDatabasePayload::decrypt(payload.payload.as_str(), private_key_param.lock().as_ref().unwrap())
                .unwrap();

        assert!(decrypted_database_export.as_ref() == &expected_database_export)
    }
}
