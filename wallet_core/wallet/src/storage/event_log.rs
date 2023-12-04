use chrono::{DateTime, Utc};
use uuid::Uuid;

pub use entity::event_log::EventType;
use entity::event_log::Model;
use nl_wallet_mdoc::{utils::x509::Certificate, DocType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventStatus {
    Success,
    Error(String),
    Cancelled,
}

impl EventStatus {
    pub fn description(&self) -> Option<&str> {
        if let EventStatus::Error(description) = self {
            Some(description)
        } else {
            None
        }
    }
}

impl From<EventStatus> for entity::event_log::EventStatus {
    fn from(source: EventStatus) -> Self {
        match source {
            EventStatus::Success => Self::Success,
            EventStatus::Error(_) => Self::Error,
            EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&Model> for EventStatus {
    fn from(source: &Model) -> Self {
        use entity::event_log::EventStatus::*;
        match source.status {
            Success => Self::Success,
            Error => {
                // unwrap is safe here, assuming the data has been inserted using [Status]
                Self::Error(source.status_description.as_ref().unwrap().to_owned())
            }
            Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub doc_type: DocType,
    pub timestamp: DateTime<Utc>,
    pub remote_party_certificate: Certificate,
    pub status: EventStatus,
}

impl WalletEvent {
    pub fn new(
        event_type: EventType,
        doc_type: DocType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        status: EventStatus,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            doc_type,
            timestamp,
            remote_party_certificate,
            status,
        }
    }

    pub fn issuance_success(
        doc_type: DocType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
    ) -> Self {
        Self::new(
            EventType::Issuance,
            doc_type,
            timestamp,
            remote_party_certificate,
            EventStatus::Success,
        )
    }

    pub fn disclosure_cancelled(
        doc_type: DocType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
    ) -> Self {
        Self::new(
            EventType::Disclosure,
            doc_type,
            timestamp,
            remote_party_certificate,
            EventStatus::Cancelled,
        )
    }

    pub fn disclosure_error(
        doc_type: DocType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        reason: String,
    ) -> Self {
        Self::new(
            EventType::Disclosure,
            doc_type,
            timestamp,
            remote_party_certificate,
            EventStatus::Error(reason),
        )
    }
}

impl From<Model> for WalletEvent {
    fn from(source: Model) -> Self {
        Self {
            id: source.id,
            status: EventStatus::from(&source),
            event_type: source.event_type,
            doc_type: source.doc_type,
            timestamp: source.timestamp,
            remote_party_certificate: source.remote_party_certificate.into(),
        }
    }
}

impl From<WalletEvent> for Model {
    fn from(source: WalletEvent) -> Self {
        Self {
            id: source.id,
            event_type: source.event_type,
            doc_type: source.doc_type,
            timestamp: source.timestamp,
            remote_party_certificate: source.remote_party_certificate.as_bytes().to_owned(),
            status_description: source.status.description().map(|d| d.to_owned()),
            status: source.status.into(),
        }
    }
}
