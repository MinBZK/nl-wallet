use anyhow::Result;
use anyhow::ensure;
use attestation_data::registration_certificate::RegistrationCertificateAuthorizationError;
use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
use attestation_data::registration_certificate::RegistrationCertificateRequestParseError;
use attestation_data::registration_certificate::RegistrationCertificateStatusValidationError;
use attestation_data::registration_certificate::RegistrationCertificateValidationError;
use attestation_data::registration_certificate::SubjectType;
use attestation_data::registration_certificate::mock::MockRegistrationCertificateAuthority;
use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
use attestation_types::pid_constants::PID_GIVEN_NAME;
use attestation_types::pid_constants::PID_MDOC_DOCTYPE;
use chrono::Utc;
use ciborium::value::Value;
use cose::wrprc_cwt::WrprcCwtError;
use indexmap::IndexMap;
use jwt::error::JwtParseError;
use mdoc::DeviceResponseStatus;
use mdoc::ItemsRequest;
use serde_json::json;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_jwt::StatusListJwt;
use token_status_list::verification::client::StatusListClientError;

use crate::errors::DisclosureError;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError::RegistrationCertificateAuthorization;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError::RegistrationCertificateBinding;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError::RegistrationCertificateParsing;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError::RegistrationCertificateStatus;
use crate::wallet::close_proximity_disclosure::CloseProximityDisclosureError::RegistrationCertificateVerification;

const UNAUTHORIZED_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";

/// EUDI WRPRC rejection cases, shared by host tests and the native device runner.
#[derive(Clone, Copy, Debug, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RegistrationCertificateCase {
    Missing,
    WrongType,
    Malformed,
    TamperedSignature,
    Expired,
    Untrusted,
    MismatchedReader,
    MismatchedSubjectType,
    Revoked,
    Suspended,
    StatusUnavailable,
    UntrustedStatusList,
    StatusIndexOutOfBounds,
    UnauthorizedDocumentType,
    UnauthorizedNamespace,
    UnauthorizedAttribute,
    UnauthorizedSecondDocument,
}

impl RegistrationCertificateCase {
    pub(super) fn modify_payload(self, payload: &mut RegistrationCertificateFixture) {
        match self {
            Self::Expired => {
                let now = Utc::now().timestamp();
                payload.0["iat"] = json!(now - 180);
                payload.0["exp"] = json!(now - 120);
            }
            Self::MismatchedReader => payload.0["sub"] = json!("other-relying-party"),
            Self::MismatchedSubjectType => {
                payload.0["sub_ln"] = json!(null);
                payload.0["sub_gn"] = json!("Test");
                payload.0["sub_fn"] = json!("Person");
            }
            // A packed ten-entry list is padded to sixteen entries.
            Self::StatusIndexOutOfBounds => payload.0["status"]["idx"] = json!("16"),
            _ => {}
        }
    }

    pub(super) fn certificate(
        self,
        authority: &MockRegistrationCertificateAuthority,
        payload: &RegistrationCertificateFixture,
    ) -> Option<Value> {
        let certificate = match self {
            Self::Missing => return None,
            Self::WrongType => return Some(Value::Text("not a byte string".to_owned())),
            Self::Malformed => vec![0xff],
            Self::Untrusted => MockRegistrationCertificateAuthority::new().sign_cwt(payload),
            _ => {
                let mut certificate = authority.sign_cwt(payload);
                if matches!(self, Self::TamperedSignature) {
                    // COSE_Sign1 ends with the signature; preserve the envelope structure.
                    *certificate.last_mut().unwrap() ^= 1;
                }
                certificate
            }
        };
        Some(Value::Bytes(certificate))
    }

    pub(super) fn status_list(
        self,
        authority: &MockRegistrationCertificateAuthority,
    ) -> Result<StatusListJwt, StatusListClientError> {
        match self {
            Self::Revoked | Self::Suspended => {
                let mut list = StatusList::new(10);
                list.insert(
                    0,
                    if matches!(self, Self::Revoked) {
                        StatusType::Invalid
                    } else {
                        StatusType::Suspended
                    },
                );
                Ok(authority.sign_status_list(list))
            }
            // Simulate retrieval returning an unusable status-list token, without network dependencies.
            Self::StatusUnavailable => Err(StatusListClientError::JwtParsing(
                JwtParseError::UnexpectedNumberOfParts(0),
            )),
            Self::UntrustedStatusList => Ok(MockRegistrationCertificateAuthority::new().status_list_client.0),
            _ => Ok(authority.status_list_client.0.clone()),
        }
    }

