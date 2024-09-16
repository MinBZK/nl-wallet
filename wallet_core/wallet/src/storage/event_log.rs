use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use entity::{disclosure_history_event, issuance_history_event};
use nl_wallet_mdoc::{
    holder::{Mdoc, ProposedAttributes, ProposedDocumentAttributes},
    unsigned::Entry,
    utils::{cose::CoseError, x509::Certificate},
    DataElementIdentifier, DataElementValue, DocType, NameSpace,
};

use crate::document::DisclosureType;

// TODO: Think about refactoring/renaming EventStatus.
// For rationale, see comment for DisclosureType in mdoc.rs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventStatus {
    Success,
    Error,
    Cancelled,
}

impl From<EventStatus> for disclosure_history_event::EventStatus {
    fn from(source: EventStatus) -> Self {
        match source {
            EventStatus::Success => Self::Success,
            EventStatus::Error => Self::Error,
            EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<&disclosure_history_event::Model> for EventStatus {
    fn from(source: &disclosure_history_event::Model) -> Self {
        match source.status {
            disclosure_history_event::EventStatus::Success => Self::Success,
            disclosure_history_event::EventStatus::Error => Self::Error,
            disclosure_history_event::EventStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<DisclosureType> for disclosure_history_event::EventType {
    fn from(source: DisclosureType) -> Self {
        match source {
            DisclosureType::Login => Self::Login,
            DisclosureType::Regular => Self::Regular,
        }
    }
}

impl From<&disclosure_history_event::Model> for DisclosureType {
    fn from(source: &disclosure_history_event::Model) -> Self {
        match source.r#type {
            disclosure_history_event::EventType::Login => Self::Login,
            disclosure_history_event::EventType::Regular => Self::Regular,
        }
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
        r#type: DisclosureType,
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
        r#type: DisclosureType,
    ) -> Self {
        Self::Disclosure {
            id: Uuid::new_v4(),
            documents,
            timestamp: Utc::now(),
            reader_certificate,
            status,
            r#type,
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

impl TryFrom<disclosure_history_event::Model> for WalletEvent {
    type Error = serde_json::Error;
    fn try_from(event: disclosure_history_event::Model) -> Result<Self, Self::Error> {
        let result = Self::Disclosure {
            id: event.id,
            status: EventStatus::from(&event),
            r#type: DisclosureType::from(&event),
            documents: event.attributes.map(serde_json::from_value).transpose()?,
            timestamp: event.timestamp,
            reader_certificate: event.relying_party_certificate.into(),
        };
        Ok(result)
    }
}

impl TryFrom<issuance_history_event::Model> for WalletEvent {
    type Error = serde_json::Error;
    fn try_from(event: issuance_history_event::Model) -> Result<Self, Self::Error> {
        let result = Self::Issuance {
            id: event.id,
            mdocs: serde_json::from_value(event.attributes)?,
            timestamp: event.timestamp,
        };
        Ok(result)
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
                attributes: serde_json::to_value(mdocs)?,
                id,
                timestamp,
            }),
            WalletEvent::Disclosure {
                id,
                status,
                documents,
                timestamp,
                reader_certificate,
                r#type,
            } => Self::Disclosure(disclosure_history_event::Model {
                attributes: documents.map(serde_json::to_value).transpose()?,
                id,
                timestamp,
                relying_party_certificate: reader_certificate.into(),
                status: status.into(),
                r#type: r#type.into(),
            }),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventAttributes {
    pub issuer: Certificate,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
}

impl From<(Certificate, IndexMap<NameSpace, Vec<Entry>>)> for EventAttributes {
    fn from((issuer, attributes): (Certificate, IndexMap<NameSpace, Vec<Entry>>)) -> Self {
        Self {
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

impl From<ProposedDocumentAttributes> for EventAttributes {
    fn from(source: ProposedDocumentAttributes) -> Self {
        (source.issuer, source.attributes).into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EventDocuments(pub IndexMap<DocType, EventAttributes>);

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

impl From<ProposedAttributes> for EventDocuments {
    fn from(source: ProposedAttributes) -> Self {
        let documents = source
            .into_iter()
            .map(|(doc_type, document)| (doc_type, document.into()))
            .collect();
        Self(documents)
    }
}

#[cfg(test)]
mod test {
    use nl_wallet_mdoc::unsigned::UnsignedMdoc;

    use crate::document::{
        create_full_unsigned_address_mdoc, create_full_unsigned_pid_mdoc, create_minimal_unsigned_address_mdoc,
        create_minimal_unsigned_pid_mdoc,
    };

    use super::*;

    fn from_unsigned_mdocs_filtered(
        docs: Vec<UnsignedMdoc>,
        doc_types: &[&str],
        issuer_certificate: &Certificate,
    ) -> EventDocuments {
        let map = docs
            .into_iter()
            .filter(|doc| doc_types.contains(&doc.doc_type.as_str()))
            .map(|doc| {
                (
                    doc.doc_type,
                    (issuer_certificate.clone(), doc.attributes.into_inner()).into(),
                )
            })
            .collect();
        EventDocuments(map)
    }

    impl WalletEvent {
        pub fn issuance_from_str(
            doc_types: &[&str],
            timestamp: DateTime<Utc>,
            issuer_certificate: &Certificate,
        ) -> Self {
            let docs = vec![create_full_unsigned_pid_mdoc(), create_full_unsigned_address_mdoc()];
            let mdocs = from_unsigned_mdocs_filtered(docs, doc_types, issuer_certificate);
            Self::Issuance {
                id: Uuid::new_v4(),
                mdocs,
                timestamp,
            }
        }

        pub fn disclosure_from_str(
            doc_types: &[&str],
            timestamp: DateTime<Utc>,
            reader_certificate: Certificate,
            issuer_certificate: &Certificate,
        ) -> Self {
            let docs = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ];
            let documents = from_unsigned_mdocs_filtered(docs, doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate,
                status: EventStatus::Success,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error_from_str(
            doc_types: &[&str],
            timestamp: DateTime<Utc>,
            reader_certificate: Certificate,
            issuer_certificate: &Certificate,
        ) -> Self {
            let docs = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ];
            let documents = from_unsigned_mdocs_filtered(docs, doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate,
                status: EventStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, reader_certificate: Certificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate,
                status: EventStatus::Cancelled,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error(timestamp: DateTime<Utc>, reader_certificate: Certificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate,
                status: EventStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }
    }
}
