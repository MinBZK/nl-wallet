use std::collections::HashMap;
use std::collections::HashSet;

use chrono::Duration;
use chrono::Utc;
use indexmap::IndexMap;
use sea_orm::DbErr;
use uuid::Uuid;

use crypto::x509::BorrowingCertificate;
use openid4vc::issuance_session::CredentialWithMetadata;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::issuance_session::IssuedCredentialCopies;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use super::data::KeyedData;
use super::data::RegistrationData;
use super::event_log::WalletEvent;
use super::Storage;
use super::StorageResult;
use super::StorageState;
use super::StoredAttestationCopy;
use super::StoredAttestationFormat;
use super::StoredMdocCopy;

#[derive(Debug)]
pub enum KeyedDataResult {
    Data(String),
    Error,
}

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct MockStorage {
    pub state: StorageState,
    pub data: HashMap<&'static str, KeyedDataResult>,
    pub issued_credential_copies: IndexMap<String, Vec<(IssuedCredentialCopies, NormalizedTypeMetadata)>>,
    pub mdoc_copies_usage_counts: HashMap<Uuid, u32>,
    pub event_log: Vec<WalletEvent>,
    pub has_query_error: bool,
}

impl MockStorage {
    pub fn new(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(
                RegistrationData::KEY,
                KeyedDataResult::Data(serde_json::to_string(&registration).unwrap()),
            );
        }

