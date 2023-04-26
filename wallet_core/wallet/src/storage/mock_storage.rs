use anyhow::{anyhow, Result};

use super::{data::Registration, Storage};

#[derive(Debug, Default)]
pub struct MockStorage {
    registration: Option<Registration>,
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn get_registration(&self) -> Result<Option<Registration>> {
        let registration = self.registration.clone();

        Ok(registration)
    }

    async fn save_registration(&mut self, registration: &Registration) -> Result<()> {
        if self.registration.is_some() {
            return Err(anyhow!("Registration already present"));
        }

        self.registration = Some(registration.clone());

        Ok(())
    }
}
