use std::error::Error;

use futures::TryFutureExt;
use tokio::sync::Mutex;
use tracing::{info, instrument};
use url::Url;

use nl_wallet_mdoc::basic_sa_ext::UnsignedMdoc;
pub use platform_support::hw_keystore::PlatformEcdsaKey;
use platform_support::utils::PlatformUtilities;
use wallet_common::account::messages::{auth::Registration, errors::ErrorType, instructions::CheckPin};

pub use crate::{
    account_server::AccountServerClient,
    config::{Configuration, ConfigurationRepository},
    digid::DigidAuthenticator,
    pid_issuer::PidRetriever,
    storage::Storage,
};
use crate::{
    account_server::{AccountServerClientError, AccountServerResponseError},
    digid::{DigidAuthenticatorError, DigidClient},
    lock::WalletLock,
    pid_issuer::{PidIssuerClient, PidRetrieverError},
    pin::{
        key::{new_pin_salt, PinKey},
        validation::{validate_pin, PinValidationError},
    },
    remote::{InstructionClient, RemoteEcdsaKeyFactory},
    storage::{RegistrationData, StorageError, StorageState},
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
pub enum InstructionError {
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
    ServerError(#[source] AccountServerClientError),
    #[error("Wallet Provider could not validate instruction")]
    InstructionValidation,
    #[error("could not sign instruction: {0}")]
    Signing(#[source] wallet_common::errors::Error),
    #[error("could not validate instruction result received from Wallet Provider: {0}")]
    InstructionResultValidation(#[source] wallet_common::errors::Error),
    #[error("could not store instruction sequence number in database: {0}")]
    StoreInstructionSequenceNumber(#[from] StorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum WalletUnlockError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("could not retrieve registration from database: {0}")]
    Database(#[from] StorageError),
    #[error("could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn Error + Send + Sync>),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
}

impl From<AccountServerClientError> for InstructionError {
    fn from(value: AccountServerClientError) -> Self {
        if let AccountServerClientError::Response(AccountServerResponseError::Data(_, errordata)) = &value {
            match errordata.typ {
                ErrorType::PinTimeout(data) => InstructionError::Timeout {
                    timeout_millis: data.time_left_in_ms,
                },
                ErrorType::IncorrectPin(data) => InstructionError::IncorrectPin {
                    leftover_attempts: data.attempts_left,
                    is_final_attempt: data.is_final_attempt,
                },
                ErrorType::AccountBlocked => InstructionError::Blocked,
                ErrorType::InstructionValidation => InstructionError::InstructionValidation,
                _ => InstructionError::ServerError(value),
            }
        } else {
            InstructionError::ServerError(value)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PidIssuanceError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidAuthenticatorError),
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidAuthenticatorError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[source] PidRetrieverError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    InstructionError(#[from] InstructionError),
}

pub enum RedirectUriType {
    PidIssuance,
    Unknown,
}

type ConfigurationCallback = Box<dyn Fn(&Configuration) + Send + Sync>;

pub struct Wallet<C, A, S, K, D = DigidClient, P = PidIssuerClient> {
    config_repository: C,
    account_server: A,
    storage: Mutex<S>,
    hw_privkey: K,
    digid: D,
    pid_issuer: P,
    lock: WalletLock,
    registration: Option<RegistrationData>,
    config_callback: Option<ConfigurationCallback>,
}

impl<C, A, S, K> Wallet<C, A, S, K>
where
    C: ConfigurationRepository,
    A: AccountServerClient + Sync,
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey,
{
    pub async fn init_all<U: PlatformUtilities>(config_repository: C) -> Result<Self, WalletInitError> {
        Self::init_wp_and_storage::<U>(config_repository, DigidClient::default(), PidIssuerClient::default()).await
    }
}

impl<C, A, S, K, D, P> Wallet<C, A, S, K, D, P>
where
    C: ConfigurationRepository,
    A: AccountServerClient + Sync,
    S: Storage + Send + Sync,
    K: PlatformEcdsaKey,
    D: DigidAuthenticator,
    P: PidRetriever,
{
    /// Initialize the wallet by loading initial state.
    pub async fn init_wp_and_storage<U: PlatformUtilities>(
        config_repository: C,
        digid: D,
        pid_issuer: P,
    ) -> Result<Self, WalletInitError> {
        let storage_path = U::storage_path().await.map_err(StorageError::from)?;
        let storage = Mutex::new(S::new(storage_path));

        let account_server = A::new(&config_repository.config().account_server.base_url).await;

        let mut wallet = Wallet {
            config_repository,
            account_server,
            storage,
            digid,
            pid_issuer,
            hw_privkey: K::new(WALLET_KEY_ID),
            lock: WalletLock::new(true),
            registration: None,
            config_callback: None,
        };

        wallet.fetch_registration().await?;

        Ok(wallet)
    }

    /// Attempts to fetch the registration data from storage, without creating a database if there is none.
    async fn fetch_registration(&mut self) -> Result<(), StorageError> {
        let mut storage = self.storage.lock().await;
        match storage.state().await? {
            // If there is no database file, we can conclude early that there is no registration.
            StorageState::Uninitialized => return Ok(()),
            // Open the database, if necessary.
            StorageState::Unopened => storage.open().await?,
            _ => (),
        }

        // Finally, fetch the registration.
        self.registration = storage.fetch_data::<RegistrationData>().await?;

        Ok(())
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
        let mut storage = self.storage.lock().await;
        let storage_state = storage.state().await?;
        if !matches!(storage_state, StorageState::Opened) {
            storage.open().await?;
        }

        // Save the registration data in storage.
        let registration_data = RegistrationData {
            pin_salt: pin_salt.into(),
            wallet_certificate: cert,
        };
        storage.insert_data(&registration_data).await?;

        // Keep the registration data in memory.
        self.registration = Some(registration_data);

        // Unlock the wallet after successful registration
        self.lock.unlock();

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn unlock(&mut self, pin: String) -> Result<(), WalletUnlockError> {
        info!("Validating pin");

        info!("Checking if already registered");
        if !self.has_registration() {
            return Err(WalletUnlockError::NotRegistered);
        }

        let registration_data = self.registration.as_ref().unwrap();

        let remote_instruction = InstructionClient::new(
            pin,
            &registration_data.pin_salt,
            &registration_data.wallet_certificate,
            &self.hw_privkey,
            &self.account_server,
            &self.storage,
            &self
                .config_repository
                .config()
                .account_server
                .instruction_result_public_key,
        );

        remote_instruction
            .send(CheckPin)
            .inspect_ok(|_| self.lock.unlock())
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn create_pid_issuance_redirect_uri(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if already registered");
        if !self.has_registration() {
            return Err(PidIssuanceError::NotRegistered);
        }

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

    pub fn cancel_pid_issuance(&mut self) {
        self.digid.cancel_session();
    }

    #[instrument(skip_all)]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: &Url) -> Result<Vec<UnsignedMdoc>, PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        info!("Checking if already registered");
        if !self.has_registration() {
            return Err(PidIssuanceError::NotRegistered);
        }

        let access_token = self
            .digid
            .get_access_token(redirect_uri)
            .await
            .map_err(PidIssuanceError::DigidSessionFinish)?;

        info!("DigiD access token retrieved, starting actual PID issuance");

        let config = self.config_repository.config();

        let unsigned_mdocs = self
            .pid_issuer
            .start_retrieve_pid(&config.pid_issuance.pid_issuer_url, &access_token)
            .await
            .map_err(PidIssuanceError::PidIssuer)?;

        info!("PID received successfully from issuer");

        Ok(unsigned_mdocs)
    }

    #[instrument(skip_all)]
    pub async fn accept_pid_issuance(&mut self, pin: String) -> Result<(), PidIssuanceError> {
        info!("Accepting PID issuance");

        let config = self.config_repository.config();

        info!("Checking if already registered");
        if !self.has_registration() {
            return Err(PidIssuanceError::NotRegistered);
        }

        let registration_data = self.registration.as_ref().unwrap();

        let remote_instruction = InstructionClient::new(
            pin,
            &registration_data.pin_salt,
            &registration_data.wallet_certificate,
            &self.hw_privkey,
            &self.account_server,
            &self.storage,
            &config.account_server.instruction_result_public_key,
        );
        let remote_key_factory = RemoteEcdsaKeyFactory::new(&remote_instruction);

        self.pid_issuer
            .accept_pid(&config.mdoc_trust_anchors(), &remote_key_factory)
            .await
            .map_err(PidIssuanceError::PidIssuer)
    }
}

#[cfg(test)]
mod tests {
    use platform_support::utils::software::SoftwareUtilities;
    use wallet_common::keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier};
    use wallet_provider::{stub, AccountServer};

    use crate::{
        config::MockConfigurationRepository, digid::MockDigidAuthenticator, pid_issuer::MockPidRetriever,
        storage::MockStorage,
    };

    use super::*;

    type MockWallet = Wallet<
        MockConfigurationRepository,
        AccountServer,
        MockStorage,
        SoftwareEcdsaKey,
        MockDigidAuthenticator,
        MockPidRetriever,
    >;

    // Emulate wallet:init_wp_and_storage(), with the option to override the mock storage.
    async fn init_wallet(storage: Option<MockStorage>) -> Result<MockWallet, WalletInitError> {
        let mut config_repository = MockConfigurationRepository::default();

        let account_server = stub::account_server().await;
        let storage = Mutex::new(storage.unwrap_or_default());

        config_repository.0.account_server.certificate_public_key = account_server.certificate_pubkey.clone();

        let mut wallet = Wallet {
            config_repository,
            account_server,
            storage,
            digid: MockDigidAuthenticator::new(),
            pid_issuer: MockPidRetriever {},
            hw_privkey: SoftwareEcdsaKey::new(WALLET_KEY_ID),
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
        let config_repository = MockConfigurationRepository::default();
        let wallet = MockWallet::init_wp_and_storage::<SoftwareUtilities>(
            config_repository,
            MockDigidAuthenticator::new(),
            MockPidRetriever {},
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
            wallet.storage.lock().await.state().await.unwrap(),
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
        assert!(matches!(
            wallet.storage.lock().await.state().await.unwrap(),
            StorageState::Opened
        ));

        // Test with a wallet with a database file, contains registration.
        let pin_salt = new_pin_salt();
        let wallet = init_wallet(Some(MockStorage::mock(
            StorageState::Unopened,
            Some(RegistrationData {
                pin_salt: pin_salt.clone().into(),
                wallet_certificate: "thisisjwt".to_string().into(),
            }),
        )))
        .await
        .expect("Could not initialize wallet");

        // The wallet should have a registration, the database should be opened.
        assert!(wallet.registration.is_some());
        assert!(wallet.has_registration());
        assert!(matches!(
            wallet.storage.lock().await.state().await.unwrap(),
            StorageState::Opened
        ));

        // The registration data should now be available.
        assert_eq!(wallet.registration.unwrap().pin_salt.0, pin_salt);
    }

    // Test a full registration with the mock wallet.
    //
    // TODO: Since the wallet_provider integration tests also covers this, this should be removed.
    //       This can be done whenever the AccountServerClient has a proper mock.
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
