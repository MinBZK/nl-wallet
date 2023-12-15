use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use uuid::Uuid;

pub use entity::{history_doc_type, history_event};
use nl_wallet_mdoc::{basic_sa_ext::Entry, holder::Mdoc, utils::x509::Certificate};

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
                // unwrap is safe here, assuming the data has been inserted using [Status]
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

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Issuance(DocTypeMap),
    Disclosure(Option<DocTypeMap>),
}

impl EventType {
    pub fn doc_types(&self) -> IndexSet<&str> {
        match self {
            Self::Issuance(DocTypeMap(mdocs)) => mdocs.keys().map(String::as_str).collect(),
            Self::Disclosure(Some(DocTypeMap(proposal))) => proposal.keys().map(String::as_str).collect(),
            Self::Disclosure(None) => Default::default(),
        }
    }
}

impl From<EventType> for history_event::EventType {
    fn from(source: EventType) -> Self {
        match source {
            EventType::Issuance(_) => history_event::EventType::Issuance,
            EventType::Disclosure(_) => history_event::EventType::Disclosure,
        }
    }
}

impl From<(&history_event::Model, Vec<history_doc_type::Model>)> for EventType {
    fn from(source: (&history_event::Model, Vec<history_doc_type::Model>)) -> Self {
        let (event, doc_types) = source;
        match event.event_type {
            history_event::EventType::Issuance => {
                let map = doc_types
                    .into_iter()
                    .map(|doc_type| (doc_type.doc_type, Default::default()))
                    .collect();
                EventType::Issuance(DocTypeMap(map))
            } // TODO fix once attributes are stored
            history_event::EventType::Disclosure => {
                let map = doc_types
                    .into_iter()
                    .map(|doc_type| (doc_type.doc_type, Default::default()))
                    .collect();
                EventType::Disclosure(Some(DocTypeMap(map)))
            } // TODO fix once attributes are stored
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub remote_party_certificate: Certificate,
    pub status: EventStatus,
}

impl WalletEvent {
    pub fn new(
        id: Uuid,
        event_type: EventType,
        timestamp: DateTime<Utc>,
        remote_party_certificate: Certificate,
        status: EventStatus,
    ) -> Self {
        Self {
            id,
            event_type,
            timestamp,
            remote_party_certificate,
            status,
        }
    }

    pub fn now(event_type: EventType, remote_party_certificate: Certificate, status: EventStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            remote_party_certificate,
            status,
        }
    }

    pub fn success(event_type: EventType, remote_party_certificate: Certificate) -> Self {
        Self::now(event_type, remote_party_certificate, EventStatus::Success)
    }

    pub fn cancelled(event_type: EventType, remote_party_certificate: Certificate) -> Self {
        Self::now(event_type, remote_party_certificate, EventStatus::Cancelled)
    }

    pub fn error(event_type: EventType, remote_party_certificate: Certificate, reason: String) -> Self {
        Self::now(event_type, remote_party_certificate, EventStatus::Error(reason))
    }
}

impl From<(history_event::Model, Vec<history_doc_type::Model>)> for WalletEvent {
    fn from(source: (history_event::Model, Vec<history_doc_type::Model>)) -> Self {
        let (event, doc_types) = source;
        Self {
            id: event.id,
            status: EventStatus::from(&event),
            event_type: (&event, doc_types).into(),
            timestamp: event.timestamp,
            remote_party_certificate: event.remote_party_certificate.into(),
        }
    }
}

impl From<WalletEvent> for history_event::Model {
    fn from(source: WalletEvent) -> Self {
        Self {
            id: source.id,
            event_type: source.event_type.into(),
            timestamp: source.timestamp,
            remote_party_certificate: source.remote_party_certificate.into(),
            status_description: source.status.description().map(str::to_owned),
            status: source.status.into(),
        }
    }
}

#[cfg(feature = "mock")]
mod mock {
    use super::*;

    impl EventType {
        pub fn issuance_from_str(doc_types: Vec<&str>) -> Self {
            let mut map = IndexMap::new();
            for doc_type in doc_types {
                map.insert(doc_type.to_owned(), Default::default());
            }
            Self::Issuance(DocTypeMap(map))
        }

        pub fn disclosure_from_str(doc_types: Vec<&str>) -> Self {
            let mut map = IndexMap::new();
            for doc_type in doc_types {
                map.insert(doc_type.to_owned(), Default::default());
            }
            Self::Disclosure(Some(DocTypeMap(map)))
        }
    }
}
