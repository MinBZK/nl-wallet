use std::{collections::HashMap, path::PathBuf};

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
