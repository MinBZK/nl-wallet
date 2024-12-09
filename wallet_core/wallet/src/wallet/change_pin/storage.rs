use tokio::sync::RwLock;

use wallet_common::account::messages::auth::WalletCertificate;

use crate::errors::StorageError;
use crate::pin::change::ChangePinStorage;
use crate::pin::change::State;
use crate::storage::ChangePinData;
use crate::storage::RegistrationData;
use crate::storage::Storage;

impl<S> ChangePinStorage for RwLock<S>
where
    S: Storage,
{
    async fn get_change_pin_state(&self) -> Result<Option<State>, StorageError> {
        let storage = self.read().await;
        let change_pin_data: Option<ChangePinData> = storage.fetch_data().await?;
        Ok(change_pin_data.and_then(|data| data.state))
    }

    async fn store_change_pin_state(&self, state: State) -> Result<(), StorageError> {
        let mut storage = self.write().await;
        let data = ChangePinData { state: Some(state) };
        storage.upsert_data(&data).await
    }

    async fn clear_change_pin_state(&self) -> Result<(), StorageError> {
        let mut storage = self.write().await;
        let data = ChangePinData { state: None };
        storage.upsert_data(&data).await
    }

    async fn change_pin(
        &self,
        current_registration_data: RegistrationData,
        new_pin_salt: Vec<u8>,
        new_pin_certificate: WalletCertificate,
    ) -> Result<(), StorageError> {
        let mut storage = self.write().await;
        let registration_data = RegistrationData {
            pin_salt: new_pin_salt,
            wallet_certificate: new_pin_certificate,
            ..current_registration_data
        };
        storage.upsert_data(&registration_data).await
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::pin::change::ChangePinStorage;
    use crate::pin::change::State;
    use crate::storage::MockStorage;
    use crate::storage::StorageState;

    use super::*;

    #[tokio::test]
    async fn test_change_pin_storage() {
        let storage = MockStorage::new(StorageState::Opened, None);
        let change_pin_storage = RwLock::new(storage);

        assert_matches!(change_pin_storage.get_change_pin_state().await, Ok(None));

        assert_matches!(change_pin_storage.store_change_pin_state(State::Commit).await, Ok(()));

        assert_matches!(change_pin_storage.get_change_pin_state().await, Ok(Some(State::Commit)));

        assert_matches!(change_pin_storage.clear_change_pin_state().await, Ok(()));

        assert_matches!(change_pin_storage.get_change_pin_state().await, Ok(None));

        {
            let storage = change_pin_storage.read().await;
            assert_matches!(storage.fetch_data::<RegistrationData>().await, Ok(None));
        }

        let registration_data = RegistrationData {
            attested_key_identifier: "key_id".to_string(),
            pin_salt: b"pin_salt_1234_old".to_vec(),
            wallet_id: "wallet_123".to_string(),
            wallet_certificate: WalletCertificate::from("thisisdefinitelyvalid_current"),
        };
        let new_pin_salt = b"pin_salt_1234_new".to_vec();
        let new_wallet_certificate = WalletCertificate::from("thisisdefinitelyvalid_new");

        assert_matches!(
            change_pin_storage
                .change_pin(registration_data, new_pin_salt, new_wallet_certificate)
                .await,
            Ok(())
        );

        {
            let storage = change_pin_storage.read().await;
            let actual = storage
                .fetch_data::<RegistrationData>()
                .await
                .expect("database error")
                .expect("no registation data found");
            assert_eq!(actual.pin_salt, b"pin_salt_1234_new");
        }
    }
}
