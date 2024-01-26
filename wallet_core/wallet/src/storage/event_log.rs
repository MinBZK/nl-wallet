use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use entity::history_event;
use nl_wallet_mdoc::{
    basic_sa_ext::Entry,
    holder::{Mdoc, ProposedAttributes, ProposedDocumentAttributes},
    utils::{
        cose::CoseError,
        serialization::{cbor_deserialize, cbor_serialize, CborError},
        x509::Certificate,
    },
    DataElementIdentifier, DataElementValue, DocType, NameSpace,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventAttributes {
    pub issuer: Certificate,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventDocuments(pub IndexMap<DocType, EventAttributes>);

impl From<(Certificate, IndexMap<NameSpace, Vec<Entry>>)> for EventAttributes {
    fn from((issuer, attributes): (Certificate, IndexMap<NameSpace, Vec<Entry>>)) -> EventAttributes {
        EventAttributes {
            issuer,
            attributes: attributes
                .into_iter()
                .map(|(namespace, attributes)| {
                    (
                        namespace,
                        attributes.into_iter().map(|entry| (entry.name, entry.value)).collect(),
                    )
                })
                .collect(),
        }
    }
}

impl From<EventAttributes> for IndexMap<NameSpace, Vec<Entry>> {
    fn from(source: EventAttributes) -> IndexMap<NameSpace, Vec<Entry>> {
        source
            .attributes
            .into_iter()
            .map(|(namespace, attributes)| {
                (
                    namespace,
                    attributes
                        .into_iter()
                        .map(|(name, value)| Entry { name, value })
                        .collect(),
                )
            })
            .collect()
    }
}

impl TryFrom<Vec<Mdoc>> for EventDocuments {
    type Error = CoseError;
    fn try_from(source: Vec<Mdoc>) -> Result<Self, Self::Error> {
        let doc_type_map = source
            .into_iter()
            .map(|mdoc| {
                let doc_type = mdoc.doc_type.clone();
                let issuer = mdoc.issuer_certificate()?;
                let attributes = mdoc.attributes();
                Ok((doc_type, (issuer, attributes).into()))
            })
            .collect::<Result<IndexMap<_, _>, CoseError>>()?;
        Ok(Self(doc_type_map))
    }
}

impl From<ProposedDocumentAttributes> for EventAttributes {
    fn from(source: ProposedDocumentAttributes) -> Self {
        (source.issuer, source.attributes).into()
    }
}

impl From<ProposedAttributes> for EventDocuments {
    fn from(source: ProposedAttributes) -> Self {
        let documents = source
            .into_iter()
            .map(|(doc_type, document)| (doc_type, document.into()))
            .collect();
        Self(documents)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        mdocs: EventDocuments,
        timestamp: DateTime<Utc>,
    },
    Disclosure {
        id: Uuid,
        documents: Option<EventDocuments>,
        timestamp: DateTime<Utc>,
        reader_certificate: Certificate,
        status: EventStatus,
    },
}

impl WalletEvent {
    pub fn new_issuance(mdocs: EventDocuments) -> Self {
        Self::Issuance {
            id: Uuid::new_v4(),
            mdocs,
            timestamp: Utc::now(),
        }
    }

    pub fn new_disclosure(
        documents: Option<EventDocuments>,
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
                mdocs: EventDocuments(mdocs),
                ..
            }
            | Self::Disclosure {
                documents: Some(EventDocuments(mdocs)),
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

impl TryFrom<history_event::Model> for WalletEvent {
    type Error = CborError;
    fn try_from(event: history_event::Model) -> Result<Self, Self::Error> {
        let result = match event.event_type {
            history_event::EventType::Issuance => Self::Issuance {
                id: event.id,
                mdocs: EventDocuments(cbor_deserialize(event.attributes.unwrap().as_slice())?), // Unwrap is safe here
                timestamp: event.timestamp,
            },
            history_event::EventType::Disclosure => Self::Disclosure {
                id: event.id,
                status: EventStatus::from(&event),
                documents: event
                    .attributes
                    .map(|attributes| {
                        Ok::<EventDocuments, CborError>(EventDocuments(cbor_deserialize(attributes.as_slice())?))
                    })
                    .transpose()?,
                timestamp: event.timestamp,
                reader_certificate: event.relying_party_certificate.map(Into::into).unwrap(),
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
                mdocs: EventDocuments(mdocs),
                timestamp,
            } => Self {
                attributes: Some(cbor_serialize(&mdocs)?),
                id,
                event_type: history_event::EventType::Issuance,
                timestamp,
                relying_party_certificate: None,
                status_description: None,
                status: history_event::EventStatus::Success,
            },
            WalletEvent::Disclosure {
                id,
                status,
                documents,
                timestamp,
                reader_certificate,
            } => Self {
                attributes: documents
                    .map(|EventDocuments(mdocs)| cbor_serialize(&mdocs))
                    .transpose()?,
                id,
                event_type: history_event::EventType::Disclosure,
                timestamp,
                relying_party_certificate: Some(reader_certificate.into()),
                status_description: status.description().map(ToString::to_string),
                status: status.into(),
            },
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

    impl EventDocuments {
        fn from_unsigned_mdocs_filtered(
            docs: Vec<UnsignedMdoc>,
            doc_types: &[&str],
            issuer_certificate: &Certificate,
        ) -> Self {
            let map = docs
                .into_iter()
                .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
                .map(|doc| (doc.doc_type, (issuer_certificate.clone(), doc.attributes).into()))
                .collect();
            Self(map)
        }
    }

    impl WalletEvent {
        pub fn issuance_from_str(
            doc_types: Vec<&str>,
            timestamp: DateTime<Utc>,
            issuer_certificate: Certificate,
        ) -> Self {
            let docs = vec![create_full_unsigned_pid_mdoc(), create_full_unsigned_address_mdoc()];
            let mdocs = EventDocuments::from_unsigned_mdocs_filtered(docs, &doc_types, &issuer_certificate);
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
            let documents = EventDocuments::from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
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
            let documents = EventDocuments::from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
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
