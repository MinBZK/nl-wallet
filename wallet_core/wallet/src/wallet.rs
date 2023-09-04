use std::{error::Error, marker::PhantomData};

use platform_support::utils::{hardware::HardwareUtilities, PlatformUtilities};
use tracing::{info, instrument};
use url::Url;

pub use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::messages::{
    auth::Registration,
    errors::ErrorType,
    instructions::{CheckPin, Instruction, InstructionChallengeRequest, InstructionChallengeRequestMessage},
};

use crate::{
    account_server::{AccountProviderError, AccountProviderResponseError},
    digid::{DigidAuthenticatorError, DigidClient},
    lock::WalletLock,
    pid_issuer::{PidIssuerClient, PidRetrieverError},
    pin::{
        key::{new_pin_salt, PinKey},
        validation::{validate_pin, PinValidationError},
    },
    storage::{RegistrationData, StorageError, StorageState},
};

pub use crate::{
    account_server::AccountProvider,
    config::{Configuration, ConfigurationRepository},
    digid::DigidAuthenticator,
    pid_issuer::PidRetriever,
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
    ChallengeRequest(#[source] AccountProviderError),
    #[error("could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("could not sign registration message: {0}")]
    Signing(#[source] wallet_common::errors::Error),
    #[error("could not request registration from Wallet Provider: {0}")]
    RegistrationRequest(#[source] AccountProviderError),
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
    #[error(
        "PIN provided is incorrect: (leftover_attempts: {leftover_attempts}, is_final_attempt: {is_final_attempt})"
    )]
    IncorrectPin {
        leftover_attempts: u8,
        is_final_attempt: bool,
    },
    #[error("unlock disabled due to timeout")]
    Timeout { timeout_millis: u64 },
    #[error("unlock permanently disabled")]
    Blocked,
    #[error("server error: {0}")]
    ServerError(#[source] AccountProviderError),
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

impl From<AccountProviderError> for WalletUnlockError {
    fn from(value: AccountProviderError) -> Self {
        if let AccountProviderError::Response(AccountProviderResponseError::Data(_, errordata)) = &value {
            match errordata.typ {
                ErrorType::PinTimeout(data) => WalletUnlockError::Timeout {
                    timeout_millis: data.time_left_in_ms,
                },
                ErrorType::IncorrectPin(data) => WalletUnlockError::IncorrectPin {
                    leftover_attempts: data.attempts_left,
                    is_final_attempt: data.is_final_attempt,
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

#[derive(Debug, thiserror::Error)]
pub enum PidIssuanceError {
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidAuthenticatorError),
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidAuthenticatorError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[source] PidRetrieverError),
}

pub enum RedirectUriType {
    PidIssuance,
    Unknown,
}

type ConfigurationCallback = Box<dyn Fn(&Configuration) + Send + Sync>;

pub struct Wallet<C, S, K, A, D = DigidClient, P = PidIssuerClient, U = HardwareUtilities> {
    config_repository: C,
    storage: S,
    hw_privkey: K,
    account_server: A,
    digid: D,
    pid_issuer: P,
    platform_utils: PhantomData<U>,
    lock: WalletLock,
    registration: Option<RegistrationData>,
    config_callback: Option<ConfigurationCallback>,
}

impl<C, S, K, A> Wallet<C, S, K, A>
where
    C: ConfigurationRepository,
    S: Storage,
    K: PlatformEcdsaKey,
    A: AccountProvider,
{
    pub async fn init_all(config_repository: C) -> Result<Self, WalletInitError> {
        Self::init_wp_and_storage(config_repository, DigidClient::default(), PidIssuerClient::default()).await
    }
}

impl<C, S, K, A, D, P, U> Wallet<C, S, K, A, D, P, U>
where
    C: ConfigurationRepository,
    S: Storage,
    K: PlatformEcdsaKey,
    A: AccountProvider,
    D: DigidAuthenticator,
    P: PidRetriever,
    U: PlatformUtilities,
{
    /// Initialize the wallet by loading initial state.
    pub async fn init_wp_and_storage(config_repository: C, digid: D, pid_issuer: P) -> Result<Self, WalletInitError> {
        let storage_path = U::storage_path().await.map_err(StorageError::from)?;
        let storage = S::new(storage_path);

        let account_server = A::new(&config_repository.config().account_server.base_url).await;

        let mut wallet = Wallet {
            config_repository,
            storage,
            hw_privkey: K::new(WALLET_KEY_ID),
            account_server,
            digid,
            pid_issuer,
            platform_utils: PhantomData,
            lock: WalletLock::new(true),
            registration: None,
            config_callback: None,
        };

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

    async fn new_instruction_challenge_request(
        &mut self,
    ) -> Result<InstructionChallengeRequestMessage, WalletUnlockError> {
        self.increment_sequence_number().await?;

        let registration_data = self.registration.as_ref().unwrap();
        let seq_num = registration_data.instruction_sequence_number;

        let message = InstructionChallengeRequest::new_signed(seq_num, "wallet", &self.hw_privkey)
            .await
            .map_err(WalletUnlockError::Signing)?;

        let challenge_request = InstructionChallengeRequestMessage {
            message,
            certificate: registration_data.wallet_certificate.clone(),
        };

        Ok(challenge_request)
    }

    async fn new_check_pin_request(
        &mut self,
        pin: String,
        challenge: Vec<u8>,
    ) -> Result<Instruction<CheckPin>, WalletUnlockError> {
        self.increment_sequence_number().await?;

        let registration_data = self.registration.as_ref().unwrap();

        let seq_num = registration_data.instruction_sequence_number;

        let pin_key = PinKey::new(&pin, &registration_data.pin_salt.0);
        let signed = CheckPin::new_signed(seq_num, &self.hw_privkey, &pin_key, &challenge)
            .await
            .map_err(WalletUnlockError::Signing)?;

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
        let challenge = self.account_server.instruction_challenge(challenge_request).await?;

        let instruction = self.new_check_pin_request(pin, challenge).await?;
        let signed_result = self.account_server.check_pin(instruction).await?;

        signed_result
            .parse_and_verify(
                &self
                    .config_repository
                    .config()
                    .account_server
                    .instruction_result_public_key,
            )
            .map_err(WalletUnlockError::InstructionResultValidation)?;

        self.lock.unlock();
        Ok(())
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
        // Generate a new PIN salt and derive the private key from the provided PIN.
        let pin_salt = new_pin_salt();
        let pin_key = PinKey::new(&pin, &pin_salt);

        // Retrieve the public key and sign the registration message (these calls may block).
        let hw_pubkey = self
            .hw_privkey
            .verifying_key()
            .await
            .map_err(|e| WalletRegistrationError::HardwarePublicKey(e.into()))?;
        let registration_message = Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)
            .await
            .map_err(WalletRegistrationError::Signing)?;

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

    #[instrument(skip_all)]
    pub async fn create_pid_issuance_redirect_uri(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        let config = &self.config_repository.config().pid_issuance;

        let auth_url = self
            .digid
            .start_session(
                config.digid_url.clone(),
                config.digid_client_id.clone(),
                config.digid_redirect_uri.clone(),
            )
            .await
            .map_err(PidIssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");

        Ok(auth_url)
    }

    pub fn identify_redirect_uri(&self, redirect_uri: &Url) -> RedirectUriType {
        if self.digid.accepts_redirect_uri(redirect_uri) {
            return RedirectUriType::PidIssuance;
        }

        RedirectUriType::Unknown
    }

    #[instrument(skip_all)]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: &Url) -> Result<(), PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        let access_token = self
            .digid
            .get_access_token(redirect_uri)
            .await
            .map_err(PidIssuanceError::DigidSessionFinish)?;

        info!("DigiD access token retrieved, starting actual PID issuance");

        let config = self.config_repository.config();

        self.pid_issuer
            .retrieve_pid(
                &config.pid_issuance.pid_issuer_url,
                &config.mdoc_trust_anchors(),
                &access_token,
            )
            .await
            .map_err(PidIssuanceError::PidIssuer)?;

        info!("PID received successfully from issuer");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::utils::software::SoftwareUtilities;
    use wallet_common::keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier};

    use crate::{
        account_server::MockAccountProvider, config::MockConfigurationRepository, digid::MockDigidAuthenticator,
        pid_issuer::MockPidRetriever, storage::MockStorage,
    };

    use super::*;

    type MockWallet = Wallet<
        MockConfigurationRepository,
        MockStorage,
        SoftwareEcdsaKey,
        MockAccountProvider,
        MockDigidAuthenticator,
        MockPidRetriever,
        SoftwareUtilities,
    >;

    // Emulate wallet:init_wp_and_storage(), with the option to override the mock storage.
    async fn init_wallet(storage: Option<MockStorage>) -> Result<MockWallet, WalletInitError> {
        let storage = storage.unwrap_or_default();

        let mut wallet = Wallet {
            config_repository: MockConfigurationRepository::default(),
            storage,
            hw_privkey: SoftwareEcdsaKey::new(WALLET_KEY_ID),
            account_server: MockAccountProvider::default(),
            digid: MockDigidAuthenticator::new(),
            pid_issuer: MockPidRetriever::new(),
            platform_utils: PhantomData,
            lock: WalletLock::new(true),
            registration: None,
            config_callback: None,
        };

        wallet.fetch_registration().await?;

        Ok(wallet)
    }

    // Tests if the Wallet::init() method completes successfully with the mock generics.
    #[tokio::test]
    async fn test_init() {
        let account_server_client_new_context = MockAccountProvider::new_context();
        account_server_client_new_context
            .expect()
            .returning(|_| MockAccountProvider::default());

        let config_repository = MockConfigurationRepository::default();
        let wallet = MockWallet::init_wp_and_storage(
            config_repository,
            MockDigidAuthenticator::new(),
            MockPidRetriever::new(),
        )
        .await
        .expect("Could not initialize wallet");

        assert!(!wallet.has_registration());
    }

    // Tests the logic of fetching the wallet registration during init and its interaction with the database.
    #[tokio::test]
    async fn test_init_fetch_registration() {
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
        let wallet = init_wallet(Some(MockStorage::mock(StorageState::Unopened, None)))
            .await
            .expect("Could not initialize wallet");

        // The wallet should have no registration, the database should be opened.
        assert!(wallet.registration.is_none());
        assert!(!wallet.has_registration());
        assert!(matches!(wallet.storage.state().await.unwrap(), StorageState::Opened));

        // Test with a wallet with a database file, contains registration.
        let pin_salt = new_pin_salt();
        let wallet = init_wallet(Some(MockStorage::mock(
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
}
