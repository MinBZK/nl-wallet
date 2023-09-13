use std::{any::Any, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;

use crate::storage::RegistrationData;

use super::{data::KeyedData, Storage, StorageError, StorageState};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: Arc<DashMap<&'static str, Box<dyn Any + Send + Sync>>>,
}

impl MockStorage {
    pub fn mock(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let data: Arc<DashMap<&str, Box<dyn Any + Send + Sync>>> = Arc::new(DashMap::new());

        if let Some(registration) = registration {
            data.insert(RegistrationData::KEY, Box::new(registration));
        }

        MockStorage { state, data }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::mock(StorageState::Uninitialized, None)
    }
}

#[async_trait]
impl Storage for MockStorage {
    fn new(_: PathBuf) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    async fn state(&self) -> Result<StorageState, StorageError> {
        Ok(self.state)
    }

    async fn open(&mut self) -> Result<(), StorageError> {
        self.state = StorageState::Opened;

        Ok(())
    }

    async fn clear(&mut self) -> Result<(), StorageError> {
        self.state = StorageState::Uninitialized;

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, StorageError> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(StorageError::NotOpened);
        }

        // If self.data contains the key for the requested type,
        // assume that its value is of that specific type.
        // Downcast it to the type using the Any trait, then return a cloned result.
        let data: Option<D> = self
            .data
            .get(D::KEY)
            .map(|m| m.value().downcast_ref::<D>().unwrap().clone());
        Ok(data)
    }

    async fn insert_data<D: KeyedData>(&self, data: &D) -> Result<(), StorageError> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(StorageError::NotOpened);
        }

        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }

    async fn update_data<D: KeyedData>(&self, data: &D) -> Result<(), StorageError> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(StorageError::NotOpened);
        }

        if !self.data.contains_key(D::KEY) {
            panic!("Registration not present");
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::storage::{KeyedData, Storage};

    use super::MockStorage;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Data {
        a: u8,
        b: String,
    }

    impl KeyedData for Data {
        const KEY: &'static str = "test_data";
    }

    #[tokio::test]
    async fn it_works() {
        let mut storage = MockStorage::default();
        storage.open().await.unwrap();

        let data = Data {
            a: 32,
            b: "foo".to_string(),
        };

        storage.insert_data(&data).await.unwrap();

        let fetched = storage.fetch_data::<Data>().await.unwrap().unwrap();
        assert_eq!(data, fetched);

        let updated = Data {
            a: 64,
            b: "bar".to_string(),
        };

        storage.update_data(&updated).await.unwrap();

        let fetched = storage.fetch_data::<Data>().await.unwrap().unwrap();
        assert_eq!(updated, fetched);
    }
}
