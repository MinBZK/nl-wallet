use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_history_event::EventStatus;
use openid4vc::disclosure_session::VerifierCertificate;
use utils::vec_at_least::VecNonEmpty;

use crate::attestation::AttestationPresentation;

pub type DisclosureStatus = EventStatus;

#[derive(Debug, Clone, Copy)]
pub enum DataDisclosureStatus {
    Disclosed,
    NotDisclosed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletEvent {
    Issuance {
        id: Uuid,
        attestations: VecNonEmpty<AttestationPresentation>,
        timestamp: DateTime<Utc>,
    },
    Disclosure {
        id: Uuid,
        attestations: Vec<AttestationPresentation>,
        timestamp: DateTime<Utc>,
        // TODO (PVW-4135): Only store reader registration in event.
        reader_certificate: Box<BorrowingCertificate>,
        reader_registration: Box<ReaderRegistration>,
        status: DisclosureStatus,
        r#type: DisclosureType,
    },
}

impl WalletEvent {
    pub(crate) fn new_issuance(attestations: VecNonEmpty<AttestationPresentation>) -> Self {
        Self::Issuance {
            id: Uuid::now_v7(),
            attestations,
            timestamp: Utc::now(),
        }
    }

    fn new_disclosure(
        attestations: Option<VecNonEmpty<AttestationPresentation>>,
        verifier_certificate: VerifierCertificate,
        disclosure_type: DisclosureType,
        status: DisclosureStatus,
        data_status: DataDisclosureStatus,
    ) -> Self {
        let attestations = match data_status {
            DataDisclosureStatus::Disclosed => attestations.map(VecNonEmpty::into_inner),
            DataDisclosureStatus::NotDisclosed => None,
        }
        .unwrap_or_default();

        let (reader_certificate, reader_registration) = verifier_certificate.into_certificate_and_registration();

        Self::Disclosure {
            id: Uuid::now_v7(),
            attestations,
            timestamp: Utc::now(),
            reader_certificate: Box::new(reader_certificate),
            reader_registration: Box::new(reader_registration),
            status,
            r#type: disclosure_type,
        }
    }

    pub(crate) fn new_disclosure_success(
        attestations: VecNonEmpty<AttestationPresentation>,
        verifier_certificate: VerifierCertificate,
        disclosure_type: DisclosureType,
    ) -> Self {
        Self::new_disclosure(
            Some(attestations),
            verifier_certificate,
            disclosure_type,
            EventStatus::Success,
            DataDisclosureStatus::Disclosed,
        )
    }

    pub(crate) fn new_disclosure_error(
        attestations: VecNonEmpty<AttestationPresentation>,
        verifier_certificate: VerifierCertificate,
        disclosure_type: DisclosureType,
        data_status: DataDisclosureStatus,
    ) -> Self {
        Self::new_disclosure(
            Some(attestations),
            verifier_certificate,
            disclosure_type,
            EventStatus::Error,
            data_status,
        )
    }

    pub(crate) fn new_disclosure_cancel(
        attestations: Option<VecNonEmpty<AttestationPresentation>>,
        verifier_certificate: VerifierCertificate,
        disclosure_type: DisclosureType,
    ) -> Self {
        Self::new_disclosure(
            attestations,
            verifier_certificate,
            disclosure_type,
            EventStatus::Cancelled,
            DataDisclosureStatus::NotDisclosed,
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
    use indexmap::IndexMap;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use crypto::x509::BorrowingCertificate;
    use crypto::x509::BorrowingCertificateExtension;
    use mdoc::DataElementValue;
    use mdoc::Entry;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attestation::AttestationIdentity;
    use crate::issuance::BSN_ATTR_NAME;

    use super::*;

    fn mock_attestations_for_attestation_types(
        attestation_types: &[&str],
        issuer_certificate: &BorrowingCertificate,
    ) -> Vec<AttestationPresentation> {
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

                AttestationPresentation::create_for_issuance(
                    AttestationIdentity::Ephemeral,
                    metadata,
                    issuer_org,
                    attributes,
                )
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
