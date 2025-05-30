use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use uuid::Uuid;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::auth::reader_auth::ReaderRegistration;
use crypto::x509::BorrowingCertificate;
use crypto::x509::BorrowingCertificateExtension;
use entity::disclosure_history_event::EventStatus;
use entity::disclosure_history_event::EventType;
use mdoc::holder::ProposedAttributes;
use utils::vec_at_least::VecNonEmpty;

use crate::attestation::Attestation;
use crate::issuance::BSN_ATTR_NAME;
use crate::issuance::PID_DOCTYPE;

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
        .and_then(|entry| (entry.name == BSN_ATTR_NAME).then_some(DisclosureType::Login))
        .unwrap_or(DisclosureType::Regular)
}

#[derive(Debug, Clone, Copy)]
pub enum DataDisclosureStatus {
    Disclosed,
    NotDisclosed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        attestations: VecNonEmpty<Attestation>,
        timestamp: DateTime<Utc>,
    },
    Disclosure {
        id: Uuid,
        attestations: Vec<Attestation>,
        timestamp: DateTime<Utc>,
        // TODO (PVW-4135): Only store reader registration in event.
        reader_certificate: Box<BorrowingCertificate>,
        reader_registration: Box<ReaderRegistration>,
        status: DisclosureStatus,
        r#type: DisclosureType,
    },
}

impl WalletEvent {
    pub(crate) fn new_issuance(attestations: VecNonEmpty<Attestation>) -> Self {
        Self::Issuance {
            id: Uuid::now_v7(),
            attestations,
            timestamp: Utc::now(),
        }
    }

    fn new_disclosure(
        proposed_attributes: Option<ProposedAttributes>,
        reader_certificate: BorrowingCertificate,
        reader_registration: ReaderRegistration,
        status: DisclosureStatus,
        data_status: DataDisclosureStatus,
    ) -> Self {
        // If no attributes are available, do not record that this disclosure was for the purposes of logging in.
        let r#type = proposed_attributes
            .as_ref()
            .map(disclosure_type_for_proposed_attributes)
            .unwrap_or(DisclosureType::Regular);

        let attestations = match data_status {
            DataDisclosureStatus::Disclosed => proposed_attributes,
            DataDisclosureStatus::NotDisclosed => None,
        }
        .unwrap_or_default()
        .into_values()
        .map(|document_attributes| {
            // As the proposed attributes come from the database, we can make assumptions about them and use `expect()`.
            // TODO (PVW-4132): Use the type system to codify these assumptions.
            let reader_registration = IssuerRegistration::from_certificate(&document_attributes.issuer)
                .expect("proposed attributes should contain valid issuer registration")
                .expect("proposed attributes should contain issuer registration");

            Attestation::create_for_disclosure(
                document_attributes.type_metadata,
                reader_registration.organization,
                document_attributes.attributes,
            )
            .expect("proposed attributes should succesfully be transformed for display by metadata")
        })
        .collect();

        Self::Disclosure {
            id: Uuid::now_v7(),
            attestations,
            timestamp: Utc::now(),
            reader_certificate: Box::new(reader_certificate),
            reader_registration: Box::new(reader_registration),
            status,
            r#type,
        }
    }

    pub(crate) fn new_disclosure_success(
        proposed_attributes: ProposedAttributes,
        reader_certificate: BorrowingCertificate,
        reader_registration: ReaderRegistration,
        data_status: DataDisclosureStatus,
    ) -> Self {
        Self::new_disclosure(
            Some(proposed_attributes),
            reader_certificate,
            reader_registration,
            EventStatus::Success,
            data_status,
        )
    }

    pub(crate) fn new_disclosure_error(
        proposed_attributes: ProposedAttributes,
        reader_certificate: BorrowingCertificate,
        reader_registration: ReaderRegistration,
        data_status: DataDisclosureStatus,
    ) -> Self {
        Self::new_disclosure(
            Some(proposed_attributes),
            reader_certificate,
            reader_registration,
            EventStatus::Error,
            data_status,
        )
    }

    pub(crate) fn new_disclosure_cancel(
        proposed_attributes: Option<ProposedAttributes>,
        reader_certificate: BorrowingCertificate,
        reader_registration: ReaderRegistration,
        data_status: DataDisclosureStatus,
    ) -> Self {
        Self::new_disclosure(
            proposed_attributes,
            reader_certificate,
            reader_registration,
            EventStatus::Cancelled,
            data_status,
        )
    }

