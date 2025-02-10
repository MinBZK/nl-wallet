use chrono::DateTime;
use chrono::Utc;
use tracing::info;
use tracing::instrument;

use error_category::sentry_capture_error;
use error_category::ErrorCategory;
use nl_wallet_mdoc::holder::ProposedDocumentAttributes;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::utils::x509::MdocCertificateExtension;
use platform_support::attested_key::AttestedKeyHolder;
use wallet_common::update_policy::VersionState;

pub use crate::storage::EventStatus;

use crate::document::DisclosureType;
use crate::document::DocumentMdocError;
use crate::errors::StorageError;
use crate::repository::Repository;
use crate::storage::EventDocuments;
use crate::storage::Storage;
use crate::storage::WalletEvent;
use crate::DisclosureDocument;
use crate::Document;
use crate::DocumentPersistence;

use super::Wallet;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum HistoryError {
    #[category(expected)]
    #[error("app version is blocked")]
    VersionBlocked,
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,
    #[error("could not access history database: {0}")]
    EventStorage(#[from] EventStorageError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum EventStorageError {
    #[error("could not access event in history database: {0}")]
    Storage(#[from] StorageError),
    #[error("could not convert event items for display: {0}")]
    Conversion(#[from] EventConversionError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum EventConversionError {
    #[error("could not prepare event for UI: {0}")]
    Mapping(#[from] DocumentMdocError),
    #[error("could not read organization info from certificate: {0}")]
    Certificate(#[from] CertificateError),
    #[error("certificate does not contain reader registration")]
    #[category(critical)]
    NoReaderRegistrationFound,
    #[error("certificate does not contain issuer registration")]
    #[category(critical)]
    NoIssuerRegistrationFound,
}

impl From<StorageError> for HistoryError {
    fn from(value: StorageError) -> Self {
        EventStorageError::Storage(value).into()
    }
}

impl From<EventConversionError> for HistoryError {
    fn from(value: EventConversionError) -> Self {
        EventStorageError::Conversion(value).into()
    }
}

type HistoryResult<T> = Result<T, HistoryError>;

pub type RecentHistoryCallback = Box<dyn FnMut(Vec<HistoryEvent>) + Send + Sync>;

impl<CR, UR, S, AKH, APC, DS, IS, MDS, WIC> Wallet<CR, UR, S, AKH, APC, DS, IS, MDS, WIC>
where
    S: Storage,
    UR: Repository<VersionState>,
    AKH: AttestedKeyHolder,
{
    pub(super) async fn store_history_event(&mut self, event: WalletEvent) -> Result<(), EventStorageError> {
        info!("Storing history event");
        self.storage.write().await.log_wallet_event(event).await?;

        info!("Emitting recent history");
        self.emit_recent_history().await
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_history(&self) -> HistoryResult<Vec<HistoryEvent>> {
        info!("Retrieving history");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(HistoryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
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

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn get_history_for_card(&self, doc_type: &str) -> HistoryResult<Vec<HistoryEvent>> {
        info!("Retrieving Card history");

        info!("Checking if blocked");
        if self.is_blocked() {
            return Err(HistoryError::VersionBlocked);
        }

        info!("Checking if registered");
        if !self.registration.is_registered() {
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

    async fn emit_recent_history(&mut self) -> Result<(), EventStorageError> {
        info!("Emit recent history from storage");

        let storage = self.storage.read().await;
        let events: Vec<HistoryEvent> = storage
            .fetch_recent_wallet_events()
            .await?
            .into_iter()
            .map(HistoryEvent::try_from)
            .collect::<Result<_, _>>()?;

        if let Some(ref mut recent_history_callback) = self.recent_history_callback {
            recent_history_callback(events);
        }

        Ok(())
    }

    #[sentry_capture_error]
    pub async fn set_recent_history_callback(
        &mut self,
        callback: RecentHistoryCallback,
    ) -> HistoryResult<Option<RecentHistoryCallback>> {
        let previous_callback = self.recent_history_callback.replace(Box::new(callback));

        // If the `Wallet` is not registered, the database will not be open.
        // In that case don't emit anything.
        if self.registration.is_registered() {
            self.emit_recent_history().await?;
        }

        Ok(previous_callback)
    }

    pub fn clear_recent_history_callback(&mut self) {
        self.recent_history_callback.take();
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
        r#type: DisclosureType,
        timestamp: DateTime<Utc>,
        reader_registration: Box<ReaderRegistration>,
        attributes: Option<Vec<DisclosureDocument>>,
    },
}

impl TryFrom<WalletEvent> for HistoryEvent {
    type Error = EventConversionError;

    fn try_from(source: WalletEvent) -> Result<Self, Self::Error> {
        let result = match source {
            WalletEvent::Issuance {
                id: _,
                timestamp,
                mdocs,
            } => Self::Issuance {
                timestamp,
                mdocs: mdocs
                    .0
                    .into_iter()
                    .map(|(doc_type, proposed_card)| {
                        let issuer_registration = IssuerRegistration::from_certificate(&proposed_card.issuer)?
                            .ok_or(EventConversionError::NoIssuerRegistrationFound)?;

                        let document = Document::from_mdoc_attributes(
                            DocumentPersistence::InMemory,
                            &doc_type,
                            proposed_card.into(),
                            issuer_registration,
                        )?;
                        Ok(document)
                    })
                    .collect::<Result<_, EventConversionError>>()?,
            },
            WalletEvent::Disclosure {
                id: _,
                reader_certificate,
                timestamp,
                documents,
                status,
                r#type,
            } => Self::Disclosure {
                status,
                r#type,
                timestamp,
                attributes: documents
                    .map(|EventDocuments(mdocs)| {
                        mdocs
                            .into_iter()
                            .map(|(doc_type, namespaces)| {
                                DisclosureDocument::from_mdoc_attributes(
                                    &doc_type,
                                    ProposedDocumentAttributes {
                                        issuer: namespaces.issuer.clone(),
                                        attributes: namespaces.into(),
                                    },
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?,
                reader_registration: {
                    let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)?
                        .ok_or(EventConversionError::NoReaderRegistrationFound)?;
                    Box::new(reader_registration)
                },
            },
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use chrono::Duration;
    use chrono::TimeZone;
    use chrono::Utc;

    use nl_wallet_mdoc::server_keys::generate::Ca;
    use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;

    use super::Wallet;

    use crate::storage::WalletEvent;
    use crate::HistoryEvent;

    use super::super::test;
    use super::super::test::WalletDeviceVendor;
    use super::super::test::WalletWithMocks;
    use super::super::test::ISSUER_KEY;
    use super::EventStorageError;
    use super::HistoryError;

    const PID_DOCTYPE: &str = "com.example.pid";
    const ADDRESS_DOCTYPE: &str = "com.example.address";

    #[tokio::test]
    async fn test_history_fails_when_not_registered() {
        let wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

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
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

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
        let mut wallet = WalletWithMocks::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        let reader_ca = Ca::generate_reader_mock_ca().unwrap();
        let reader_key = reader_ca
            .generate_reader_mock(ReaderRegistration::new_mock().into())
            .unwrap();

        // history should be empty
        let history = wallet.get_history().await.unwrap();
        assert!(history.is_empty());

        let timestamp_older = Utc.with_ymd_and_hms(2023, 11, 11, 11, 11, 00).unwrap();
        let timestamp_newer = Utc.with_ymd_and_hms(2023, 11, 21, 13, 37, 00).unwrap();

        let pid_doc_type_event =
            WalletEvent::issuance_from_str(&[PID_DOCTYPE], timestamp_older, ISSUER_KEY.issuance_key.certificate());
        wallet.store_history_event(pid_doc_type_event.clone()).await.unwrap();

        let disclosure_cancelled_event =
            WalletEvent::disclosure_cancel(timestamp_older + Duration::days(1), reader_key.certificate().clone());
        wallet
            .store_history_event(disclosure_cancelled_event.clone())
            .await
            .unwrap();

        let disclosure_error_event =
            WalletEvent::disclosure_error(timestamp_older + Duration::days(2), reader_key.certificate().clone());
        wallet
            .store_history_event(disclosure_error_event.clone())
            .await
            .unwrap();

        let address_doc_type_event = WalletEvent::disclosure_from_str(
            &[ADDRESS_DOCTYPE],
            timestamp_newer,
            reader_key.certificate().clone(),
            ISSUER_KEY.issuance_key.certificate(),
        );
        wallet
            .store_history_event(address_doc_type_event.clone())
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

    // Tests both setting and clearing the recent_history callback on an unregistered `Wallet`.
    #[tokio::test]
    async fn test_set_clear_recent_history_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered(WalletDeviceVendor::Apple);

        // Register mock recent history callback
        let events = test::setup_mock_recent_history_callback(&mut wallet)
            .await
            .expect("Failed to set mock recent history callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&events), 2);

        // Confirm that we received an empty `Vec` in the callback.
        {
            let events = events.lock();
            assert!(events.is_empty());
        }

        // Clear the recent_history callback on the `Wallet.`
        wallet.clear_recent_history_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&events), 1);
    }

    // Tests both setting and clearing the recent_history callback on a registered `Wallet`.
    #[tokio::test]
    async fn test_set_clear_recent_history_callback_registered() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // The database contains a single Issuance Event
        let event = WalletEvent::new_issuance(Default::default());
        wallet.storage.write().await.event_log.push(event);

        // Register mock recent history callback
        let events = test::setup_mock_recent_history_callback(&mut wallet)
            .await
            .expect("Failed to set mock recent history callback");

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&events), 2);

        // Confirm that we received a single Issuance event on the callback.
        {
            let events = events.lock().pop().unwrap();

            let event = events
                .first()
                .expect("Recent history callback should have been provided an issuance event");
            assert_matches!(event, HistoryEvent::Issuance { .. });
        }

        // Clear the recent_history callback on the `Wallet.`
        wallet.clear_recent_history_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&events), 1);
    }

    #[tokio::test]
    async fn test_set_recent_history_callback_error() {
        let mut wallet = Wallet::new_registered_and_unlocked(WalletDeviceVendor::Apple);

        // Have the database return an error on query.
        wallet.storage.write().await.has_query_error = true;

        // Confirm that setting the callback returns an error.
        let error = wallet
            .set_recent_history_callback(Box::new(|_| {}))
            .await
            .map(|_| ())
            .expect_err("Setting recent_history callback should have resulted in an error");

        assert_matches!(error, HistoryError::EventStorage(EventStorageError::Storage(_)));
    }
}
