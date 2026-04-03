use std::sync::Arc;

use chrono::Utc;
use itertools::Itertools;
use tracing::info;
use tracing::instrument;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::client::TlsPinningConfig;
use openid4vc::disclosure_session::DisclosureClient;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use utils::vec_at_least::VecNonEmpty;
use wallet_account::messages::instructions::DeleteKeys;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::errors::ChangePinError;
use crate::errors::InstructionError;
use crate::errors::StorageError;
use crate::instruction::InstructionClientParameters;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::update_policy::UpdatePolicyError;
use crate::wallet::HistoryError;
use crate::wallet::attestations::AttestationsError;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum DeleteAttestationError {
    // State errors
    #[error("app version is blocked")]
    #[category(expected)]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,

    // Errors from dependencies of this module
    #[error("error fetching update policy: {0}")]
    UpdatePolicy(#[from] UpdatePolicyError),
    #[error("error finalizing pin change: {0}")]
    ChangePin(#[from] ChangePinError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("error accessing wallet storage: {0}")]
    Storage(#[from] StorageError),
    #[error("error emitting attestations: {0}")]
    Attestations(#[from] AttestationsError),
    #[error("error emitting history: {0}")]
    History(#[from] HistoryError),

    // Errors specific to deleting attestations
    #[error("attestation not found")]
    #[category(critical)]
    AttestationNotFound,
    #[error("PID cannot be deleted")]
    #[category(critical)]
    CannotDeletePid,
    #[error("could not parse attestation id: {0}")]
    #[category(critical)]
    AttestationIdParsing(#[from] uuid::Error),
}

impl<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC> Wallet<CR, UR, S, AKH, APC, CID, DCC, CPC, SLC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    CID: IssuanceDiscovery,
    DCC: DisclosureClient,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn delete_attestation(
        &mut self,
        pin: String,
        attestation_id: String,
    ) -> Result<(), DeleteAttestationError> {
        info!("Deleting attestation {attestation_id}");

        let attestation_id = attestation_id.parse()?;

        let config = &self.config_repository.get();

        info!("Fetching update policy");
        self.update_policy_repository
            .fetch(&config.update_policy_server.http_config)
            .await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(DeleteAttestationError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or(DeleteAttestationError::NotRegistered)?;
        let attested_key = Arc::clone(attested_key);

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(DeleteAttestationError::Locked);
        }

        info!("Fetching key identifiers for attestation");
        let (attestation_type, key_identifiers) = self
            .storage
            .read()
            .await
            .fetch_type_and_key_identifiers_by_attestation_id(attestation_id)
            .await?
            .ok_or(DeleteAttestationError::AttestationNotFound)?;

        if config
            .pid_attributes
            .pid_attestation_types()
            .contains(attestation_type.as_str())
        {
            return Err(DeleteAttestationError::CannotDeletePid);
        }

        // No attestation is ever stored without corresponding private keys.
        let key_identifiers: VecNonEmpty<_> = key_identifiers.try_into().unwrap();

        let instruction_client = self
            .new_instruction_client(
                pin,
                attested_key,
                InstructionClientParameters::new(
                    registration_data.wallet_id.clone(),
                    registration_data.pin_salt.clone(),
                    registration_data.wallet_certificate.clone(),
                    config.account_server.http_config.clone(),
                    config.account_server.instruction_result_public_key.as_inner().into(),
                ),
            )
            .await?;

        info!("Sending DeleteKeys instruction to Wallet Provider");
        self.check_result_for_wallet_revocation(
            instruction_client
                .send(DeleteKeys {
                    identifiers: key_identifiers,
                })
                .await,
        )
        .await?;

        info!("Deleting attestation from local storage");
        self.storage
            .write()
            .await
            .delete_attestation(Utc::now(), attestation_id)
            .await?;

        self.emit_attestations().await?;
        self.emit_recent_history().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use mockall::predicate::always;
    use mockall::predicate::eq;
    use uuid::Uuid;

    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use update_policy_model::update_policy::VersionState;
    use wallet_account::messages::instructions::DeleteKeys;
    use wallet_account::messages::instructions::Instruction;

    use crate::storage::ChangePinData;
    use crate::storage::InstructionData;
    use crate::storage::StorageError;
    use crate::wallet::test::setup_mock_attestations_callback;
    use crate::wallet::test::setup_mock_recent_history_callback;

    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::create_wp_result;
    use super::*;

    const PIN: &str = "051097";

    fn setup_delete_attestation_mocks(
        wallet: &mut TestWalletMockStorage,
        attestation_id: Uuid,
        delete_result: Result<(), StorageError>,
    ) {
        wallet
            .mut_storage()
            .expect_fetch_type_and_key_identifiers_by_attestation_id()
            .with(eq(attestation_id))
            .return_once(|_| Ok(Some(("some_type".to_string(), vec!["test_key_id".to_string()]))));

        wallet
            .mut_storage()
            .expect_fetch_data::<ChangePinData>()
            .returning(|| Ok(None));

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
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .once()
            .return_once(|_, _: Instruction<DeleteKeys>| Ok(create_wp_result(())));

        wallet
            .mut_storage()
            .expect_delete_attestation()
            .with(always(), eq(attestation_id))
            .return_once(move |_, _| delete_result);
    }

    #[tokio::test]
    async fn test_delete_attestation_error_invalid_uuid() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let error = wallet
            .delete_attestation(PIN.to_string(), "not-a-valid-uuid".to_owned())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::AttestationIdParsing(_));
    }

    #[tokio::test]
    async fn test_delete_attestation_error_not_registered() {
        let mut wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        let error = wallet
            .delete_attestation(PIN.to_string(), Uuid::new_v4().to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::NotRegistered);
    }

    #[tokio::test]
    async fn test_delete_attestation_error_locked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.lock();

        let error = wallet
            .delete_attestation(PIN.to_string(), Uuid::new_v4().to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::Locked);
    }

    #[tokio::test]
    async fn test_delete_attestation_error_version_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        wallet.update_policy_repository.state = VersionState::Block;

        let error = wallet
            .delete_attestation(PIN.to_string(), Uuid::new_v4().to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_delete_attestation_error_attestation_not_found() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let attestation_id = Uuid::new_v4();

        wallet
            .mut_storage()
            .expect_fetch_type_and_key_identifiers_by_attestation_id()
            .with(eq(attestation_id))
            .return_once(|_| Ok(None));

        let error = wallet
            .delete_attestation(PIN.to_string(), attestation_id.to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::AttestationNotFound);
    }

    #[tokio::test]
    async fn test_delete_attestation_success() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Called by setup_mock_attestations_callback() and wallet.delete_attestation().
        wallet
            .mut_storage()
            .expect_fetch_unique_attestations()
            .times(2)
            .returning(|| Ok(vec![]));

        // Register an attestations callback and clear the initial emission.
        let attestations = setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("setting attestations callback should succeed");
        attestations.lock().clear();

        let attestation_id = Uuid::new_v4();

        setup_delete_attestation_mocks(&mut wallet, attestation_id, Ok(()));

        wallet
            .delete_attestation(PIN.to_string(), attestation_id.to_string())
            .await
            .expect("delete_attestation should succeed");

        // The attestations callback should have been called once with an empty list.
        let attestations = attestations.lock();
        assert_eq!(attestations.len(), 1);
        assert!(attestations[0].is_empty());
    }

    #[tokio::test]
    async fn test_delete_attestation_success_emits_history() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        // Called once by setup_mock_attestations_callback() and once by wallet.delete_attestation().
        wallet
            .mut_storage()
            .expect_fetch_unique_attestations()
            .times(2)
            .returning(|| Ok(vec![]));

        // Called once by setup_mock_recent_history_callback() and once by wallet.delete_attestation().
        wallet
            .mut_storage()
            .expect_fetch_recent_wallet_events()
            .times(2)
            .returning(|| Ok(vec![]));

        setup_mock_attestations_callback(&mut wallet)
            .await
            .expect("setting attestations callback should succeed");

        let history = setup_mock_recent_history_callback(&mut wallet)
            .await
            .expect("setting recent history callback should succeed");

        // The initial history emission from registration has occurred.
        assert_eq!(history.lock().len(), 1);

        let attestation_id = Uuid::new_v4();
        setup_delete_attestation_mocks(&mut wallet, attestation_id, Ok(()));

        wallet
            .delete_attestation(PIN.to_string(), attestation_id.to_string())
            .await
            .expect("delete_attestation should succeed");

        // Another history emission due to the deletion has occurred.
        assert_eq!(history.lock().len(), 2);
    }

    #[tokio::test]
    async fn test_delete_attestation_error_storage() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let attestation_id = Uuid::new_v4();

        setup_delete_attestation_mocks(&mut wallet, attestation_id, Err(StorageError::AlreadyOpened));

        let error = wallet
            .delete_attestation(PIN.to_string(), attestation_id.to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::Storage(_));
    }

    #[tokio::test]
    async fn test_delete_attestation_error_delete_pid() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;
        let attestation_id = Uuid::new_v4();

        wallet
            .mut_storage()
            .expect_fetch_type_and_key_identifiers_by_attestation_id()
            .with(eq(attestation_id))
            .return_once(|_| {
                Ok(Some((
                    PID_ATTESTATION_TYPE.to_string(),
                    vec!["test_key_id".to_string()],
                )))
            });

        let error = wallet
            .delete_attestation(PIN.to_string(), attestation_id.to_string())
            .await
            .expect_err("delete_attestation should have resulted in an error");

        assert_matches!(error, DeleteAttestationError::CannotDeletePid);
    }
}
