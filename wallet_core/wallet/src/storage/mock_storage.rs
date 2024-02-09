use std::collections::{HashMap, HashSet};

use sea_orm::DbErr;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::MdocCopies,
    utils::{mdocs_map::MdocsMap, x509::Certificate},
};

use crate::storage::event_log::WalletEventModel;

use super::{
    data::{KeyedData, RegistrationData},
    event_log::WalletEvent,
    Storage, StorageResult, StorageState, StoredMdocCopy,
};

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, String>,
    pub mdocs: MdocsMap,
    pub mdoc_copies_usage_counts: HashMap<Uuid, u32>,
    pub event_log: Vec<WalletEvent>,
    pub has_query_error: bool,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(RegistrationData::KEY, serde_json::to_string(&registration).unwrap());
        }

        let mdocs = MdocsMap::new();

        MockStorage {
            state,
            data,
            mdocs,
            mdoc_copies_usage_counts: HashMap::new(),
            event_log: vec![],
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
        Self::new(StorageState::Uninitialized, None)
    }
}

impl Storage for MockStorage {
    async fn state(&self) -> StorageResult<StorageState> {
        Ok(self.state)
    }

    async fn open(&mut self) -> StorageResult<()> {
        self.state = StorageState::Opened;

        Ok(())
    }

    async fn clear(&mut self) -> StorageResult<()> {
        self.data.clear();
        self.state = StorageState::Uninitialized;

        Ok(())
    }

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        self.check_query_error()?;

        let data = self.data.get(D::KEY).map(|s| serde_json::from_str(s).unwrap());

        Ok(data)
    }

    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        self.check_query_error()?;

        if self.data.contains_key(D::KEY) {
            panic!("Registration already present");
        }

        self.data.insert(D::KEY, serde_json::to_string(&data).unwrap());

        Ok(())
    }

    async fn update_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
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

    async fn increment_mdoc_copies_usage_count(&mut self, mdoc_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        mdoc_copy_ids.into_iter().for_each(|mdoc_copy_id| {
            self.mdoc_copies_usage_counts
                .entry(mdoc_copy_id)
                .and_modify(|usage_count| *usage_count += 1)
                .or_insert(1);
        });

        Ok(())
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>> {
        self.check_query_error()?;

        // Get a single copy of every unique Mdoc, along with a random `Uuid`.
        let mdocs = self
            .mdocs
            .0
            .values()
            .flat_map(|doc_type_mdocs| doc_type_mdocs.values())
            .flat_map(|mdoc_copies| mdoc_copies.cred_copies.first())
            .map(|mdoc| StoredMdocCopy {
                mdoc_id: Uuid::new_v4(),
                mdoc_copy_id: Uuid::new_v4(),
                mdoc: mdoc.clone(),
            })
            .collect();

        Ok(mdocs)
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>> {
        // Get every unique Mdoc and filter them based on the requested doc types.
        let mdoc_copies = self.fetch_unique_mdocs().await?;

        let mdocs = mdoc_copies
            .into_iter()
            .filter(|mdoc_copy| doc_types.contains(mdoc_copy.mdoc.doc_type.as_str()))
            .collect();

        Ok(mdocs)
    }

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()> {
        // Convert to database entity and back to check whether the `TryFrom` implementations are complete.
        let converted_event = match WalletEventModel::try_from(event.clone())? {
            WalletEventModel::Issuance(entity) => entity.try_into()?,
            WalletEventModel::Disclosure(entity) => entity.try_into()?,
        };
        assert_eq!(event, converted_event);
        self.event_log.push(converted_event);
        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        let mut events = self.event_log.to_vec();
        events.sort_by(|e1, e2| e2.timestamp().cmp(e1.timestamp()));
        Ok(events)
    }

    async fn fetch_wallet_events_by_doc_type(&self, doc_type: &str) -> StorageResult<Vec<WalletEvent>> {
        let mut events = self
            .event_log
            .clone()
            .into_iter()
            .filter(|e| e.associated_doc_types().contains(doc_type))
            .collect::<Vec<_>>();
        events.sort_by(|e1, e2| e2.timestamp().cmp(e1.timestamp()));
        Ok(events)
    }

    async fn did_share_data_with_relying_party(&self, certificate: &Certificate) -> StorageResult<bool> {
        let exists = self.event_log.iter().any(|event| match event {
            WalletEvent::Issuance { .. } => false,
            WalletEvent::Disclosure { reader_certificate, .. } => reader_certificate == certificate,
        });
        Ok(exists)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde::{Deserialize, Serialize};

    use crate::storage::{
        database_storage::tests::{test_history_by_doc_type, test_history_ordering},
        KeyedData, Storage,
    };

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

    #[tokio::test]
    async fn history_events_ordering() {
        let mut storage = MockStorage::default();
        storage.open().await.unwrap();
        test_history_ordering(&mut storage).await;
    }

    #[tokio::test]
    async fn history_events_work() {
        let mut storage = MockStorage::default();
        storage.open().await.unwrap();
        test_history_by_doc_type(&mut storage).await;
    }
}
