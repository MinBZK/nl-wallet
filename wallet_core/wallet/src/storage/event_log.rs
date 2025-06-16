use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;
use entity::disclosure_event::EventType;
use mdoc::holder::ProposedAttributes;

use crate::attestation::AttestationPresentation;
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
        attestation: Box<AttestationPresentation>,
        timestamp: DateTime<Utc>,
        renewed: bool,
    },
    Disclosure {
        id: Uuid,
        attestations: Box<Vec<AttestationPresentation>>,
        timestamp: DateTime<Utc>,
        // TODO (PVW-4135): Only store reader registration in event.
        reader_certificate: Box<BorrowingCertificate>,
        reader_registration: Box<ReaderRegistration>,
        status: DisclosureStatus,
        r#type: DisclosureType,
    },
}

impl WalletEvent {
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

    use indexmap::IndexMap;
    use rstest::rstest;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::BorrowingCertificate;
    use mdoc::holder::ProposedDocumentAttributes;
    use mdoc::Entry;
    use mdoc::NameSpace;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadata;

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
}
