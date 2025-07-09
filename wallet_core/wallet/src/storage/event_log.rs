use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::disclosure_type::DisclosureType;
use crypto::x509::BorrowingCertificate;
use entity::disclosure_event::EventStatus;

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
        attestation: Box<AttestationPresentation>,
        timestamp: DateTime<Utc>,
        renewed: bool,
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
}
