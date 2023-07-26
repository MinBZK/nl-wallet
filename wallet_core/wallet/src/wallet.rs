use std::{error::Error, panic};

use tokio::task;
use tracing::{info, instrument};

pub use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::messages::{
    auth::Registration,
    errors::ErrorType,
    instructions::{CheckPin, Instruction, InstructionChallenge, InstructionChallengeRequest},
};

use crate::{
    account_server::AccountServerResponseError,
    lock::WalletLock,
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
    storage::{RegistrationData, StorageError, StorageState},
    PinValidationError,
};
pub use crate::{
    account_server::{AccountServerClient, AccountServerClientError},
    config::{Configuration, ConfigurationRepository},
    storage::Storage,
};

const WALLET_KEY_ID: &str = "wallet";

#[derive(Debug, thiserror::Error)]
pub enum WalletInitError {
    #[error("could not initialize database: {0}")]
    Database(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum WalletRegistrationError {
    #[error("wallet is already registered")]
    AlreadyRegistered,
    #[error("PIN provided for registration does not adhere to requirements: {0}")]
    InvalidPin(#[from] PinValidationError),
    #[error("could not request registration challenge from Wallet Provider: {0}")]
    ChallengeRequest(#[source] AccountServerClientError),
    #[error("could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not sign registration message: {0}")]
    Signing(#[source] wallet_common::errors::Error),
    #[error("could not request registration from Wallet Provider: {0}")]
    RegistrationRequest(#[source] AccountServerClientError),
    #[error("could not validate registration certificate received from Wallet Provider: {0}")]
    CertificateValidation(#[source] wallet_common::errors::Error),
    #[error("public key in registration certificate received from Wallet Provider does not match hardware public key")]
    PublicKeyMismatch,
    #[error("could not store registration certificate in database: {0}")]
    StoreCertificate(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum WalletUnlockError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("PIN provided is incorrect")]
    IncorrectPin {
        leftover_attempts: u64,
        is_final_attempt: bool,
    },
    #[error("unlock disabled due to timeout")]
    Timeout { timeout_millis: u64 },
    #[error("unlock permanently disabled")]
    Blocked,
    #[error("server error: {0}")]
    ServerError(#[source] AccountServerClientError),
    #[error("could not request instruction challenge from Wallet Provider: {0}")]
    ChallengeRequest(#[source] AccountServerClientError),
    #[error("Wallet Provider could not validate instruction")]
    InstructionValidation,
    #[error("could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not sign instruction: {0}")]
    Signing(#[source] wallet_common::errors::Error),
    #[error("could not validate instruction result received from Wallet Provider: {0}")]
    InstructionResultValidation(#[source] wallet_common::errors::Error),
    #[error("could not store instruction sequence number in database: {0}")]
    StoreInstructionSequenceNumber(#[from] StorageError),
}

impl From<AccountServerClientError> for WalletUnlockError {
    fn from(value: AccountServerClientError) -> Self {
        if let AccountServerClientError::Response(AccountServerResponseError::Data(_, errordata)) = &value {
            match errordata.typ {
                ErrorType::PinTimeout => WalletUnlockError::Timeout {
                    timeout_millis: errordata.unwrap_get("time_left_in_millis"),
                },
                ErrorType::IncorrectPin => WalletUnlockError::IncorrectPin {
                    leftover_attempts: errordata.unwrap_get("leftover_attempts"),
                    is_final_attempt: errordata.unwrap_get("is_final_attempt"),
                },
                ErrorType::AccountBlocked => WalletUnlockError::Blocked,
                ErrorType::InstructionValidation => WalletUnlockError::InstructionValidation,
                _ => WalletUnlockError::ServerError(value),
            }
        } else {
            WalletUnlockError::ServerError(value)
        }
    }
}

type ConfigurationCallback = Box<dyn Fn(&Configuration) + Send + Sync>;

pub struct Wallet<C, A, S, K> {
    config_repository: C,
    account_server: A,
    storage: S,
    hw_privkey: K,
    registration: Option<RegistrationData>,
    lock: WalletLock,
    config_callback: Option<ConfigurationCallback>,
}

impl<C, A, S, K> Wallet<C, A, S, K>
where
    C: ConfigurationRepository,
    A: AccountServerClient,
    S: Storage + Default,
    K: PlatformEcdsaKey + Clone + Send + 'static,
{
    /// Initialize the wallet, but without registration loaded.
    pub fn new_without_registration(config_repository: C) -> Self {
        let account_server = A::new(&config_repository.config().account_server.base_url);
        let storage = S::default();
        let hw_privkey = K::new(WALLET_KEY_ID);

        Wallet {
            config_repository,
            account_server,
            storage,
            hw_privkey,
            registration: None,
            lock: WalletLock::new(true),
            config_callback: None,
        }
    }

    /// Initialize the wallet by loading initial state.
    pub async fn new(config_repository: C) -> Result<Self, WalletInitError> {
        let mut wallet = Self::new_without_registration(config_repository);
        wallet.fetch_registration().await?;

        Ok(wallet)
    }

    /// Attempts to fetch the registration from storage, without creating a database if there is none.
    async fn fetch_registration(&mut self) -> Result<(), StorageError> {
        match self.storage.state().await? {
            // If there is no database file, we can conclude early that there is no registration.
            StorageState::Uninitialized => return Ok(()),
            // Open the database, if necessary.
            StorageState::Unopened => self.storage.open().await?,
            _ => (),
        }

        // Finally, fetch the registration.
        self.registration = self.storage.fetch_data::<RegistrationData>().await?;

        Ok(())
    }

    async fn increment_sequence_number(&mut self) -> Result<(), StorageError> {
        let storage_state = self.storage.state().await?;
        if !matches!(storage_state, StorageState::Opened) {
            self.storage.open().await?;
        }

        let registration_data = self.registration.as_mut().unwrap();
        registration_data.instruction_sequence_number += 1;

        // Update the registration data in storage with incremented instruction sequence number.
        self.storage.update_data(registration_data).await?;

        Ok(())
    }

    async fn new_instruction_challenge_request(&mut self) -> Result<InstructionChallengeRequest, WalletUnlockError> {
        self.increment_sequence_number().await?;

        let registration_data = self.registration.as_ref().unwrap();

        Ok(InstructionChallengeRequest {
            message: InstructionChallenge::new_signed(
                registration_data.instruction_sequence_number,
                "wallet",
                &self.hw_privkey.clone(),
            )
            .map_err(WalletUnlockError::Signing)?,
            certificate: registration_data.wallet_certificate.clone(),
        })
    }

    async fn new_check_pin_request(
        &mut self,
        pin: String,
        challenge: Vec<u8>,
    ) -> Result<Instruction<CheckPin>, WalletUnlockError> {
        self.increment_sequence_number().await?;

        let registration_data = self.registration.as_ref().unwrap();

        let hw_privkey = self.hw_privkey.clone();
        let pin_salt = registration_data.pin_salt.0.clone();

        let seq_num = registration_data.instruction_sequence_number;

        let signed = task::spawn_blocking(move || {
            let pin_key = PinKey::new(&pin, &pin_salt);

            let signed =
                CheckPin::new_signed(seq_num, &hw_privkey, &pin_key, &challenge).map_err(WalletUnlockError::Signing)?;

            // Return ownership of the signed instruction.
            Ok::<_, WalletUnlockError>(signed)
        })
        .await
        .unwrap_or_else(|e| panic::resume_unwind(e.into_panic()))?;

        let instruction = Instruction {
            instruction: signed,
            certificate: registration_data.wallet_certificate.clone(),
        };

        Ok(instruction)
    }

    pub fn set_lock_callback<F>(&mut self, callback: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        callback(self.lock.is_locked());
        self.lock.set_lock_callback(callback);
    }

    pub fn clear_lock_callback(&mut self) {
        self.lock.clear_lock_callback()
    }

    pub fn set_config_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Configuration) + Send + Sync + 'static,
    {
        callback(self.config_repository.config());
        // TODO: Once configuration fetching from the Wallet Provider is implemented,
        //       this callback should be called every time the config updates.
        self.config_callback.replace(Box::new(callback));
    }

    pub fn clear_config_callback(&mut self) {
        self.config_callback.take();
    }

    pub fn has_registration(&self) -> bool {
        self.registration.is_some()
    }

    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    pub fn lock(&mut self) {
        self.lock.lock()
    }

    pub async fn unlock(&mut self, pin: String) -> Result<(), WalletUnlockError> {
        info!("Validating pin");

        if self.registration.is_none() {
            return Err(WalletUnlockError::NotRegistered);
        }

        let challenge_request = self.new_instruction_challenge_request().await?;
        // Retrieve a challenge from the account server
        let challenge = self
            .account_server
            .instruction_challenge(challenge_request)
            .await
            .map_err(WalletUnlockError::ChallengeRequest)?;

        let instruction = self.new_check_pin_request(pin, challenge).await?;
        let signed_result = self.account_server.check_pin(instruction).await?;

        let result = signed_result
            .parse_and_verify(
                &self
                    .config_repository
                    .config()
                    .account_server
                    .instruction_result_public_key,
            )
            .map_err(WalletUnlockError::InstructionResultValidation);

        result.map(|_| self.lock.unlock())
    }

    #[instrument(skip_all)]
    pub async fn register(&mut self, pin: String) -> Result<(), WalletRegistrationError> {
        info!("Checking if already registered");

        // Registration is only allowed if we do not currently have a registration on record.
        if self.has_registration() {
            return Err(WalletRegistrationError::AlreadyRegistered);
        }

        info!("Validating PIN");

        // Make sure the PIN adheres to the requirements.
        validate_pin(&pin)?; // TODO: do not keep PIN in memory while request is in flight

        info!("Requesting challenge from account server");

        // Retrieve a challenge from the account server
        let challenge = self
            .account_server
            .registration_challenge()
            .await
            .map_err(WalletRegistrationError::ChallengeRequest)?;

        info!("Challenge received from account server, signing and sending registration to account server");

        // Create a registration message and double sign it with the challenge.
        // This needs to be performed within a separate thread, since this may be blocking
        // and we are in an async context. While we are in this thread, also retrieve the
        // hardware public key.
        let hw_privkey = self.hw_privkey.clone();
        let (pin_salt, hw_pubkey, registration_message) = task::spawn_blocking(move || {
            // Generate a new PIN salt and derrive the private key from the provided PIN
            let pin_salt = new_pin_salt();
            let pin_key = PinKey::new(&pin, &pin_salt);

            // Retrieve the public key and sign the registration message (these calls may block).
            let hw_pubkey = hw_privkey
                .verifying_key()
                .map_err(|e| WalletRegistrationError::HardwarePublicKey(e.into()))?;
            let registration_message = Registration::new_signed(&hw_privkey, &pin_key, &challenge)
                .map_err(WalletRegistrationError::Signing)?;

            // Return ownership of the pin_salt, the hardware public key and signed registration message.
            Ok::<_, WalletRegistrationError>((pin_salt, hw_pubkey, registration_message))
        })
        .await
        .unwrap_or_else(|e| panic::resume_unwind(e.into_panic()))?;

        // Send the registration message to the account server and receive the wallet certificate in response.
        let cert = self
            .account_server
            .register(registration_message)
            .await
            .map_err(WalletRegistrationError::RegistrationRequest)?;

        info!("Certificate received from account server, verifying contents");

        // Double check that the public key returned in the wallet certificate
        // matches that of our hardware key.
        let cert_claims = cert
            .parse_and_verify(&self.config_repository.config().account_server.certificate_public_key)
            .map_err(WalletRegistrationError::CertificateValidation)?;
        if cert_claims.hw_pubkey.0 != hw_pubkey {
            return Err(WalletRegistrationError::PublicKeyMismatch);
        }

        info!("Storing received registration");

        // If the storage database does not exist, create it now.
        let storage_state = self.storage.state().await?;
        if !matches!(storage_state, StorageState::Opened) {
            self.storage.open().await?;
        }

        // Save the registration data in storage.
        let registration_data = RegistrationData {
            pin_salt: pin_salt.into(),
            wallet_certificate: cert,
            instruction_sequence_number: 0,
        };
        self.storage.insert_data(&registration_data).await?;

        // Keep the registration data in memory.
        self.registration = Some(registration_data);

        // Unlock the wallet after successful registration
        self.lock.unlock();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::AccountServer;

    use crate::{config::MockConfigurationRepository, storage::MockStorage};

    use super::*;

    async fn init_wallet(
        storage: Option<MockStorage>,
    ) -> Result<Wallet<MockConfigurationRepository, AccountServer, MockStorage, SoftwareEcdsaKey>, WalletInitError>
    {
        let config = MockConfigurationRepository::default();

        let mut wallet: Wallet<MockConfigurationRepository, AccountServer, MockStorage, SoftwareEcdsaKey> =
            match storage {
                Some(storage) => {
                    let mut wallet = Wallet::new_without_registration(config);
                    wallet.storage = storage;
                    wallet.fetch_registration().await?;

                    Ok(wallet)
                }
                None => Wallet::new(config).await,
            }?;

        wallet.config_repository.0.account_server.certificate_public_key =
            wallet.account_server.certificate_pubkey.clone();

        Ok(wallet)
    }

    #[tokio::test]
    async fn test_init() {
        // Test with a wallet without a database file.
        let wallet = init_wallet(None).await.expect("Could not initialize wallet");

        // The wallet should have no registration, and no database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(
            wallet.storage.state().await.unwrap(),
            StorageState::Uninitialized
        ));

        // The wallet should be locked by default
        assert!(wallet.is_locked());

        // Test with a wallet with a database file, no registration.
        let wallet = init_wallet(Some(MockStorage::new(StorageState::Unopened, None)))
            .await
            .expect("Could not initialize wallet");

        // The wallet should have no registration, the database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(wallet.storage.state().await.unwrap(), StorageState::Opened));

        // Test with a wallet with a database file, contains registration.
        let pin_salt = new_pin_salt();
        let wallet = init_wallet(Some(MockStorage::new(
            StorageState::Unopened,
            Some(RegistrationData {
                pin_salt: pin_salt.clone().into(),
                wallet_certificate: "thisisjwt".to_string().into(),
                instruction_sequence_number: 0,
            }),
        )))
        .await
        .expect("Could not initialize wallet");

        // The wallet should have a registration, the database should be opened.
        assert!(wallet.registration.is_some());
        assert!(wallet.has_registration());
        assert!(matches!(wallet.storage.state().await.unwrap(), StorageState::Opened));

        // The registration data should now be available.
        assert_eq!(wallet.registration.unwrap().pin_salt.0, pin_salt);
    }

    #[tokio::test]
    async fn test_register() {
        let mut wallet = init_wallet(None).await.expect("Could not initialize wallet");

        // No registration should be loaded initially.
        assert!(!wallet.has_registration());

        // An invalid PIN should result in an error.
        assert!(wallet.register("123456".to_owned()).await.is_err());

        // Actually register with a valid PIN.
        wallet
            .register("112233".to_owned())
            .await
            .expect("Could not register wallet");

        // The registration should now be loaded.
        assert!(wallet.has_registration());

        // Registering again should result in an error.
        assert!(wallet.register("112233".to_owned()).await.is_err());
    }
}
