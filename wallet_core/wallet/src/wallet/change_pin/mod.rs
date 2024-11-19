mod instruction;
mod storage;

use tracing::info;

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::serialization::DerVerifyingKey;

use crate::account_provider::AccountProviderClient;
use crate::config::ConfigurationRepository;
use crate::instruction::InstructionClientFactory;
use crate::pin::change::BeginChangePinOperation;
use crate::pin::change::ChangePinError;
use crate::pin::change::FinishChangePinOperation;
use crate::storage::Storage;
use crate::Wallet;

const CHANGE_PIN_RETRIES: u8 = 3;

impl<CR, S, PEK, APC, DS, IC, MDS, WIC> Wallet<CR, S, PEK, APC, DS, IC, MDS, WIC>
where
    CR: ConfigurationRepository,
    S: Storage,
    PEK: PlatformEcdsaKey,
    APC: AccountProviderClient,
    WIC: Default,
{
    pub async fn begin_change_pin(&mut self, old_pin: String, new_pin: String) -> Result<(), ChangePinError> {
        info!("Begin PIN change");

        info!("Checking if registered");
        let registration = self
            .registration
            .as_mut()
            .ok_or_else(|| ChangePinError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(ChangePinError::Locked);
        }

        let config = &self.config_repository.config().account_server;
        let DerVerifyingKey(instruction_result_public_key) = &config.instruction_result_public_key;
        let instruction_result_public_key = instruction_result_public_key.into();
        let DerVerifyingKey(certificate_public_key) = &config.certificate_public_key;

        let hw_pubkey = registration
            .hw_privkey
            .verifying_key()
            .await
            .map_err(|e| ChangePinError::HardwarePublicKey(e.into()))?;

        let instruction_client = InstructionClientFactory::new(
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            &config.base_url,
            &instruction_result_public_key,
        );

        let session = BeginChangePinOperation::new(
            &instruction_client,
            &self.storage,
            &registration.data.wallet_id,
            certificate_public_key,
            &hw_pubkey,
        );
        let (new_pin_salt, new_wallet_certificate) = session.begin_change_pin(old_pin, new_pin).await?;

        registration.data.pin_salt = new_pin_salt;
        registration.data.wallet_certificate = new_wallet_certificate;

        info!("PIN change started");

        Ok(())
    }

    pub async fn continue_change_pin(&self, pin: String) -> Result<(), ChangePinError> {
        info!("Continue PIN change");

        info!("Checking if registered");
        let registration = self
            .registration
            .as_ref()
            .ok_or_else(|| ChangePinError::NotRegistered)?;

        // Wallet does not need to be unlocked, see [`Wallet::unlock`].

        let config = &self.config_repository.config().account_server;
        let DerVerifyingKey(instruction_result_public_key) = &config.instruction_result_public_key;
        let instruction_result_public_key = instruction_result_public_key.into();

        let instruction_client = InstructionClientFactory::new(
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            &config.base_url,
            &instruction_result_public_key,
        );

        let session = FinishChangePinOperation::new(&instruction_client, &self.storage, CHANGE_PIN_RETRIES);

        session.finish_change_pin(pin).await?;

        info!("PIN change successfully finalized");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use wallet_common::account::messages::instructions::ChangePinCommit;
    use wallet_common::account::messages::instructions::ChangePinStart;
    use wallet_common::account::messages::instructions::Instruction;
    use wallet_common::account::messages::instructions::InstructionResultClaims;
    use wallet_common::jwt::Jwt;
    use wallet_common::utils;

    use crate::pin::change::ChangePinStorage;
    use crate::pin::change::State;
    use crate::wallet::test::WalletWithMocks;
    use crate::wallet::test::ACCOUNT_SERVER_KEYS;

    async fn create_wp_result<T>(result: T) -> Jwt<InstructionResultClaims<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let result_claims = InstructionResultClaims {
            result,
            iss: "wallet_unit_test".to_string(),
            iat: jsonwebtoken::get_current_timestamp(),
        };
        Jwt::sign_with_sub(&result_claims, &ACCOUNT_SERVER_KEYS.instruction_result_signing_key)
            .await
            .expect("could not sign instruction result")
    }

    #[tokio::test]
    async fn test_wallet_begin_and_continue_change_pin() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet
            .account_provider_client
            .expect_instruction_challenge()
            .times(2)
            .returning(|_, _| Ok(utils::random_bytes(32)));

        let wp_result = create_wp_result(wallet.valid_certificate().await).await;

        wallet
            .account_provider_client
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

        let wp_result = create_wp_result(()).await;

        wallet
            .account_provider_client
            .expect_instruction()
            .times(1)
            .return_once(|_, _: Instruction<ChangePinCommit>| Ok(wp_result));

        let actual = wallet.continue_change_pin("111122".to_string()).await;
        assert_matches!(actual, Ok(()));

        let change_pin_state = wallet
            .storage
            .get_change_pin_state()
            .await
            .expect("could not read change_pin_state");
        assert_eq!(change_pin_state, None);
    }
}
