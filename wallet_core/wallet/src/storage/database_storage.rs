use anyhow::Result;
use platform_support::preferred::{PlatformEncryptionKey, PlatformUtilities};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use wallet_entity::keyed_data;

use super::{data::Registration, wallet_database::WalletDatabase, Storage};

const DATABASE_NAME: &str = "wallet";

const REGISTRATION_KEY: &str = "registration";

#[derive(Debug)]
pub struct DatabaseStorage {
    database: WalletDatabase,
}

impl DatabaseStorage {
    fn new(database: WalletDatabase) -> Self {
        DatabaseStorage { database }
    }

    pub async fn open() -> Result<Self> {
        let database = WalletDatabase::open::<PlatformEncryptionKey, PlatformUtilities>(DATABASE_NAME).await?;

        Ok(Self::new(database))
    }

    pub async fn destroy(self) -> Result<()> {
        self.database.close_and_delete::<PlatformUtilities>().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Storage for DatabaseStorage {
    async fn get_registration(&self) -> Result<Option<Registration>> {
        let registration = keyed_data::Entity::find_by_id(REGISTRATION_KEY)
            .one(self.database.get_connection())
            .await?
            .map(|m| serde_json::from_value::<Registration>(m.data))
            .transpose()?;

        Ok(registration)
    }

    async fn save_registration(&mut self, registration: &Registration) -> Result<()> {
        let _ = keyed_data::ActiveModel {
            key: Set(REGISTRATION_KEY.to_string()),
            data: Set(serde_json::to_value(registration)?),
        }
        .insert(self.database.get_connection())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use platform_support::{
        hw_keystore::software::SoftwareEncryptionKey,
        utils::{software::SoftwareUtilities, PlatformUtilities},
    };
    use tokio::{fs, try_join};
    use wallet_common::account::WalletCertificate;

    use super::*;

    #[tokio::test]
    async fn test_database_storage() {
        let name = "test_database_storage";
        let registration = Registration {
            pin_salt: vec![1, 2, 3, 4],
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid"),
        };

        // Make sure we start with a clean slate.
        let database_path = SoftwareUtilities::storage_path().unwrap().join(format!("{}.db", name));
        let key_file_path = SoftwareUtilities::storage_path()
            .unwrap()
            .join(format!("{}_db.key", name));
        _ = try_join!(fs::remove_file(&database_path), fs::remove_file(&key_file_path));

        // Create a test database, pass it to the private new() constructor.
        let database = WalletDatabase::open::<SoftwareEncryptionKey, SoftwareUtilities>("test_database_storage")
            .await
            .expect("Could not open database");
        let mut storage = DatabaseStorage::new(database);

        // Try to fetch the registration, none should be there.
        let no_registration = storage.get_registration().await.expect("Could not get registration");

        assert!(no_registration.is_none());

        // Save the registration and fetch it again.
        storage
            .save_registration(&registration)
            .await
            .expect("Could not save registration");

        let fetched_registration = storage.get_registration().await.expect("Could not get registration");

        assert!(fetched_registration.is_some());
        let fetched_registration = fetched_registration.unwrap();
        assert_eq!(fetched_registration.pin_salt, registration.pin_salt);
        assert_eq!(
            fetched_registration.wallet_certificate.0,
            registration.wallet_certificate.0
        );

        // Save the registration again, should result in an error.
        let save_result = storage.save_registration(&registration).await;

        assert!(save_result.is_err());
    }
}