    pub(super) fn items_requests(self, mut request: ItemsRequest) -> Result<Vec<ItemsRequest>> {
        match self {
            Self::UnauthorizedDocumentType => request.doc_type = UNAUTHORIZED_DOC_TYPE.to_owned(),
            Self::UnauthorizedNamespace => {
                request.name_spaces = IndexMap::from([(
                    "unauthorized.namespace".to_owned(),
                    IndexMap::from([(PID_GIVEN_NAME.to_owned(), false)]).try_into()?,
                )])
                .try_into()?;
            }
            Self::UnauthorizedAttribute => {
                // Keep an authorized attribute too, to detect validation that checks only the first claim.
                request.name_spaces = IndexMap::from([(
                    PID_MDOC_DOCTYPE.to_owned(),
                    IndexMap::from([
                        (PID_GIVEN_NAME.to_owned(), false),
                        ("unauthorized_attribute".to_owned(), false),
                    ])
                    .try_into()?,
                )])
                .try_into()?;
            }
            Self::UnauthorizedSecondDocument => {
                let mut second = request.clone();
                second.doc_type = UNAUTHORIZED_DOC_TYPE.to_owned();
                return Ok(vec![request, second]);
            }
            _ => {}
        }
        Ok(vec![request])
    }

    pub(super) fn response_status(self) -> DeviceResponseStatus {
        match self {
            Self::Missing | Self::WrongType | Self::Malformed => DeviceResponseStatus::InvalidRequest,
            _ => DeviceResponseStatus::GeneralError,
        }
    }

    pub(super) fn check_error(self, error: &DisclosureError) -> Result<()> {
        use RegistrationCertificateEnvelopeError as Envelope;
        use RegistrationCertificateRequestParseError as Parse;
        use RegistrationCertificateStatusValidationError as Status;
        use RegistrationCertificateValidationError as Validation;

        let matches = match (self, error) {
            (case, DisclosureError::CloseProximityDisclosureSessionError(error)) => match (case, error) {
                (Self::Missing, RegistrationCertificateParsing(Parse::Missing))
                | (Self::WrongType, RegistrationCertificateParsing(Parse::NotByteString))
                | (Self::Malformed, RegistrationCertificateParsing(Parse::Envelope(_)))
                | (Self::TamperedSignature | Self::Untrusted, RegistrationCertificateVerification(Envelope::Cwt(_)))
                | (Self::Expired, RegistrationCertificateVerification(Envelope::Cwt(WrprcCwtError::Expired)))
                | (
                    Self::MismatchedReader,
                    RegistrationCertificateBinding(Validation::SubjectIdentifierMismatch {
                        field: "organizationIdentifier",
                    }),
                )
                | (
                    Self::MismatchedSubjectType,
                    RegistrationCertificateBinding(Validation::AccessCertificateSubjectTypeMismatch {
                        registration: SubjectType::NaturalPerson,
                        access: SubjectType::LegalPerson,
                    }),
                )
                | (Self::Revoked | Self::Suspended, RegistrationCertificateStatus(Status::NotValid))
                | (Self::StatusUnavailable, RegistrationCertificateStatus(Status::Undetermined))
                | (
                    Self::UntrustedStatusList | Self::StatusIndexOutOfBounds,
                    RegistrationCertificateStatus(Status::InvalidStatusListReference),
                ) => true,
                (
                    Self::UnauthorizedDocumentType | Self::UnauthorizedSecondDocument,
                    RegistrationCertificateAuthorization(
                        RegistrationCertificateAuthorizationError::UnauthorizedItemsRequest(doc_type),
                    ),
                ) => doc_type == UNAUTHORIZED_DOC_TYPE,
                (
                    Self::UnauthorizedNamespace | Self::UnauthorizedAttribute,
                    RegistrationCertificateAuthorization(
                        RegistrationCertificateAuthorizationError::UnauthorizedItemsRequest(doc_type),
                    ),
                ) => doc_type == PID_MDOC_DOCTYPE,
                _ => false,
            },
            _ => false,
        };
        ensure!(matches, "{self}: unexpected rejection: {error:?}");
        Ok(())
    }
}
