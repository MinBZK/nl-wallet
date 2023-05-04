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

#[derive(Debug)]
enum RegistrationData {
    Unloaded,
    Loaded(Option<data::Registration>),
}

impl RegistrationData {
    #[allow(dead_code)] // TODO: remove when this is by future Wallet methods
    fn data(&self) -> Option<&data::Registration> {
        if let RegistrationData::Loaded(data) = self {
            return data.as_ref();
        }

        None
    }
}

pub struct Wallet<A, S, K> {
    account_server: A,
    account_server_pubkey: EcdsaDecodingKey,
    storage: S,
    hw_privkey: K,
    registration: RegistrationData,
}

impl<A, S, K> Wallet<A, S, K>
where
    A: AccountServerClient,
    S: Storage,
    K: PlatformEcdsaKey,
{
    pub fn new(account_server: A, account_server_pubkey: EcdsaDecodingKey, storage: S) -> Wallet<A, S, K> {
        Wallet {
            account_server,
            account_server_pubkey,
            storage,
            hw_privkey: K::new(WALLET_KEY_ID),
            registration: RegistrationData::Unloaded,
        }
    }

    /// If the wallet was registered with the wallet provider before,
    /// load the regisration from storage. Returns a boolean indicating
    /// whether we have have a registration.
    pub async fn load_registration(&mut self) -> Result<bool> {
        // Return early if the registration was previously loaded.
        if let RegistrationData::Loaded(registration) = &self.registration {
            return Ok(registration.is_some());
        }

        let storage_state = self.storage.state().await?;

        // If there is no database file, we can already conclude that we are not registered.
        if matches!(storage_state, StorageState::Uninitialized) {
            self.registration = RegistrationData::Loaded(None);

            return Ok(false);
        }

        // Open the database, if necessary.
        if matches!(storage_state, StorageState::Unopened) {
            self.storage.open().await?;
        }

        // Finally, fetch the registration.
        let registration = self.storage.registration().await?;
        let has_registration = registration.is_some();

        self.registration = RegistrationData::Loaded(registration);

        Ok(has_registration)
    }

    pub async fn register(&mut self, pin: String) -> Result<()> {
        // Registration is only allowed if we do not currently have a registration on record.
        // If the registration data was not loaded from storage before, do so now.
        if self.load_registration().await? {
            return Err(anyhow!("Wallet is already registered"));
        }

        // Make sure the PIN adheres to the requirements.
        validate_pin(&pin)?; // TODO: do not keep PIN in memory while request is in flight

        let challenge = self.account_server.registration_challenge()?;
        let pin_salt = new_pin_salt();
        let pin_key = PinKey::new(&pin, &pin_salt);

        let registration_message = Registration::new_signed(&self.hw_privkey, &pin_key, &challenge)?;
        let cert = self.account_server.register(registration_message)?;

        let cert_claims = cert.parse_and_verify(&self.account_server_pubkey)?;
        if cert_claims.hw_pubkey.0 != self.hw_privkey.verifying_key()? {
            return Err(anyhow!("hardware pubkey did not match"));
        }

        // If the storage datbase does not exist, create it now
        let storage_state = self.storage.state().await?;
        if !matches!(storage_state, StorageState::Opened) {
            self.storage.open().await?;
        }

        // Save the registration data in storage
        let registration = data::Registration {
            pin_salt,
            wallet_certificate: cert,
        };
        self.storage.insert_registration(&registration).await?;
        self.registration = RegistrationData::Loaded(Some(registration));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::hw_keystore::software::SoftwareEcdsaKey;
    use wallet_provider::account_server::AccountServer;

    use crate::storage::MockStorage;

    use super::*;

    fn create_wallet(storage: Option<MockStorage>) -> Wallet<AccountServer, MockStorage, SoftwareEcdsaKey> {
        let account_server = AccountServer::new_stub();
        let pubkey = account_server.pubkey.clone();

        Wallet::new(account_server, pubkey, storage.unwrap_or_default())
    }

    #[tokio::test]
    async fn test_load_registration() {
        // Test with a wallet without a database file.
        let mut wallet = create_wallet(None);

        // No registration should be loaded initially.
        assert!(matches!(wallet.registration, RegistrationData::Unloaded));

        let has_registration = wallet.load_registration().await.expect("Could not load registration");

        // We should be informed that there is no registration, and no database should be opened.
        assert!(!has_registration);
        assert!(matches!(wallet.registration, RegistrationData::Loaded(None)));
        assert!(matches!(
            wallet.storage.state().await.unwrap(),
            StorageState::Uninitialized
        ));

        // Test with a wallet with a database file, no registration.
        let mut wallet = create_wallet(Some(MockStorage {
            state: StorageState::Unopened,
            registration: None,
        }));

        let has_registration = wallet.load_registration().await.expect("Could not load registration");

        // We should be informed that there is no registration, the database should be opened.
        assert!(!has_registration);
        assert!(matches!(wallet.registration, RegistrationData::Loaded(None)));
        assert!(matches!(wallet.storage.state().await.unwrap(), StorageState::Opened));

        // Test with a wallet with a database file, contains registration.
        let pin_salt = new_pin_salt();
        let mut wallet = create_wallet(Some(MockStorage {
            state: StorageState::Unopened,
            registration: Some(data::Registration {
                pin_salt: pin_salt.clone(),
                wallet_certificate: "thisisjwt".to_string().into(),
            }),
        }));

        let has_registration = wallet.load_registration().await.expect("Could not load registration");

        // We should be informed that there is a registration, the database should be opened.
        assert!(has_registration);
        assert!(matches!(wallet.registration, RegistrationData::Loaded(Some(_))));
        assert!(matches!(wallet.storage.state().await.unwrap(), StorageState::Opened));

        // The registration data should now be available.
        assert_eq!(wallet.registration.data().unwrap().pin_salt, pin_salt);
    }

    #[tokio::test]
    async fn test_register() {
        let mut wallet = create_wallet(None);

        // No registration should be loaded initially.
        assert!(matches!(wallet.registration, RegistrationData::Unloaded));

        // An invalid PIN should result in an error.
        assert!(wallet.register("123456".to_owned()).await.is_err());

        // Actually register with a valid PIN.
        wallet
            .register("112233".to_owned())
            .await
            .expect("Could not register wallet");

        // The registration should now be loaded.
        assert!(matches!(wallet.registration, RegistrationData::Loaded(Some(_))));

        // Registering again should result in an error.
        assert!(wallet.register("112233".to_owned()).await.is_err());
    }
}
