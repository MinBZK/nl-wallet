use chrono::{DateTime, Utc};
use tracing::info;

use nl_wallet_mdoc::utils::{
    reader_auth::ReaderRegistration,
    x509::{CertificateError, CertificateType},
};

pub use crate::storage::EventStatus;
use crate::{
    document::DocumentMdocError,
    errors::StorageError,
    storage::{DocTypeMap, Storage, WalletEvent},
    DisclosureDocument, Document, DocumentPersistence,
};

use super::Wallet;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("wallet is not registered")]
    NotRegistered,
    #[error("wallet is locked")]
    Locked,
    #[error("could not access history database: {0}")]
    Storage(#[from] StorageError),
    #[error("could not prepare history event for UI: {0}")]
    Mapping(#[from] DocumentMdocError),
    #[error("could not read organization info from certificate: {0}")]
    Certificate(#[from] CertificateError),
    #[error("certificate does not contain reader registration")]
    NotAReaderCertificate,
}

type HistoryResult<T> = Result<T, HistoryError>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    S: Storage,
{
    pub async fn get_history(&self) -> HistoryResult<Vec<HistoryEvent>> {
        info!("Retrieving history");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(HistoryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(HistoryError::Locked);
        }

        info!("Retrieving history from storage");
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events().await?;
        let result = events.into_iter().map(TryFrom::try_from).collect::<Result<_, _>>()?;
        Ok(result)
    }

    pub async fn get_history_for_card(&self, doc_type: &str) -> HistoryResult<Vec<HistoryEvent>> {
        info!("Retrieving Card history");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(HistoryError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(HistoryError::Locked);
        }

        info!("Retrieving Card history from storage");
        let storage = self.storage.read().await;
        let events = storage.fetch_wallet_events_by_doc_type(doc_type).await?;
        let result = events.into_iter().map(TryFrom::try_from).collect::<Result<_, _>>()?;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryEvent {
    Issuance {
        timestamp: DateTime<Utc>,
        mdocs: Vec<Document>,
    },
    Disclosure {
        status: EventStatus,
        timestamp: DateTime<Utc>,
        reader_registration: Box<ReaderRegistration>,
        attributes: Option<Vec<DisclosureDocument>>,
    },
}

impl TryFrom<WalletEvent> for HistoryEvent {
    type Error = HistoryError;

    fn try_from(source: WalletEvent) -> Result<Self, Self::Error> {
        let result = match source {
            WalletEvent::Issuance {
                id: _,
                remote_party_certificate: _,
                timestamp,
                mdocs,
            } => Self::Issuance {
                timestamp,
                mdocs: mdocs
                    .0
                    .into_iter()
                    .map(|(doc_type, namespaces)| {
                        // TODO: Refer to persisted mdoc from the mdoc table, or not?
                        Document::from_mdoc_attributes(DocumentPersistence::InMemory, &doc_type, namespaces)
                    })
                    .collect::<Result<_, _>>()?,
            },
            WalletEvent::Disclosure {
                id: _,
                remote_party_certificate,
                timestamp,
                documents,
                status,
            } => Self::Disclosure {
                status,
                timestamp,
                attributes: documents
                    .map(|DocTypeMap(mdocs)| {
                        mdocs
                            .into_iter()
                            .map(|(doc_type, namespaces)| {
                                DisclosureDocument::from_mdoc_attributes(&doc_type, namespaces)
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?,
                reader_registration: {
                    let certificate_type = CertificateType::from_certificate(&remote_party_certificate)?;
                    let reader_registration =
                        if let CertificateType::ReaderAuth(Some(reader_registration)) = certificate_type {
                            *reader_registration
                        } else {
                            return Err(HistoryError::NotAReaderCertificate);
                        };
                    Box::new(reader_registration)
                },
            },
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use chrono::{Duration, TimeZone, Utc};
    use nl_wallet_mdoc::utils::x509::{Certificate, CertificateType};

    use crate::{storage::Storage, storage::WalletEvent, wallet::tests::WalletWithMocks};

    use super::HistoryError;

    const PID_DOCTYPE: &str = "com.example.pid";
    const ADDRESS_DOCTYPE: &str = "com.example.address";

    #[tokio::test]
    async fn test_history_fails_when_not_registered() {
        let wallet = WalletWithMocks::new_unregistered().await;

        let error = wallet
            .get_history()
            .await
            .expect_err("Expect error when Wallet is not registered");
        assert_matches!(error, HistoryError::NotRegistered);

        let error = wallet
            .get_history_for_card(PID_DOCTYPE)
            .await
            .expect_err("Expect error when Wallet is not registered");
        assert_matches!(error, HistoryError::NotRegistered);
    }

    #[tokio::test]
    async fn test_history_fails_when_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        let error = wallet
            .get_history()
            .await
            .expect_err("Expect error when Wallet is locked");
        assert_matches!(error, HistoryError::Locked);

        let error = wallet
            .get_history_for_card(PID_DOCTYPE)
            .await
            .expect_err("Expect error when Wallet is locked");
        assert_matches!(error, HistoryError::Locked);
    }

    #[tokio::test]
    async fn test_history() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        let (ca_cert, ca_key) = Certificate::new_ca("test-ca").unwrap();
        let (certificate, _) = Certificate::new(
            &ca_cert,
            &ca_key,
            "test-certificate",
            CertificateType::ReaderAuth(Some(Box::default())),
        )
        .unwrap();

        // history should be empty
        let history = wallet.get_history().await.unwrap();
        assert_eq!(history, vec![]);

        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();
        let timestamp_newer = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();

        let pid_doc_type_event =
            WalletEvent::issuance_from_str(vec![PID_DOCTYPE], timestamp_older, certificate.clone());
        wallet
            .storage
            .get_mut()
            .log_wallet_event(pid_doc_type_event.clone())
            .await
            .unwrap();

        let disclosure_cancelled_event =
            WalletEvent::disclosure_cancel(timestamp_older + Duration::days(1), certificate.clone());
        wallet
            .storage
            .get_mut()
            .log_wallet_event(disclosure_cancelled_event.clone())
            .await
            .unwrap();

        let disclosure_error_event = WalletEvent::disclosure_error(
            timestamp_older + Duration::days(2),
            certificate.clone(),
            "Some Error".to_owned(),
        );
        wallet
            .storage
            .get_mut()
            .log_wallet_event(disclosure_error_event.clone())
            .await
            .unwrap();

        let address_doc_type_event =
            WalletEvent::disclosure_from_str(vec![ADDRESS_DOCTYPE], timestamp_newer, certificate);
        wallet
            .storage
            .get_mut()
            .log_wallet_event(address_doc_type_event.clone())
            .await
            .unwrap();

        // get history should return both events, in correct order, newest first
        let history = wallet.get_history().await.unwrap();
        assert_eq!(
            history,
            vec![
                address_doc_type_event.clone().try_into().unwrap(),
                disclosure_error_event.try_into().unwrap(),
                disclosure_cancelled_event.try_into().unwrap(),
                pid_doc_type_event.clone().try_into().unwrap()
            ]
        );

        // get history for card should return single event
        let history = wallet.get_history_for_card(PID_DOCTYPE).await.unwrap();
        assert_eq!(history, vec![pid_doc_type_event.try_into().unwrap()]);

        let history = wallet.get_history_for_card(ADDRESS_DOCTYPE).await.unwrap();
        assert_eq!(history, vec![address_doc_type_event.try_into().unwrap()]);
    }
}
