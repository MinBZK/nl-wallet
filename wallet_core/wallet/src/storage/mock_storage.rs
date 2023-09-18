use std::collections::HashMap;

use async_trait::async_trait;

use super::{
    data::{KeyedData, RegistrationData},
    Storage, StorageError, StorageState,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, String>,
}

impl MockStorage {
    pub fn mock(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(RegistrationData::KEY, serde_json::to_string(&registration).unwrap());
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
        let data = self.data.get(D::KEY).map(|s| serde_json::from_str(s).unwrap());

        Ok(data)
    }

    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError> {
        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, serde_json::to_string(&data).unwrap());

        Ok(())
    }

    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> Result<(), StorageError> {
        if !self.data.contains_key(D::KEY) {
            panic!("Registration not present");
        }

        self.data.insert(D::KEY, serde_json::to_string(&data).unwrap());

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
