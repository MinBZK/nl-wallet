use std::{any::Any, collections::HashMap, path::PathBuf};

use async_trait::async_trait;

use super::{
    data::{KeyedData, RegistrationData},
    Storage, StorageError, StorageState,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl MockStorage {
    pub fn mock(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data: HashMap<&str, Box<dyn Any + Send + Sync>> = HashMap::new();

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
        // If self.data contains the key for the requested type,
        // assume that its value is of that specific type.
        // Downcast it to the type using the Any trait, then return a cloned result.
        let data = self.data.get(D::KEY).map(|m| m.downcast_ref::<D>().unwrap()).cloned();

        Ok(data)
    }

    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<(), StorageError> {
        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }

    async fn update_data<D: KeyedData>(&mut self, data: &D) -> Result<(), StorageError> {
        if !self.data.contains_key(D::KEY) {
            panic!("Registration not present");
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }
}
