use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use uuid::Uuid;

pub use entity::history_event;
use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    holder::Mdoc,
    utils::{
        serialization::{cbor_deserialize, cbor_serialize, CborError},
        x509::Certificate,
    },
};

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

impl From<EventStatus> for history_event::EventStatus {
    fn from(source: EventStatus) -> Self {
        match source {
            EventStatus::Success => Self::Success,
            EventStatus::Error(_) => Self::Error,
            EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&history_event::Model> for EventStatus {
    fn from(source: &history_event::Model) -> Self {
        match source.status {
            history_event::EventStatus::Success => Self::Success,
            history_event::EventStatus::Error => {
                // unwrap is safe here, assuming the data has been inserted using [EventStatus]
                Self::Error(source.status_description.as_ref().unwrap().to_owned())
            }
            history_event::EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

type NamespaceMap = IndexMap<String, Vec<Entry>>;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocTypeMap(pub IndexMap<String, NamespaceMap>);

impl From<Vec<Mdoc>> for DocTypeMap {
    fn from(source: Vec<Mdoc>) -> Self {
        let doc_type_map = source
            .into_iter()
            .map(|mdoc| {
                let namespace_map = mdoc.attributes(); // extracted to prevent borrow after move compilation error
                (mdoc.doc_type, namespace_map)
            })
            .collect();
        Self(doc_type_map)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        mdocs: DocTypeMap,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
    },
    Disclosure {
        id: Uuid,
        documents: Option<DocTypeMap>,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        status: EventStatus,
    },
}

impl WalletEvent {
    /// Returns the associated doc_types for this event. Will return an empty set if there are no attributes.
    pub fn associated_doc_types(&self) -> IndexSet<&str> {
        match self {
            Self::Issuance {
                mdocs: DocTypeMap(mdocs),
                ..
            }
            | Self::Disclosure {
                documents: Some(DocTypeMap(mdocs)),
                ..
            } => mdocs.keys().map(String::as_str).collect(),
            Self::Disclosure { documents: None, .. } => Default::default(),
        }
    }
}

impl TryFrom<history_event::Model> for WalletEvent {
    type Error = CborError;
    fn try_from(event: history_event::Model) -> Result<Self, Self::Error> {
        let result = match event.event_type {
            history_event::EventType::Issuance => Self::Issuance {
                id: event.id,
                mdocs: DocTypeMap(cbor_deserialize(event.attributes.unwrap().as_slice())?), // Unwrap is safe here
                timestamp: event.timestamp,
                remote_party_certificate: event.remote_party_certificate.into(),
            },
            history_event::EventType::Disclosure => Self::Disclosure {
                id: event.id,
                status: EventStatus::from(&event),
                documents: event
                    .attributes
                    .map(|attributes| Ok::<DocTypeMap, CborError>(DocTypeMap(cbor_deserialize(attributes.as_slice())?)))
                    .transpose()?,
                timestamp: event.timestamp,
                remote_party_certificate: event.remote_party_certificate.into(),
            },
        };
        Ok(result)
    }
}

impl TryFrom<WalletEvent> for history_event::Model {
    type Error = CborError;
    fn try_from(source: WalletEvent) -> Result<Self, Self::Error> {
        let result = match source {
            WalletEvent::Issuance {
                id,
                mdocs: DocTypeMap(mdocs),
                timestamp,
                remote_party_certificate,
            } => Self {
                attributes: Some(cbor_serialize(&mdocs)?),
                id,
                event_type: history_event::EventType::Issuance,
                timestamp,
                remote_party_certificate: remote_party_certificate.into(),
                status_description: None,
                status: history_event::EventStatus::Success,
            },
            WalletEvent::Disclosure {
                id,
                status,
                documents,
                timestamp,
                remote_party_certificate,
            } => Self {
                attributes: documents.map(|DocTypeMap(mdocs)| cbor_serialize(&mdocs)).transpose()?,
                id,
                event_type: history_event::EventType::Disclosure,
                timestamp,
                remote_party_certificate: remote_party_certificate.into(),
                status_description: status.description().map(ToString::to_string),
                status: status.into(),
            },
        };
        Ok(result)
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use crate::document::{
        create_full_unsigned_address_mdoc, create_full_unsigned_pid_mdoc, create_minimal_unsigned_address_mdoc,
        create_minimal_unsigned_pid_mdoc,
    };

    use super::*;

    impl WalletEvent {
        pub fn issuance_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            remote_party_certificate: Certificate,
        ) -> Self {
            let docs = vec![create_full_unsigned_pid_mdoc(), create_full_unsigned_address_mdoc()];
            let map = docs
                .into_iter()
                .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
                .map(|doc| (doc.doc_type, doc.attributes))
                .collect();
            Self::Issuance {
                id: Uuid::new_v4(),
                mdocs: DocTypeMap(map),
                timestamp,
                remote_party_certificate,
            }
        }

        pub fn disclosure_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            remote_party_certificate: Certificate,
        ) -> Self {
            let docs = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ];
            let map = docs
                .into_iter()
                .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
                .map(|doc| (doc.doc_type, doc.attributes))
                .collect();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: Some(DocTypeMap(map)),
                timestamp,
                remote_party_certificate,
                status: EventStatus::Success,
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, remote_party_certificate: Certificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                remote_party_certificate,
                status: EventStatus::Cancelled,
            }
        }

        pub fn disclosure_error(
            timestamp: DateTime<Utc>,
            remote_party_certificate: Certificate,
            error_message: String,
        ) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                remote_party_certificate,
                status: EventStatus::Error(error_message),
            }
        }

        pub fn timestamp(&self) -> &DateTime<Utc> {
            match self {
                Self::Issuance { timestamp, .. } => timestamp,
                Self::Disclosure { timestamp, .. } => timestamp,
            }
        }
    }
}
