use std::sync::Arc;

use tracing::info;
use tracing::instrument;

use error_category::ErrorCategory;
use error_category::sentry_capture_error;
use http_utils::tls::pinning::TlsPinningConfig;
use openid4vc::disclosure_session::DisclosureClient;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_account::RevocationCode;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::digid::DigidClient;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::storage::StorageError;
use crate::update_policy::UpdatePolicyError;

use super::Wallet;
use super::WalletRegistration;
use super::issuance::PidAttestationFormat;
use super::lock::WalletUnlockError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum RevocationCodeError {
    #[error("app version is blocked")]
    #[category(expected)]
    VersionBlocked,

    #[error("wallet is not registered, no revocation code present")]
    #[category(expected)]
    NotRegistered,

    #[error("could not read pid from database: {0}")]
    PidRetrieval(#[from] StorageError),

    #[error("PID attestation is already present, revocation code is protected")]
    #[category(expected)]
    PidPresent,

    #[error("could not verify PIN: {0}")]
    Unlock(#[from] WalletUnlockError),
}

impl<CR, UR, S, AKH, APC, DC, IS, DCC, SLC> Wallet<CR, UR, S, AKH, APC, DC, IS, DCC, SLC>
where
    AKH: AttestedKeyHolder,
    DC: DigidClient,
    DCC: DisclosureClient,
{
    fn revocation_code(&self) -> Option<&RevocationCode> {
        match &self.registration {
            WalletRegistration::Registered { data, .. } => Some(&data.revocation_code),
            _ => None,
        }
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_revocation_code_before_pid(&self) -> Result<&RevocationCode, RevocationCodeError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        S: Storage,
        UR: Repository<VersionState>,
    {
        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(RevocationCodeError::VersionBlocked);
        }

        info!("Checking if registered");
        let revocation_code = self.revocation_code().ok_or(RevocationCodeError::NotRegistered)?;

        info!("Checking if a PID is already present");

        let has_pid = self
            .has_pid(
                &self.config_repository.get().pid_attributes,
                PidAttestationFormat::Either,
            )
            .await?;

        if has_pid {
            return Err(RevocationCodeError::PidPresent);
        }

        Ok(revocation_code)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_revocation_code_with_pin(&mut self, pin: String) -> Result<&RevocationCode, RevocationCodeError>
    where
        CR: Repository<Arc<WalletConfiguration>>,
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
        S: Storage,
        APC: AccountProviderClient,
    {
        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(RevocationCodeError::VersionBlocked);
        }

        self.send_check_pin_instruction(pin).await?;

        let revocation_code = self
            .revocation_code()
            .expect("revocation code should be present if checking PIN succeeds");

        Ok(revocation_code)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use assert_matches::assert_matches;

    use crypto::utils::random_bytes;
    use update_policy_model::update_policy::VersionState;
    use wallet_account::messages::errors::AccountError;
    use wallet_account::messages::errors::IncorrectPinData;
    use wallet_account::messages::instructions::CheckPin;
    use wallet_account::messages::instructions::Instruction;

    use crate::account_provider::AccountProviderError;
    use crate::account_provider::AccountProviderResponseError;
    use crate::instruction::InstructionError;
    use crate::storage::ChangePinData;
    use crate::storage::InstructionData;

    use super::super::lock::WalletUnlockError;
    use super::super::test::TestWalletMockStorage;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::create_wp_result;
    use super::RevocationCodeError;

    const PIN: &str = "293847";

    #[tokio::test]
    async fn test_wallet_get_revocation_code_before_pid() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .return_once(|_| Ok(false));

        let _ = wallet
            .get_revocation_code_before_pid()
            .await
            .expect("retrieving revocation code before PID issuance should succeed");
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_before_pid_error_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.update_policy_repository.state = VersionState::Block;

        let error = wallet
            .get_revocation_code_before_pid()
            .await
            .expect_err("retrieving revocation code before PID issuance should not succeed when the wallet is blocked");

        assert_matches!(error, RevocationCodeError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_before_pid_not_registered() {
        let wallet = TestWalletMockStorage::new_unregistered(WalletDeviceVendor::Apple).await;

        let error = wallet.get_revocation_code_before_pid().await.expect_err(
            "retrieving revocation code before PID issuance should not succeed when the wallet is not registered",
        );

        assert_matches!(error, RevocationCodeError::NotRegistered);
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_before_pid_error_pid_present() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet
            .mut_storage()
            .expect_has_any_attestations_with_types()
            .return_once(|_| Ok(true));

        let error = wallet
            .get_revocation_code_before_pid()
            .await
            .expect_err("retrieving revocation code before PID issuance should not succeed when the wallet has a PID");

        assert_matches!(error, RevocationCodeError::PidPresent);
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_with_pin() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let storage = wallet.mut_storage();

        storage.expect_fetch_data::<ChangePinData>().return_once(|| Ok(None));

        storage.expect_fetch_data::<InstructionData>().returning(|| {
            Ok(Some(InstructionData {
                instruction_sequence_number: 0,
            }))
        });

        storage.expect_upsert_data::<InstructionData>().returning(|_| Ok(()));

        let account_provider_client = Arc::get_mut(&mut wallet.account_provider_client).unwrap();

        account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Ok(random_bytes(32)));

        account_provider_client
            .expect_instruction()
            .return_once(|_, _: Instruction<CheckPin>| Ok(create_wp_result(())));

        let _ = wallet
            .get_revocation_code_with_pin(PIN.to_string())
            .await
            .expect("retrieving revocation code using PIN should succeed");
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_with_pin_error_blocked() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        wallet.update_policy_repository.state = VersionState::Block;

        let error = wallet
            .get_revocation_code_with_pin(PIN.to_string())
            .await
            .expect_err("retrieving revocation code using PIN should not succeed when the wallet is blocked");

        assert_matches!(error, RevocationCodeError::VersionBlocked);
    }

    #[tokio::test]
    async fn test_wallet_get_revocation_code_with_pin_error_unlock() {
        let mut wallet = TestWalletMockStorage::new_registered_and_unlocked(WalletDeviceVendor::Apple).await;

        let storage = wallet.mut_storage();

        storage.expect_fetch_data::<ChangePinData>().return_once(|| Ok(None));

        storage.expect_fetch_data::<InstructionData>().returning(|| {
            Ok(Some(InstructionData {
                instruction_sequence_number: 0,
            }))
        });

        storage.expect_upsert_data::<InstructionData>().returning(|_| Ok(()));

        let account_provider_client = Arc::get_mut(&mut wallet.account_provider_client).unwrap();

        account_provider_client
            .expect_instruction_challenge()
            .return_once(|_, _| Ok(random_bytes(32)));

        account_provider_client
            .expect_instruction()
            .return_once(|_, _: Instruction<CheckPin>| {
                Err(AccountProviderError::Response(AccountProviderResponseError::Account(
                    AccountError::IncorrectPin(IncorrectPinData {
                        attempts_left_in_round: 2,
                        is_final_round: false,
                    }),
                    None,
                )))
            });

        let error = wallet
            .get_revocation_code_with_pin(PIN.to_string())
            .await
            .expect_err("retrieving revocation code using PIN should not succeed when using an incorrect PIN");

        assert_matches!(
            error,
            RevocationCodeError::Unlock(WalletUnlockError::Instruction(InstructionError::IncorrectPin { .. }))
        );
    }
}
