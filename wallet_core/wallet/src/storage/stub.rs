use std::collections::HashMap;
use std::collections::HashSet;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use indexmap::IndexMap;
use itertools::Itertools;
use sea_orm::DbErr;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use openid4vc::issuance_session::CredentialWithMetadata;
use openid4vc::issuance_session::IssuedCredential;
use openid4vc::issuance_session::IssuedCredentialCopies;

use crate::AttestationPresentation;
use crate::DisclosureStatus;

use super::Storage;
use super::StorageError;
use super::StorageResult;
use super::StorageState;
use super::StoredAttestationCopy;
use super::StoredAttestationFormat;
use super::StoredMdocCopy;
use super::data::KeyedData;
use super::data::RegistrationData;
use super::event_log::WalletEvent;

#[derive(Debug)]
pub enum KeyedDataResult {
    Data(String),
    Error,
}

/// This is a mock implementation of [`Storage`], used for testing [`crate::Wallet`].
#[derive(Debug)]
pub struct StorageStub {
    pub state: StorageState,
    pub data: HashMap<&'static str, KeyedDataResult>,
    pub issued_credential_copies: IndexMap<String, Vec<CredentialWithMetadata>>,
    pub attestation_copies_usage_counts: HashMap<Uuid, u32>,
    pub event_log: Vec<WalletEvent>,
    pub has_query_error: bool,
}

