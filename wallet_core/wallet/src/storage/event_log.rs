use chrono::{DateTime, Utc};
use indexmap::IndexSet;
use uuid::Uuid;

use entity::history_event_documents::HistoryEventDocuments;
pub use entity::{disclosure_history_event, issuance_history_event};
use nl_wallet_mdoc::utils::x509::Certificate;

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

impl From<EventStatus> for disclosure_history_event::EventStatus {
    fn from(source: EventStatus) -> Self {
        match source {
            EventStatus::Success => Self::Success,
            EventStatus::Error(_) => Self::Error,
            EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&disclosure_history_event::Model> for EventStatus {
    fn from(source: &disclosure_history_event::Model) -> Self {
        match source.status {
            disclosure_history_event::EventStatus::Success => Self::Success,
            disclosure_history_event::EventStatus::Error => {
                // unwrap is safe here, assuming the data has been inserted using [EventStatus]
                Self::Error(source.status_description.as_ref().unwrap().to_owned())
            }
            disclosure_history_event::EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        mdocs: HistoryEventDocuments,
        timestamp: DateTime<Utc>,
    },
    Disclosure {
        id: Uuid,
        documents: Option<HistoryEventDocuments>,
        timestamp: DateTime<Utc>,
        reader_certificate: Certificate,
        status: EventStatus,
    },
}

impl WalletEvent {
    pub fn new_issuance(mdocs: HistoryEventDocuments) -> Self {
        Self::Issuance {
            id: Uuid::new_v4(),
            mdocs,
            timestamp: Utc::now(),
        }
    }

    pub fn new_disclosure(
        documents: Option<HistoryEventDocuments>,
        reader_certificate: Certificate,
        status: EventStatus,
    ) -> Self {
        Self::Disclosure {
            id: Uuid::new_v4(),
            documents,
            timestamp: Utc::now(),
            reader_certificate,
            status,
        }
    }

    /// Returns the associated doc_types for this event. Will return an empty set if there are no attributes.
    pub fn associated_doc_types(&self) -> IndexSet<&str> {
        match self {
            Self::Issuance {
                mdocs: HistoryEventDocuments(mdocs),
                ..
            }
            | Self::Disclosure {
                documents: Some(HistoryEventDocuments(mdocs)),
                ..
            } => mdocs.keys().map(String::as_str).collect(),
            Self::Disclosure { documents: None, .. } => Default::default(),
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::Issuance { timestamp, .. } => timestamp,
            Self::Disclosure { timestamp, .. } => timestamp,
        }
    }
}

impl From<disclosure_history_event::Model> for WalletEvent {
    fn from(event: disclosure_history_event::Model) -> Self {
        Self::Disclosure {
            id: event.id,
            status: EventStatus::from(&event),
            documents: event.attributes,
            timestamp: event.timestamp,
            reader_certificate: event.relying_party_certificate.into(),
        }
    }
}

impl From<issuance_history_event::Model> for WalletEvent {
    fn from(event: issuance_history_event::Model) -> Self {
        Self::Issuance {
            id: event.id,
            mdocs: event.attributes,
            timestamp: event.timestamp,
        }
    }
}

/// Enumerates the different database models for a [`WalletEvent`].
pub(crate) enum WalletEventModel {
    Issuance(issuance_history_event::Model),
    Disclosure(disclosure_history_event::Model),
}

impl TryFrom<WalletEvent> for WalletEventModel {
    type Error = serde_json::Error;
    fn try_from(source: WalletEvent) -> Result<Self, Self::Error> {
        let result = match source {
            WalletEvent::Issuance { id, mdocs, timestamp } => Self::Issuance(issuance_history_event::Model {
                attributes: mdocs,
                id,
                timestamp,
            }),
            WalletEvent::Disclosure {
                id,
                status,
                documents,
                timestamp,
                reader_certificate,
            } => Self::Disclosure(disclosure_history_event::Model {
                attributes: documents,
                id,
                timestamp,
                relying_party_certificate: reader_certificate.into(),
                status_description: status.description().map(ToString::to_string),
                status: status.into(),
            }),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use nl_wallet_mdoc::basic_sa_ext::UnsignedMdoc;

    use crate::document::{
        create_full_unsigned_address_mdoc, create_full_unsigned_pid_mdoc, create_minimal_unsigned_address_mdoc,
        create_minimal_unsigned_pid_mdoc,
    };

    use super::*;

    fn from_unsigned_mdocs_filtered(
        docs: Vec<UnsignedMdoc>,
        doc_types: &[&str],
        issuer_certificate: &Certificate,
    ) -> HistoryEventDocuments {
        let map = docs
            .into_iter()
            .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
            .map(|doc| (doc.doc_type, (issuer_certificate.clone(), doc.attributes).into()))
            .collect();
        HistoryEventDocuments(map)
    }

    impl WalletEvent {
        pub fn issuance_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            issuer_certificate: Certificate,
        ) -> Self {
            let docs = vec![create_full_unsigned_pid_mdoc(), create_full_unsigned_address_mdoc()];
            let mdocs = from_unsigned_mdocs_filtered(docs, &doc_types, &issuer_certificate);
            Self::Issuance {
                id: Uuid::new_v4(),
                mdocs,
                timestamp,
            }
        }

        pub fn disclosure_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            reader_certificate: Certificate,
            issuer_certificate: &Certificate,
        ) -> Self {
            let docs = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ];
            let documents = from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate,
                status: EventStatus::Success,
            }
        }

        pub fn disclosure_error_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            reader_certificate: Certificate,
            issuer_certificate: &Certificate,
            error_message: String,
        ) -> Self {
            let docs = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ];
            let documents = from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate,
                status: EventStatus::Error(error_message),
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, reader_certificate: Certificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate,
                status: EventStatus::Cancelled,
            }
        }

        pub fn disclosure_error(
            timestamp: DateTime<Utc>,
            reader_certificate: Certificate,
            error_message: String,
        ) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate,
                status: EventStatus::Error(error_message),
            }
        }
    }
}