    /// Returns the associated doc_types for this event. Will return an empty set if there are no attributes.
    pub(super) fn associated_attestation_types(&self) -> HashSet<&str> {
        match self {
            Self::Issuance { attestations, .. } => attestations.as_slice(),
            Self::Disclosure { attestations, .. } => attestations,
        }
        .iter()
        .map(|attestation| attestation.attestation_type.as_str())
        .collect()
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Self::Issuance { timestamp, .. } => timestamp,
            Self::Disclosure { timestamp, .. } => timestamp,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use attestation_data::identifiers::NameSpace;
    use indexmap::IndexMap;
    use rstest::rstest;

    use attestation_data::attributes::Entry;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::BorrowingCertificate;
    use mdoc::holder::ProposedDocumentAttributes;
    use mdoc::DataElementValue;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attestation::AttestationIdentity;
    use crate::issuance;

    use super::*;

    static ISSUER_CERTIFICATE: LazyLock<BorrowingCertificate> = LazyLock::new(|| {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key_pair = generate_issuer_mock(&ca, IssuerRegistration::new_mock().into()).unwrap();

        issuer_key_pair.certificate().clone()
    });

    #[rstest]
    #[case(issuance::mock::create_bsn_only_mdoc_attributes(), DisclosureType::Login)]
    #[case(issuance::mock::create_example_mdoc_attributes(), DisclosureType::Regular)]
    fn test_disclosure_type_from_proposed_attributes(
        #[case] (proposed_attributes, type_metadata): (IndexMap<NameSpace, Vec<Entry>>, TypeMetadata),
        #[case] expected: DisclosureType,
    ) {
        let proposed_attributes = ProposedAttributes::from([(
            PID_DOCTYPE.to_string(),
            ProposedDocumentAttributes {
                attributes: proposed_attributes,
                issuer: ISSUER_CERTIFICATE.clone(),
                type_metadata: NormalizedTypeMetadata::from_single_example(type_metadata.into_inner()),
            },
        )]);

        assert_eq!(disclosure_type_for_proposed_attributes(&proposed_attributes), expected);
    }

    fn mock_attestations_for_attestation_types(
        attestation_types: &[&str],
        issuer_certificate: &BorrowingCertificate,
    ) -> Vec<Attestation> {
        let issuer_registration = IssuerRegistration::from_certificate(issuer_certificate)
            .unwrap()
            .unwrap();

        attestation_types
            .iter()
            .zip(itertools::repeat_n(
                issuer_registration.organization,
                attestation_types.len(),
            ))
            .map(|(attestation_type, issuer_org)| {
                let metadata =
                    NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
                        attestation_type,
                        BSN_ATTR_NAME,
                        JsonSchemaPropertyType::String,
                        None,
                    ));
                let attributes = IndexMap::from([(
                    attestation_type.to_string(),
                    vec![Entry {
                        name: BSN_ATTR_NAME.to_string(),
                        value: DataElementValue::Text("999999999".to_string()),
                    }],
                )]);

                Attestation::create_for_issuance(AttestationIdentity::Ephemeral, metadata, issuer_org, attributes)
                    .unwrap()
            })
            .collect()
    }

    impl WalletEvent {
        pub fn issuance_from_str(
            attestation_types: &[&str],
            timestamp: DateTime<Utc>,
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            Self::Issuance {
                id: Uuid::now_v7(),
                attestations: mock_attestations_for_attestation_types(attestation_types, issuer_certificate)
                    .try_into()
                    .unwrap(),
                timestamp,
            }
        }

        pub fn disclosure_from_str(
            attestation_types: &[&str],
            timestamp: DateTime<Utc>,
            reader_certificate: BorrowingCertificate,
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
                .unwrap()
                .unwrap();

            Self::Disclosure {
                id: Uuid::now_v7(),
                attestations: mock_attestations_for_attestation_types(attestation_types, issuer_certificate),
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                reader_registration: Box::new(reader_registration),
                status: DisclosureStatus::Success,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error_from_str(
            attestation_types: &[&str],
            timestamp: DateTime<Utc>,
            reader_certificate: BorrowingCertificate,
            issuer_certificate: &BorrowingCertificate,
        ) -> Self {
            let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
                .unwrap()
                .unwrap();

            Self::Disclosure {
                id: Uuid::now_v7(),
                attestations: mock_attestations_for_attestation_types(attestation_types, issuer_certificate),
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                reader_registration: Box::new(reader_registration),
                status: DisclosureStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_cancel(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
                .unwrap()
                .unwrap();

            Self::Disclosure {
                id: Uuid::now_v7(),
                attestations: Vec::new(),
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                reader_registration: Box::new(reader_registration),
                status: DisclosureStatus::Cancelled,
                r#type: DisclosureType::Regular,
            }
        }

        pub fn disclosure_error(timestamp: DateTime<Utc>, reader_certificate: BorrowingCertificate) -> Self {
            let reader_registration = ReaderRegistration::from_certificate(&reader_certificate)
                .unwrap()
                .unwrap();

            Self::Disclosure {
                id: Uuid::now_v7(),
                attestations: Vec::new(),
                timestamp,
                reader_certificate: Box::new(reader_certificate),
                reader_registration: Box::new(reader_registration),
                status: DisclosureStatus::Error,
                r#type: DisclosureType::Regular,
            }
        }
    }
}
