use std::{any::Any, collections::HashMap};

use anyhow::{anyhow, Result};

use super::{
    data::{KeyedData, Registration},
    Storage, StorageError, StorageState,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<Registration>) -> Self {
        let mut data: HashMap<&str, Box<dyn Any + Send + Sync>> = HashMap::new();

        if let Some(registration) = registration {
            data.insert(Registration::KEY, Box::new(registration));
        }

        MockStorage { state, data }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new(StorageState::Uninitialized, None)
    }
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn state(&self) -> Result<StorageState> {
        Ok(self.state)
    }

    async fn open(&mut self) -> Result<()> {
        self.state = StorageState::Opened;

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.state = StorageState::Uninitialized;

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> Result<Option<D>> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow!(StorageError::NotOpened));
        }

        // If self.data contains the key for the requested type,
        // assume that its value is of that specific type.
        // Downcast it to the type using the Any trait, then return a cloned result.
        let data = self.data.get(D::KEY).map(|m| m.downcast_ref::<D>().unwrap()).cloned();

        Ok(data)
    }

    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> Result<()> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow!(StorageError::NotOpened));
        }

        if self.data.contains_key(D::KEY) {
            return Err(anyhow!("Registration already present"));
        }

        self.data.insert(D::KEY, Box::new(data.clone()));

        Ok(())
    }
}
