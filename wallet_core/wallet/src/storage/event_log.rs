use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use uuid::Uuid;

use nl_wallet_mdoc::holder::Mdoc;
use nl_wallet_mdoc::holder::ProposedAttributes;
use nl_wallet_mdoc::holder::ProposedDocumentAttributes;
use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::utils::cose::CoseError;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use nl_wallet_mdoc::DataElementIdentifier;
use nl_wallet_mdoc::DataElementValue;
use nl_wallet_mdoc::DocType;
use nl_wallet_mdoc::NameSpace;

use entity::disclosure_history_event;
use entity::issuance_history_event;

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
        reader_certificate: Box<BorrowingCertificate>,
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
        reader_certificate: BorrowingCertificate,
        status: EventStatus,
        r#type: DisclosureType,
    ) -> Self {
        Self::Disclosure {
            id: Uuid::new_v4(),
            documents,
            timestamp: Utc::now(),
            reader_certificate: Box::new(reader_certificate),
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
            reader_certificate: Box::new(BorrowingCertificate::from_der(event.relying_party_certificate).unwrap()), /* Unwrapping here is safe since the certificate has been parsed before */
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventAttributes {
    #[serde_as(as = "Base64")]
    pub issuer: BorrowingCertificate,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
}

impl From<(BorrowingCertificate, IndexMap<NameSpace, Vec<Entry>>)> for EventAttributes {
    fn from((issuer, attributes): (BorrowingCertificate, IndexMap<NameSpace, Vec<Entry>>)) -> Self {
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
                let issuer = mdoc.issuer_certificate()?;
                let doc_type = mdoc.mso.doc_type;
                let attributes = mdoc.issuer_signed.into_entries_by_namespace();
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
    use nl_wallet_mdoc::utils::x509::BorrowingCertificate;

    use crate::document::create_full_unsigned_address_mdoc;
    use crate::document::create_full_unsigned_pid_mdoc;
    use crate::document::create_minimal_unsigned_address_mdoc;
    use crate::document::create_minimal_unsigned_pid_mdoc;

    use super::*;

    fn from_unsigned_mdocs_filtered(
        docs: Vec<UnsignedMdoc>,
        doc_types: &[&str],
        issuer_certificate: &BorrowingCertificate,
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
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            let (docs, _): (Vec<_>, Vec<_>) =
                vec![create_full_unsigned_pid_mdoc(), create_full_unsigned_address_mdoc()]
                    .into_iter()
                    .unzip();
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
            reader_certificate: BorrowingCertificate,
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            let (docs, _): (Vec<_>, Vec<_>) = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ]
            .into_iter()
            .unzip();
            let documents = from_unsigned_mdocs_filtered(docs, doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: EventStatus::Success,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error_from_str(
            doc_types: &[&str],
            timestamp: DateTime<Utc>,
            reader_certificate: BorrowingCertificate,
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            let (docs, _): (Vec<_>, Vec<_>) = vec![
                create_minimal_unsigned_pid_mdoc(),
                create_minimal_unsigned_address_mdoc(),
            ]
            .into_iter()
            .unzip();
            let documents = from_unsigned_mdocs_filtered(docs, doc_types, issuer_certificate).into();
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: EventStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: EventStatus::Cancelled,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: EventStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }
    }
}
