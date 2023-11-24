use chrono::{DateTime, Utc};
use uuid::Uuid;

pub use entity::event_log::EventType;
use entity::event_log::{EventStatus, Model};
use nl_wallet_mdoc::{utils::x509::Certificate, DocType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Success,
    Error(String),
    Cancelled,
}

impl Status {
    pub fn description(&self) -> Option<&str> {
        if let Status::Error(description) = self {
            Some(description)
        } else {
            None
        }
    }
}

impl From<Status> for entity::event_log::EventStatus {
    fn from(source: Status) -> Self {
        match source {
            Status::Success => Self::Success,
            Status::Error(_) => Self::Error,
            Status::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&Model> for Status {
    fn from(source: &Model) -> Self {
        match source.status {
            EventStatus::Success => Self::Success,
            EventStatus::Error => {
                // unwrap is safe here, assuming the data has been inserted using [Status]
                Self::Error(source.status_description.as_ref().unwrap().to_owned())
            }
            EventStatus::Cancelled => Self::Cancelled,
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
    pub status: Status,
}

impl WalletEvent {
    pub fn new(
        event_type: EventType,
        doc_type: DocType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        status: Status,
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
}

impl From<Model> for WalletEvent {
    fn from(source: Model) -> Self {
        Self {
            id: source.id,
            status: Status::from(&source),
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
