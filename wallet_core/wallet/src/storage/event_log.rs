use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use uuid::Uuid;

use entity::disclosure_history_event::EventStatus;
use entity::disclosure_history_event::EventType;
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

const PID_DOCTYPE: &str = "com.example.pid";

pub type DisclosureStatus = EventStatus;
pub type DisclosureType = EventType;

/// Something is a login flow if the `proposed_attributes` map has exactly one element with a
/// `doc_type` of `PID_DOCTYPE`, with a `doc_attributes` map where `namespace` is `PID_DOCTYPE`
/// also, with an entry vec of exactly one entry, where the `DataElementIdentifier` string is "bsn".
pub fn disclosure_type_for_proposed_attributes(proposed_attributes: &ProposedAttributes) -> DisclosureType {
    proposed_attributes
        .iter()
        .exactly_one()
        .ok()
        .and_then(|(doc_type, doc_attributes)| (doc_type == PID_DOCTYPE).then_some(doc_attributes))
        .and_then(|doc_attributes| doc_attributes.attributes.iter().exactly_one().ok())
        .and_then(|(namespace, entries)| (namespace == PID_DOCTYPE).then_some(entries))
        .and_then(|entries| entries.iter().exactly_one().ok())
        .and_then(|entry| (entry.name == "bsn").then_some(DisclosureType::Login))
        .unwrap_or(DisclosureType::Regular)
}

#[derive(Debug, Clone, PartialEq)]
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
        status: DisclosureStatus,
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
        status: DisclosureStatus,
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
    pub(super) fn associated_doc_types(&self) -> HashSet<&str> {
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
    use std::sync::LazyLock;

    use rstest::rstest;

    use nl_wallet_mdoc::server_keys::generate::Ca;
    use nl_wallet_mdoc::unsigned::UnsignedMdoc;
    use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
    use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
    use sd_jwt::metadata::TypeMetadata;

    use crate::document::create_bsn_only_unsigned_pid_mdoc;
    use crate::document::create_full_unsigned_address_mdoc;
    use crate::document::create_full_unsigned_pid_mdoc;
    use crate::document::create_minimal_unsigned_address_mdoc;
    use crate::document::create_minimal_unsigned_pid_mdoc;

    use super::*;

    static ISSUER_CERTIFICATE: LazyLock<BorrowingCertificate> = LazyLock::new(|| {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key_pair = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

        issuer_key_pair.certificate().clone()
    });

    #[rstest]
    #[case(create_bsn_only_unsigned_pid_mdoc(), DisclosureType::Login)]
    #[case(create_minimal_unsigned_pid_mdoc(), DisclosureType::Regular)]
    #[case(create_full_unsigned_pid_mdoc(), DisclosureType::Regular)]
    fn test_disclosure_type_from_proposed_attributes(
        #[case] (unsigned_mdoc, type_metadata): (UnsignedMdoc, TypeMetadata),
        #[case] expected: DisclosureType,
    ) {
        let proposed_attributes = ProposedAttributes::from([(
            PID_DOCTYPE.to_string(),
            ProposedDocumentAttributes {
                attributes: unsigned_mdoc.attributes.into_inner(),
                issuer: ISSUER_CERTIFICATE.clone(),
                type_metadata,
            },
        )]);

        assert_eq!(disclosure_type_for_proposed_attributes(&proposed_attributes), expected);
    }

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
                status: DisclosureStatus::Success,
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
                status: DisclosureStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: DisclosureStatus::Cancelled,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            Self::Disclosure {
                id: Uuid::new_v4(),
                documents: None,
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                status: DisclosureStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }
    }
}
