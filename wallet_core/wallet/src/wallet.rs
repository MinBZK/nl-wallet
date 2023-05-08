use anyhow::{anyhow, Result};

use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::account::{instructions::Registration, jwt::EcdsaDecodingKey, AccountServerClient};

use crate::{
    pin::{
        key::{new_pin_salt, PinKey},
        validation::validate_pin,
    },
    storage::{data, Storage, StorageState},
};

const WALLET_KEY_ID: &str = "wallet";

/// If the wallet was registered with the wallet provider before,
/// fetch the regisration from storage.
pub async fn fetch_registration(storage: &mut impl Storage) -> Result<Option<data::Registration>> {
    let storage_state = storage.state().await?;

    // If there is no database file, we can conclude early that there is no registration.
    if matches!(storage_state, StorageState::Uninitialized) {
        return Ok(None);
    }

    // Open the database, if necessary.
    if matches!(storage_state, StorageState::Unopened) {
        storage.open().await?;
    }

    // Finally, fetch the registration.
    let registration = storage.fetch_data::<data::Registration>().await?;

    Ok(registration)
}

#[derive(Debug, thiserror::Error)]
pub enum WalletRegistrationError {
    #[error("Wallet is already registered")]
    AlreadyRegistered,
    #[error("Public key received from account server does not match the hardware public key")]
    PublicKeyMismatch,
}

pub struct Wallet<A, S, K> {
    account_server: A,
    account_server_pubkey: EcdsaDecodingKey,
    storage: S,
    hw_privkey: K,
    registration: Option<data::Registration>,
}

impl<A, S, K> Wallet<A, S, K>
where
    A: AccountServerClient,
    S: Storage,
    K: PlatformEcdsaKey,
{
    // Initialize the wallet by loading initial state.
    pub async fn new(account_server: A, account_server_pubkey: EcdsaDecodingKey, mut storage: S) -> Result<Self> {
        let registration = fetch_registration(&mut storage).await?;
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

    pub async fn register(&mut self, pin: String) -> Result<()> {
        // Registration is only allowed if we do not currently have a registration on record.
        if self.has_registration() {
            return Err(anyhow!(WalletRegistrationError::AlreadyRegistered));
        }

        // Make sure the PIN adheres to the requirements.
        validate_pin(&pin)?; // TODO: do not keep PIN in memory while request is in flight

        // Retrieve a challenge from the account server
        let challenge = self.account_server.registration_challenge()?;

        // Generate a new PIN salt and derrive the private key from the provided PIN
        let pin_salt = new_pin_salt();
        let pin_key = PinKey::new(&pin, &pin_salt);

        // Create a registration message and double sign it with the challenge,
        // send that to the account server and receive the wallet certificate in response.
        let registration_message = Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        // Double check that the public key returned in the wallet certificate
        // matches that of our hardware key.
        let cert_claims = cert.parse_and_verify(&self.account_server_pubkey)?;
        if cert_claims.hw_pubkey.0 != self.hw_privkey.verifying_key()? {
            return Err(anyhow!(WalletRegistrationError::PublicKeyMismatch));
        }

        // If the storage datbase does not exist, create it now
        let storage_state = self.storage.state().await?;
        if !matches!(storage_state, StorageState::Opened) {
            self.storage.open().await?;
        }

        // Save the registration data in storage
        let registration = data::Registration {
            pin_salt: pin_salt.into(),
            wallet_certificate: cert,
        };
        self.storage.insert_data(&registration).await?;
        self.registration = Some(registration);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::account_server::AccountServer;

    use crate::storage::MockStorage;

    use super::*;

    async fn init_wallet(storage: Option<MockStorage>) -> Result<Wallet<AccountServer, MockStorage, SoftwareEcdsaKey>> {
        let account_server = AccountServer::new_stub();
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
            Some(data::Registration {
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
