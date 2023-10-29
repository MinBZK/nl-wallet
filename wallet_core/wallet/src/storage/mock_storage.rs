use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{Mdoc, MdocCopies},
    utils::mdocs_map::MdocsMap,
};

use super::{
    data::{KeyedData, RegistrationData},
    Storage, StorageResult, StorageState,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, String>,
    pub mdocs: MdocsMap,
    pub has_query_error: bool,
}

impl MockStorage {
    pub fn mock(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(RegistrationData::KEY, serde_json::to_string(&registration).unwrap());
        }

        let mdocs = MdocsMap::new();

        MockStorage {
            state,
            data,
            mdocs,
            has_query_error: false,
        }
    }

    fn check_query_error(&self) -> StorageResult<()> {
        if self.has_query_error {
            return Err(DbErr::Custom("Mock error".to_string()).into());
        }

        Ok(())
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::mock(StorageState::Uninitialized, None)
    }
}

#[async_trait]
impl Storage for MockStorage {
    async fn state(&self) -> StorageResult<StorageState> {
        Ok(self.state)
    }

    async fn open(&mut self) -> StorageResult<()> {
        self.state = StorageState::Opened;

        Ok(())
    }

    async fn clear(&mut self) -> StorageResult<()> {
        self.state = StorageState::Uninitialized;

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        self.check_query_error()?;

        let data = self.data.get(D::KEY).map(|s| serde_json::from_str(s).unwrap());

        Ok(data)
    }

    async fn insert_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()> {
        self.check_query_error()?;

        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, serde_json::to_string(&data).unwrap());

        Ok(())
    }

    async fn update_data<D: KeyedData + Sync>(&mut self, data: &D) -> StorageResult<()> {
        self.check_query_error()?;

        if !self.data.contains_key(D::KEY) {
            panic!("Registration not present");
        }

        self.data.insert(D::KEY, serde_json::to_string(&data).unwrap());

        Ok(())
    }

    async fn insert_mdocs(&mut self, mdocs: Vec<MdocCopies>) -> StorageResult<()> {
        self.check_query_error()?;

        self.mdocs.add(mdocs.into_iter().flatten()).unwrap();

        Ok(())
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<(Uuid, Mdoc)>> {
        self.check_query_error()?;

        // Get a single copy of every unique Mdoc, along with a random `Uuid`.
        let mdocs = self
            .mdocs
            .0
            .values()
            .flat_map(|doc_type_mdocs| doc_type_mdocs.values())
            .flat_map(|mdoc_copies| mdoc_copies.cred_copies.first())
            .map(|mdoc| (Uuid::new_v4(), mdoc.clone()))
            .collect();

        Ok(mdocs)
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<(Uuid, Mdoc)>> {
        // Get every unique Mdoc and filter them based on the requested doc types.
        let unique_mdocs = self.fetch_unique_mdocs().await?;

        let mdocs = unique_mdocs
            .into_iter()
            .filter(|mdoc| doc_types.contains(mdoc.1.doc_type.as_str()))
            .collect();

        Ok(mdocs)
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
