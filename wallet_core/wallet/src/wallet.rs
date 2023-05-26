use tracing::{info, instrument};

use wallet_common::account::{auth::Registration, jwt::EcdsaDecodingKey};

use crate::{
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
    storage::{RegistrationData, StorageState},
    PinValidationError,
};

pub use platform_support::hw_keystore::PlatformEcdsaKey;

pub use crate::{account_server::AccountServerClient, storage::Storage};

const WALLET_KEY_ID: &str = "wallet";

#[derive(Debug, thiserror::Error)]
pub enum WalletInitError {
    #[error("Could not initialize database: {0}")]
    Database(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum WalletRegistrationError {
    #[error("Wallet is already registered")]
    AlreadyRegistered,
    #[error("PIN provided for registration does not adhere to requirements: {0}")]
    InvalidPin(#[from] PinValidationError),
    #[error("Could not request registration challenge from Wallet Provider: {0}")]
    ChallengeRequest(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Could not sign registration message: {0}")]
    Signing(#[source] wallet_common::account::errors::Error),
    #[error("Could not request registration from Wallet Provider: {0}")]
    RegistrationRequest(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Could not validate registration certificate received from Wallet Provider: {0}")]
    Validation(#[source] wallet_common::account::errors::Error),
    #[error("Could not get hardware public key: {0}")]
    HardwarePublicKey(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Public key in registration certificate received from Wallet Provider does not match hardware public key")]
    PublicKeyMismatch,
    #[error("Could not store registration certificate in database: {0}")]
    StoreCertificate(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Attempts to fetch the registration from storage,
/// without creating a database if there is none.
async fn fetch_registration<S: Storage>(storage: &mut S) -> Result<Option<RegistrationData>, S::Error> {
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

async fn store_registration_data<S: Storage>(
    storage: &mut S,
    registration_data: &RegistrationData,
) -> Result<(), S::Error> {
    // If the storage datbase does not exist, create it now
    let storage_state = storage.state().await?;
    if !matches!(storage_state, StorageState::Opened) {
        storage.open().await?;
    }

    // Save the registration data in storage
    storage.insert_data(registration_data).await?;

    Ok(())
}

pub struct Wallet<A, S, K> {
    account_server: A,
    account_server_pubkey: EcdsaDecodingKey,
    storage: S,
    hw_privkey: K,
    registration: Option<RegistrationData>,
}

impl<A, S, K> Wallet<A, S, K>
where
    A: AccountServerClient,
    S: Storage,
    K: PlatformEcdsaKey,
{
    // Initialize the wallet by loading initial state.
    pub async fn new(
        account_server: A,
        account_server_pubkey: EcdsaDecodingKey,
        mut storage: S,
    ) -> Result<Self, WalletInitError> {
        let registration = fetch_registration(&mut storage)
            .await
            .map_err(|e| WalletInitError::Database(e.into()))?;
        let hw_privkey = K::new(WALLET_KEY_ID);

        let wallet = Wallet {
            account_server,
            account_server_pubkey,
            storage,
            hw_privkey,
            registration,
        };

        Ok(wallet)
    }

    pub fn has_registration(&self) -> bool {
        self.registration.is_some()
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
            .map_err(|e| WalletRegistrationError::ChallengeRequest(e.into()))?;

        info!("Challenge received from account server, signing and sending registration to account server");

        // Generate a new PIN salt and derrive the private key from the provided PIN
        let pin_salt = new_pin_salt();
        let pin_key = PinKey::new(&pin, &pin_salt);

        // Create a registration message and double sign it with the challenge,
        // send that to the account server and receive the wallet certificate in response.
        let registration_message = Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)
            .map_err(WalletRegistrationError::Signing)?;
        let cert = self
            .account_server
            .register(registration_message)
            .await
            .map_err(|e| WalletRegistrationError::RegistrationRequest(e.into()))?;

        info!("Certificate received from account server, verifying contents");

        // Double check that the public key returned in the wallet certificate
        // matches that of our hardware key.
        let cert_claims = cert
            .parse_and_verify(&self.account_server_pubkey)
            .map_err(WalletRegistrationError::Validation)?;
        let hw_pubkey = self
            .hw_privkey
            .verifying_key()
            .map_err(|e| WalletRegistrationError::HardwarePublicKey(e.into()))?;
        if cert_claims.hw_pubkey.0 != hw_pubkey {
            return Err(WalletRegistrationError::PublicKeyMismatch);
        }

        info!("Storing received registration");

        let registration_data = RegistrationData {
            pin_salt: pin_salt.into(),
            wallet_certificate: cert,
        };
        store_registration_data(&mut self.storage, &registration_data)
            .await
            .map_err(|e| WalletRegistrationError::StoreCertificate(e.into()))?;
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
}
