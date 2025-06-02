mod instruction;
mod storage;

use std::sync::Arc;

use tracing::info;

use http_utils::tls::pinning::TlsPinningConfig;
use platform_support::attested_key::AttestedKeyHolder;
use update_policy_model::update_policy::VersionState;
use wallet_configuration::wallet_config::WalletConfiguration;

use crate::account_provider::AccountProviderClient;
use crate::errors::UpdatePolicyError;
use crate::instruction::InstructionClientFactory;
use crate::pin::change::BeginChangePinOperation;
use crate::pin::change::ChangePinError;
use crate::pin::change::FinishChangePinOperation;
use crate::repository::Repository;
use crate::repository::UpdateableRepository;
use crate::storage::Storage;
use crate::Wallet;

use super::WalletRegistration;

const CHANGE_PIN_RETRIES: u8 = 3;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    CR: Repository<Arc<WalletConfiguration>>,
    UR: Repository<VersionState>,
    S: Storage,
    AKH: AttestedKeyHolder,
    APC: AccountProviderClient,
    WIC: Default,
{
    pub async fn begin_change_pin(&mut self, old_pin: String, new_pin: String) -> Result<(), ChangePinError>
    where
        UR: UpdateableRepository<VersionState, TlsPinningConfig, Error = UpdatePolicyError>,
    {
        info!("Begin PIN change");

        let config = &self.config_repository.get().update_policy_server;

        info!("Fetching update policy");
        self.update_policy_repository.fetch(config.http_config.clone()).await?;

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(ChangePinError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = match &mut self.registration {
            WalletRegistration::Registered { attested_key, data } => (attested_key, data),
            WalletRegistration::Unregistered | WalletRegistration::KeyIdentifierGenerated(_) => {
                return Err(ChangePinError::NotRegistered)
            }
        };

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(ChangePinError::Locked);
        }

        let config = &self.config_repository.get().account_server;
        let instruction_result_public_key = config.instruction_result_public_key.as_inner().into();
        let certificate_public_key = config.certificate_public_key.as_inner();

        // Extract the public key belonging to the hardware attested key from the current certificate.
        let hw_pubkey = registration_data
            .wallet_certificate
            .parse_and_verify_with_sub(&certificate_public_key.into())
            .expect("stored wallet certificate should be valid")
            .hw_pubkey
            .into_inner();

        let instruction_client = InstructionClientFactory::new(
            Arc::clone(&self.storage),
            Arc::clone(attested_key),
            Arc::clone(&self.account_provider_client),
            registration_data.clone(),
            config.http_config.clone(),
            instruction_result_public_key,
        );

        let session = BeginChangePinOperation::new(
            &instruction_client,
            &self.storage,
            registration_data,
            certificate_public_key,
            &hw_pubkey,
        );
        let (new_pin_salt, new_wallet_certificate) = session.begin_change_pin(old_pin, new_pin).await?;

        registration_data.pin_salt = new_pin_salt;
        registration_data.wallet_certificate = new_wallet_certificate;

        info!("PIN change started");

        Ok(())
    }

    pub async fn continue_change_pin(&self, pin: &str) -> Result<(), ChangePinError> {
        info!("Continue PIN change");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(ChangePinError::VersionBlocked);
        }

        info!("Checking if registered");
        let (attested_key, registration_data) = self
            .registration
            .as_key_and_registration_data()
            .ok_or_else(|| ChangePinError::NotRegistered)?;

        // Wallet does not need to be unlocked, see [`Wallet::unlock`].

        let config = &self.config_repository.get().account_server;
        let instruction_result_public_key = config.instruction_result_public_key.as_inner().into();

        let instruction_client = InstructionClientFactory::new(
            Arc::clone(&self.storage),
            Arc::clone(attested_key),
            Arc::clone(&self.account_provider_client),
            registration_data.clone(),
            config.http_config.clone(),
            instruction_result_public_key,
        );

        let session = FinishChangePinOperation::new(&instruction_client, &self.storage, CHANGE_PIN_RETRIES);

        session.finish_change_pin(pin).await?;

        info!("PIN change successfully finalized");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use jwt::Jwt;
    use platform_support::attested_key::AttestedKey;
    use wallet_account::messages::instructions::ChangePinCommit;
    use wallet_account::messages::instructions::ChangePinStart;
    use wallet_account::messages::instructions::Instruction;
    use wallet_account::messages::instructions::InstructionResultClaims;

    use crate::pin::change::ChangePinStorage;
    use crate::pin::change::State;
    use crate::wallet::test::WalletDeviceVendor;
    use crate::wallet::test::WalletWithMocks;
    use crate::wallet::test::ACCOUNT_SERVER_KEYS;

    fn create_wp_result<T>(result: T) -> Jwt<InstructionResultClaims<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let result_claims = InstructionResultClaims {
            result,
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        Jwt::sign_with_sub(&result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
            .now_or_never()
            .unwrap()
            .expect("could not sign instruction result")
    }

    #[tokio::test]
    async fn test_wallet_begin_and_continue_change_pin() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction_challenge()
            .times(2)
            .returning(|_, _| Ok(crypto::utils::random_bytes(32)));

        let (attested_key, registration_data) = wallet.registration.as_key_and_registration_data().unwrap();
        let AttestedKey::Apple(attested_key) = attested_key.as_ref() else {
            unreachable!();
        };

        let wp_result = create_wp_result(WalletWithMocks::valid_certificate(
            Some(registration_data.wallet_id.clone()),
            *attested_key.verifying_key(),
        ));

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .times(1)
            .return_once(|_, _: Instruction<ChangePinStart>| Ok(wp_result));

        let actual = wallet
            .begin_change_pin("123456".to_string(), "111122".to_string())
            .await;
        assert_matches!(actual, Ok(()));

        let change_pin_state = wallet
            .storage
            .get_change_pin_state()
            .await
            .expect("could not read change_pin_state");
        assert_eq!(change_pin_state, Some(State::Commit));

        let wp_result = create_wp_result(());

        Arc::get_mut(&mut wallet.account_provider_client)
            .unwrap()
            .expect_instruction()
            .times(1)
            .return_once(|_, _: Instruction<ChangePinCommit>| Ok(wp_result));

        let actual = wallet.continue_change_pin("111122").await;
        assert_matches!(actual, Ok(()));

        let change_pin_state = wallet
            .storage
            .get_change_pin_state()
            .await
            .expect("could not read change_pin_state");
        assert_eq!(change_pin_state, None);
    }
}
