use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use uuid::Uuid;

pub use entity::history_event;
use nl_wallet_mdoc::{
    holder::{Mdoc, ProposedCard},
    utils::{
        cose::CoseError,
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocTypeMap(pub IndexMap<String, ProposedCard>);

impl TryFrom<Vec<Mdoc>> for DocTypeMap {
    type Error = CoseError;
    fn try_from(source: Vec<Mdoc>) -> Result<Self, Self::Error> {
        let doc_type_map = source
            .into_iter()
            .map(|mdoc| {
                let issuer = mdoc.issuer_certificate()?;
                let attributes = mdoc.attributes(); // extracted to prevent borrow after move compilation error
                Ok((mdoc.doc_type, ProposedCard { issuer, attributes }))
            })
            .collect::<Result<IndexMap<_, _>, CoseError>>()?;
        Ok(Self(doc_type_map))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        mdocs: DocTypeMap,
        timestamp: DateTime<Utc>,
        issuer_certificate: Certificate,
    },
    Disclosure {
        id: Uuid,
        documents: Option<DocTypeMap>,
        timestamp: DateTime<Utc>,
        reader_certificate: Certificate,
        status: EventStatus,
    },
}

impl WalletEvent {
    pub fn new_issuance(mdocs: DocTypeMap, issuer_certificate: Certificate) -> Self {
        Self::Issuance {
            id: Uuid::new_v4(),
            mdocs,
            timestamp: Utc::now(),
            issuer_certificate,
        }
    }

    pub fn new_disclosure(documents: Option<DocTypeMap>, reader_certificate: Certificate, status: EventStatus) -> Self {
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
                mdocs: DocTypeMap(cbor_deserialize(event.attributes.unwrap().as_slice())?), // Unwrap is safe here
                timestamp: event.timestamp,
                issuer_certificate: event.remote_party_certificate.into(),
            },
            history_event::EventType::Disclosure => Self::Disclosure {
                id: event.id,
                status: EventStatus::from(&event),
                documents: event
                    .attributes
                    .map(|attributes| Ok::<DocTypeMap, CborError>(DocTypeMap(cbor_deserialize(attributes.as_slice())?)))
                    .transpose()?,
                timestamp: event.timestamp,
                reader_certificate: event.remote_party_certificate.into(),
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
                issuer_certificate,
            } => Self {
                attributes: Some(cbor_serialize(&mdocs)?),
                id,
                event_type: history_event::EventType::Issuance,
                timestamp,
                remote_party_certificate: issuer_certificate.into(),
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
                attributes: documents.map(|DocTypeMap(mdocs)| cbor_serialize(&mdocs)).transpose()?,
                id,
                event_type: history_event::EventType::Disclosure,
                timestamp,
                remote_party_certificate: reader_certificate.into(),
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

    impl DocTypeMap {
        fn from_unsigned_mdocs_filtered(
            docs: Vec<UnsignedMdoc>,
            doc_types: &[&str],
            issuer_certificate: &Certificate,
        ) -> Self {
            let map = docs
                .into_iter()
                .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
                .map(|doc| {
                    (
                        doc.doc_type,
                        ProposedCard {
                            issuer: issuer_certificate.clone(),
                            attributes: doc.attributes,
                        },
                    )
                })
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
            let mdocs = DocTypeMap::from_unsigned_mdocs_filtered(docs, &doc_types, &issuer_certificate);
            Self::Issuance {
                id: Uuid::new_v4(),
                mdocs,
                timestamp,
                issuer_certificate,
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
            let documents = DocTypeMap::from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
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
            let documents = DocTypeMap::from_unsigned_mdocs_filtered(docs, &doc_types, issuer_certificate).into();
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