impl StorageStub {
    pub fn new(state: StorageState, registration: Option<RegistrationData>) -> Self {
        let mut data = HashMap::new();

        if let Some(registration) = registration {
            data.insert(
                RegistrationData::KEY,
                KeyedDataResult::Data(serde_json::to_string(&registration).unwrap()),
            );
        }

        StorageStub {
            state,
            data,
            issued_credential_copies: IndexMap::new(),
            attestation_copies_usage_counts: HashMap::new(),
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

impl Default for StorageStub {
    fn default() -> Self {
        Self::new(StorageState::Uninitialized, None)
    }
}

impl Storage for StorageStub {
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

    async fn insert_credentials(
        &mut self,
        timestamp: DateTime<Utc>,
        credentials: Vec<(CredentialWithMetadata, AttestationPresentation)>,
    ) -> StorageResult<()> {
        self.check_query_error()?;

        for (credential, attestation) in credentials {
            self.issued_credential_copies
                .entry(credential.attestation_type.clone())
                .or_default()
                .push(credential);

            self.event_log.push(WalletEvent::Issuance {
                id: Uuid::new_v4(),
                attestation: Box::new(attestation),
                timestamp,
                renewed: false,
            });
        }

        Ok(())
    }

    async fn update_credentials(
        &mut self,
        _timestamp: DateTime<Utc>,
        _credentials: Vec<(IssuedCredentialCopies, AttestationPresentation)>,
    ) -> StorageResult<()> {
        unimplemented!()
    }

    async fn increment_attestation_copies_usage_count(&mut self, attestation_copy_ids: Vec<Uuid>) -> StorageResult<()> {
        self.check_query_error()?;

        attestation_copy_ids.into_iter().for_each(|mdoc_copy_id| {
            self.attestation_copies_usage_counts
                .entry(mdoc_copy_id)
                .and_modify(|usage_count| *usage_count += 1)
                .or_insert(1);
        });

        Ok(())
    }

    async fn fetch_unique_attestations(&self) -> StorageResult<Vec<StoredAttestationCopy>> {
        self.check_query_error()?;

        let attestations = self
            .issued_credential_copies
            .values()
            .flatten()
            .map(|credential| {
                let attestation = match credential.copies.as_ref().first() {
                    IssuedCredential::MsoMdoc(mdoc) => StoredAttestationFormat::MsoMdoc { mdoc: mdoc.clone() },
                    IssuedCredential::SdJwt(sd_jwt) => StoredAttestationFormat::SdJwt { sd_jwt: sd_jwt.clone() },
                };

                Ok::<_, StorageError>(StoredAttestationCopy {
                    attestation_id: Uuid::now_v7(),
                    attestation_copy_id: Uuid::now_v7(),
                    attestation,
                    normalized_metadata: credential.metadata_documents.to_normalized()?,
                })
            })
            .try_collect()?;

        Ok(attestations)
    }

    async fn fetch_unique_attestations_by_type<'a>(
        &'a self,
        attestation_types: &HashSet<&'a str>,
    ) -> StorageResult<Vec<StoredAttestationCopy>> {
        Ok(self
            .fetch_unique_attestations()
            .await?
            .into_iter()
            .filter(|attestation| attestation_types.contains(attestation.attestation_id.to_string().as_str()))
            .collect_vec())
    }

    async fn has_any_attestations_with_type(&self, attestation_type: &str) -> StorageResult<bool> {
        Ok(!self
            .fetch_unique_attestations()
            .await
            .unwrap()
            .into_iter()
            .filter(|copy| match &copy.attestation {
                StoredAttestationFormat::MsoMdoc { mdoc } => mdoc.doc_type() == attestation_type,
                StoredAttestationFormat::SdJwt { sd_jwt } => {
                    sd_jwt.as_ref().as_ref().claims().properties.get("vct").unwrap() == attestation_type
                }
            })
            .collect_vec()
            .is_empty())
    }

    async fn fetch_unique_mdocs_by_doctypes<'a>(
        &'a self,
        doc_types: &HashSet<&'a str>,
    ) -> StorageResult<Vec<StoredMdocCopy>> {
        // Get every unique Mdoc and filter them based on the requested doc types.
        let copies = self.fetch_unique_attestations().await?;

        let mdocs = copies
            .into_iter()
            .filter_map(|copy| {
                match copy.attestation {
                    StoredAttestationFormat::MsoMdoc { mdoc } if doc_types.contains(mdoc.doc_type().as_str()) => {
                        Some(*mdoc)
                    }
                    _ => None,
                }
                .map(|mdoc| StoredMdocCopy {
                    mdoc_id: copy.attestation_id,
                    mdoc_copy_id: copy.attestation_copy_id,
                    mdoc,
                    normalized_metadata: copy.normalized_metadata,
                })
            })
            .collect();

        Ok(mdocs)
    }

    async fn log_disclosure_event(
        &mut self,
        timestamp: DateTime<Utc>,
        proposed_attestation_presentations: Vec<AttestationPresentation>,
        reader_certificate: BorrowingCertificate,
        status: DisclosureStatus,
        r#type: DisclosureType,
    ) -> StorageResult<()> {
        let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
            .unwrap()
            .unwrap();

        self.event_log.push(WalletEvent::Disclosure {
            id: Uuid::new_v4(),
            attestations: proposed_attestation_presentations,
            timestamp,
            reader_certificate: Box::new(reader_certificate),
            reader_registration: Box::new(reader_registration),
            status,
            r#type,
        });

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

    async fn fetch_wallet_events_by_attestation_id(&self, attestation_id: Uuid) -> StorageResult<Vec<WalletEvent>> {
        self.check_query_error()?;

        let mut events: Vec<_> = self
            .event_log
            .iter()
            .filter(|event| match *event {
                WalletEvent::Issuance { id, .. } => id == &attestation_id,
                WalletEvent::Disclosure { id, .. } => id == &attestation_id,
            })
            .cloned()
            .collect();
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
    use crate::storage::KeyedData;
    use crate::storage::Storage;
    use crate::storage::database_storage::tests::test_history_ordering;
    use serde::Deserialize;
    use serde::Serialize;

    use super::StorageStub;

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
        let mut storage = StorageStub::default();
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
        let mut storage = StorageStub::default();
        storage.open().await.unwrap();
        test_history_ordering(&mut storage).await;
    }
}
