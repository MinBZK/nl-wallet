use wallet_common::account::messages::auth::WalletCertificate;

use crate::{
    errors::StorageError,
    pin::change::{ChangePinStorage, State},
    storage::{ChangePinData, RegistrationData, Storage},
    Wallet,
};

impl<CR, S, PEK, APC, DS, IS, MDS> ChangePinStorage for Wallet<CR, S, PEK, APC, DS, IS, MDS>
where
    S: Storage,
{
    async fn get_state(&self) -> Result<Option<State>, StorageError> {
        let storage = self.storage.read().await;
        let change_pin_data: Option<ChangePinData> = storage.fetch_data().await?;
        Ok(change_pin_data.and_then(|data| data.state))
    }

    async fn store_state(&self, state: State) -> Result<(), StorageError> {
        let mut storage = self.storage.write().await;
        let data = ChangePinData { state: Some(state) };
        storage.upsert_data(&data).await
    }

    async fn clean_state(&self) -> Result<(), StorageError> {
        let mut storage = self.storage.write().await;
        let data = ChangePinData { state: None };
        storage.upsert_data(&data).await
    }

    async fn change_pin(
        &self,
        new_pin_salt: Vec<u8>,
        new_pin_certificate: WalletCertificate,
    ) -> Result<(), StorageError> {
        let mut storage = self.storage.write().await;
        let data = RegistrationData {
            pin_salt: new_pin_salt,
            wallet_certificate: new_pin_certificate,
        };
        storage.upsert_data(&data).await
    }
}
