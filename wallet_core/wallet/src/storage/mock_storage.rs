use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    data::{Keyed, Registration},
    Storage, StorageError, StorageState,
};

#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, String>,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<Registration>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(Registration::KEY, serde_json::to_string(&registration).unwrap());
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

    async fn fetch_data<D: Keyed + DeserializeOwned>(&self) -> Result<Option<D>> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow::Error::new(StorageError::NotOpened));
        }

        let data = self.data.get(D::KEY).map(|m| serde_json::from_str::<D>(m).unwrap());

        Ok(data)
    }

    async fn insert_data<D: Keyed + Serialize + Send + Sync>(&mut self, data: &D) -> Result<()> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow::Error::new(StorageError::NotOpened));
        }

        if self.data.contains_key(D::KEY) {
            return Err(anyhow!("Registration already present"));
        }

        self.data.insert(D::KEY, serde_json::to_string(data).unwrap());

        Ok(())
    }
}
