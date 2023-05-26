use std::{any::Any, collections::HashMap};

use async_trait::async_trait;

use super::{
    data::{KeyedData, RegistrationData},
    Storage, StorageOpenedError, StorageState,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data: HashMap<&str, Box<dyn Any + Send + Sync>> = HashMap::new();

        if let Some(registration) = registration {
            data.insert(RegistrationData::KEY, Box::new(registration));
        }

        MockStorage { state, data }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new(StorageState::Uninitialized, None)
    }
}

#[async_trait]
impl Storage for MockStorage {
    type Error = StorageOpenedError;

    async fn state(&self) -> Result<StorageState, Self::Error> {
        Ok(self.state)
    }

    async fn open(&mut self) -> Result<(), Self::Error> {
        self.state = StorageState::Opened;

        Ok(())
    }

    async fn clear(&mut self) -> Result<(), Self::Error> {
        self.state = StorageState::Uninitialized;

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>, Self::Error> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(StorageOpenedError::NotOpened);
        }

        // If self.data contains the key for the requested type,
        // assume that its value is of that specific type.
        // Downcast it to the type using the Any trait, then return a cloned result.
        let data = self.data.get(D::KEY).map(|m| m.downcast_ref::<D>().unwrap()).cloned();

        Ok(data)
    }

    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<(), Self::Error> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(StorageOpenedError::NotOpened);
        }

        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }
}
