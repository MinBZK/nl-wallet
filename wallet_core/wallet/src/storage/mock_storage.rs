use anyhow::{anyhow, Result};

use super::{data::Registration, Storage, StorageError, StorageState};

#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub registration: Option<Registration>,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<Registration>) -> Self {
        MockStorage { state, registration }
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new(StorageState::Uninitialized, None)
    }
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn get_state(&self) -> Result<StorageState> {
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

    async fn get_registration(&self) -> Result<Option<Registration>> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow::Error::new(StorageError::NotOpened));
        }

        let registration = self.registration.clone();

        Ok(registration)
    }

    async fn insert_registration(&mut self, registration: &Registration) -> Result<()> {
        if !matches!(self.state, StorageState::Opened) {
            return Err(anyhow::Error::new(StorageError::NotOpened));
        }

        if self.registration.is_some() {
            return Err(anyhow!("Registration already present"));
        }

        self.registration = Some(registration.clone());

        Ok(())
    }
}