        MockStorage {
            state,
            data,
            issued_credential_copies: IndexMap::new(),
            mdoc_copies_usage_counts: HashMap::new(),
            event_log: vec![],
            has_query_error: false,
        }
    }

    pub fn set_keyed_data_error(&mut self, key: &'static str) {
        self.data.insert(key, KeyedDataResult::Error);
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

    async fn clear(&mut self) {
        self.data.clear();
        self.state = StorageState::Uninitialized;
    }

    async fn fetch_data<D: KeyedData>(&self) -> StorageResult<Option<D>> {
        let data = self.data.get(D::KEY);

        match data {
            Some(KeyedDataResult::Data(data)) => Ok(Some(serde_json::from_str(data)?)),
            Some(KeyedDataResult::Error) => Err(DbErr::Custom("Mock error".to_string()).into()),
            None => Ok(None),
        }
    }

    async fn insert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        match self.data.get(D::KEY) {
            Some(KeyedDataResult::Data(_)) => panic!("{} already present", D::KEY),
            Some(KeyedDataResult::Error) => return Err(DbErr::Custom("Mock error".to_string()).into()),
            None => (),
        }

        self.data
            .insert(D::KEY, KeyedDataResult::Data(serde_json::to_string(&data)?));

        Ok(())
    }

    async fn upsert_data<D: KeyedData>(&mut self, data: &D) -> StorageResult<()> {
        if let Some(KeyedDataResult::Error) = self.data.get(D::KEY) {
            return Err(DbErr::Custom("Mock error".to_string()).into());
        }

        self.data
            .insert(D::KEY, KeyedDataResult::Data(serde_json::to_string(&data)?));

        Ok(())
    }

    async fn delete_data<D: KeyedData>(&mut self) -> StorageResult<()> {
        self.data.remove(D::KEY);

        Ok(())
    }

    async fn insert_credentials(&mut self, credentials: Vec<CredentialWithMetadata>) -> StorageResult<()> {
        self.check_query_error()?;

        for CredentialWithMetadata {
            copies,
            attestation_type,
            metadata_documents,
        } in credentials
        {
            self.issued_credential_copies
                .entry(attestation_type)
                .or_default()
                .push((copies, metadata_documents.to_normalized()?));
        }

        Ok(())
    }

    async fn increment_attestation_copies_usage_count(&mut self, attestation_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        attestation_copy_ids.into_iter().for_each(|mdoc_copy_id| {
            self.mdoc_copies_usage_counts
                .entry(mdoc_copy_id)
                .and_modify(|usage_count| *usage_count += 1)
                .or_insert(1);
        });

        Ok(())
    }

    async fn fetch_unique_attestations(&self) -> StorageResult<Vec<StoredAttestationCopy>> {
        self.check_query_error()?;

        let mdocs = self
            .issued_credential_copies
            .values()
            .flatten()
            .map(|(credential_copies, metadata)| {
                let attestation = match credential_copies.as_ref().first() {
                    IssuedCredential::MsoMdoc(mdoc) => StoredAttestationFormat::MsoMdoc { mdoc: mdoc.clone() },
                    IssuedCredential::SdJwt(sd_jwt) => StoredAttestationFormat::SdJwt { sd_jwt: sd_jwt.clone() },
                };

                StoredAttestationCopy {
                    attestation_id: Uuid::now_v7(),
                    attestation_copy_id: Uuid::now_v7(),
                    attestation,
                    normalized_metadata: metadata.clone(),
                }
            })
            .collect();

        Ok(mdocs)
    }

    async fn fetch_unique_mdocs(&self) -> StorageResult<Vec<StoredMdocCopy>> {
        self.check_query_error()?;

        // Get a single copy of every unique Mdoc, along with a random `Uuid`.
        let mdocs = self
            .issued_credential_copies
            .values()
            .flatten()
            .filter_map(
                |(credential_copies, metadata)| match credential_copies.as_ref().first() {
                    IssuedCredential::MsoMdoc(mdoc) => Some(StoredMdocCopy {
                        mdoc_id: Uuid::now_v7(),
                        mdoc_copy_id: Uuid::now_v7(),
                        mdoc: *mdoc.clone(),
                        normalized_metadata: metadata.clone(),
                    }),
                    IssuedCredential::SdJwt(_) => None,
                },
            )
            .collect();

        Ok(mdocs)
    }

    async fn fetch_unique_mdocs_by_doctypes(&self, doc_types: &HashSet<&str>) -> StorageResult<Vec<StoredMdocCopy>> {
        // Get every unique Mdoc and filter them based on the requested doc types.
        let mdoc_copies = self.fetch_unique_mdocs().await?;

        let mdocs = mdoc_copies
            .into_iter()
            .filter(|mdoc_copy| doc_types.contains(mdoc_copy.mdoc.doc_type().as_str()))
            .collect();

        Ok(mdocs)
    }

    async fn has_any_mdocs_with_doctype(&self, doc_type: &str) -> StorageResult<bool> {
        let mdoc_copies = self.fetch_unique_mdocs().await?;

        let result = mdoc_copies
            .into_iter()
            .any(|mdoc_copy| doc_type == mdoc_copy.mdoc.doc_type().as_str());

        Ok(result)
    }

    async fn log_wallet_event(&mut self, event: WalletEvent) -> StorageResult<()> {
        self.event_log.push(event);
        Ok(())
    }

    async fn fetch_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        self.check_query_error()?;

        let mut events = self.event_log.to_vec();
        events.sort_by(|e1, e2| e2.timestamp().cmp(e1.timestamp()));
        Ok(events)
    }

    async fn fetch_recent_wallet_events(&self) -> StorageResult<Vec<WalletEvent>> {
        self.check_query_error()?;

        let mut events: Vec<_> = self
            .event_log
            .iter()
            .filter(|event| *event.timestamp() > Utc::now() - Duration::days(31))
            .cloned()
            .collect();
        events.sort_by(|e1, e2| e2.timestamp().cmp(e1.timestamp()));
        Ok(events)
    }

    async fn fetch_wallet_events_by_attestation_type(&self, attestation_type: &str) -> StorageResult<Vec<WalletEvent>> {
        self.check_query_error()?;

        let mut events = self
            .event_log
            .clone()
            .into_iter()
            .filter(|e| e.associated_attestation_types().contains(attestation_type))
            .collect::<Vec<_>>();
        events.sort_by(|e1, e2| e2.timestamp().cmp(e1.timestamp()));
        Ok(events)
    }

    async fn did_share_data_with_relying_party(&self, certificate: &BorrowingCertificate) -> StorageResult<bool> {
        self.check_query_error()?;

        let exists = self.event_log.iter().any(|event| match event {
            WalletEvent::Issuance { .. } => false,
            WalletEvent::Disclosure { reader_certificate, .. } => reader_certificate.as_ref() == certificate,
        });
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;

    use crate::storage::database_storage::tests::test_history_by_entity_type;
    use crate::storage::database_storage::tests::test_history_ordering;
    use crate::storage::KeyedData;
    use crate::storage::Storage;

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

        storage.upsert_data(&updated).await.unwrap();

        let fetched = storage.fetch_data::<Data>().await.unwrap().unwrap();
        assert_eq!(updated, fetched);

        storage.delete_data::<Data>().await.unwrap();
        assert!(storage.fetch_data::<Data>().await.unwrap().is_none());
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
        test_history_by_entity_type(&mut storage).await;
    }
}
