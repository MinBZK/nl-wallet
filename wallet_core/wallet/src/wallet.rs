use std::{error::Error, panic};

use tokio::task;
use tracing::{info, instrument};

use wallet_common::account::{auth::Registration, jwt::EcdsaDecodingKey};

use crate::{
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
    storage::{RegistrationData, StorageError, StorageState},
    PinValidationError,
};

pub use platform_support::hw_keystore::PlatformEcdsaKey;

pub use crate::{
    account_server::{AccountServerClient, AccountServerClientError},
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
    #[error("PIN provided is incorrect")]
    IncorrectPin,
}

/// Attempts to fetch the registration from storage,
/// without creating a database if there is none.
async fn fetch_registration(storage: &mut impl Storage) -> Result<Option<RegistrationData>, StorageError> {
    match storage.state().await? {
        // If there is no database file, we can conclude early that there is no registration.
        StorageState::Uninitialized => return Ok(None),
        // Open the database, if necessary.
        StorageState::Unopened => storage.open().await?,
        _ => (),
    }

    // Finally, fetch the registration.
    let registration = storage.fetch_data::<RegistrationData>().await?;

    Ok(registration)
}

pub struct Wallet<A, S, K> {
    account_server: A,
    account_server_pubkey: EcdsaDecodingKey,
    storage: S,
    hw_privkey: K,
    registration: Option<RegistrationData>,
    is_locked: bool,
}

impl<A, S, K> Wallet<A, S, K>
where
    A: AccountServerClient,
    S: Storage,
    K: PlatformEcdsaKey + Clone + Send + 'static,
{
    // Initialize the wallet by loading initial state.
    pub async fn new(
        account_server: A,
        account_server_pubkey: EcdsaDecodingKey,
        mut storage: S,
    ) -> Result<Self, WalletInitError> {
        let registration = fetch_registration(&mut storage).await?;
        let hw_privkey = K::new(WALLET_KEY_ID);

        let wallet = Wallet {
            account_server,
            account_server_pubkey,
            storage,
            hw_privkey,
            registration,
            is_locked: true,
        };

        Ok(wallet)
    }

    pub fn has_registration(&self) -> bool {
        self.registration.is_some()
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn lock(&mut self) {
        self.is_locked = true
    }

    #[instrument(skip_all)]
    pub async fn unlock(&mut self, pin: String) -> Result<(), WalletUnlockError> {
        info!("Validating pin");
        //TODO: Validate pin with account server
        if pin != "000000" {
            self.is_locked = true;
            return Err(WalletUnlockError::IncorrectPin);
        }

        info!("Pin validated, unlocking wallet");
        self.is_locked = false;
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
            .parse_and_verify(&self.account_server_pubkey)
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
        };
        self.storage.insert_data(&registration_data).await?;

        // Keep the registration data in memory.
        self.registration = Some(registration_data);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::account_server::{stub, AccountServer};

    use crate::storage::MockStorage;

    use super::*;

    async fn init_wallet(
        storage: Option<MockStorage>,
    ) -> Result<Wallet<AccountServer, MockStorage, SoftwareEcdsaKey>, WalletInitError> {
        let account_server = stub::account_server();
        let pubkey = account_server.pubkey.clone();

        Wallet::new(account_server, pubkey, storage.unwrap_or_default()).await
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
        assert!(wallet.is_locked);

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

    #[tokio::test]
    async fn test_lock_mechanism() {
        let mut wallet = init_wallet(None).await.expect("Could not initialize wallet");

        // Wallet should initialize in locked state
        assert!(wallet.is_locked());

        wallet
            .unlock("000000".to_string())
            .await
            .expect("Could not unlock wallet");

        // Wallet should be unlocked after valid unlock attempt
        assert!(!wallet.is_locked());

        wallet.lock();

        // Wallet should be locked after valid lock attempt
        assert!(wallet.is_locked());
    }
}
