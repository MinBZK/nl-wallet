use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use attestation_types::qualification::AttestationQualification;
use crypto::EcdsaKey;
use crypto::server_keys::KeyPair;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_from_p256;
use mdoc::Entry;
use mdoc::MobileSecurityObject;
use mdoc::NameSpace;
use mdoc::holder::Mdoc;
use mdoc::utils::crypto::CryptoError;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::builder::SignedSdJwt;
use sd_jwt::claims::ClaimNameError;
use sd_jwt::key_binding_jwt::RequiredKeyBinding;
use sd_jwt::sd_jwt::SdJwtVcClaims;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataError;
use sd_jwt_vc_metadata::TypeMetadataValidationError;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;

use crate::attributes::Attributes;
use crate::attributes::AttributesError;
use crate::attributes::AttributesTraversalBehaviour;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum SdJwtCredentialPayloadError {
    #[error("error converting to / from JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[from] serde_json::Error),

    #[error("metadata validation error: {0}")]
    #[category(pd)]
    MetadataValidation(#[from] TypeMetadataValidationError),

    #[error("error converting from SD-JWT: {0}")]
    #[category(pd)]
    SdJwtDecoding(#[from] sd_jwt::error::DecoderError),

    #[error("error converting AttributeName to ClaimName: {0}")]
    #[category(pd)]
    InvalidClaimName(#[from] ClaimNameError),

    #[error("error converting claims to attributes: {0}")]
    #[category(pd)]
    InvalidAttributes(#[from] AttributesError),

    #[error("missing Attestation Qualification")]
    #[category(critical)]
    MissingAttestationQualification,

    #[error("missing Metadata Integrity")]
    #[category(critical)]
    MissingMetadataIntegrity,

    #[error("error converting to SD-JWT: {0}")]
    #[category(pd)]
    SdJwtEncoding(#[from] sd_jwt::error::EncoderError),

    #[error("error converting claim path to JSON path: {0}")]
    #[category(pd)]
    ClaimPathConversion(#[source] TypeMetadataError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[from] JwkConversionError),
}

/// This struct represents the Claims Set received from the issuer. Its JSON representation should be verifiable by the
/// JSON schema defined in the SD-JWT VC Type Metadata (`TypeMetadata`).
///
/// Converting both an (unsigned) mdoc and SD-JWT document to this struct should yield the same result.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct CredentialPayload {
    #[serde(rename = "iat")]
    pub issued_at: DateTimeSeconds,

    /// Contains the attestation's public key, of which the corresponding private key is used by the wallet during
    /// disclosure to sign the RP's nonce into a PoP
    #[serde(rename = "cnf")]
    pub confirmation_key: RequiredKeyBinding,

    /// Contains the integrity digest of the type metadata document of this `vct`.
    #[serde(rename = "vct#integrity")]
    pub vct_integrity: Integrity,

    /// The information on how to read the status of the Verifiable Credential.
    pub status: Option<serde_json::Value>,

    #[serde(flatten)]
    pub previewable_payload: PreviewableCredentialPayload,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreviewableCredentialPayload {
    #[serde(rename = "vct")]
    pub attestation_type: String,

    #[serde(rename = "iss")]
    pub issuer: HttpsUri,

    #[serde(rename = "exp")]
    pub expires: Option<DateTimeSeconds>,

    #[serde(rename = "nbf")]
    pub not_before: Option<DateTimeSeconds>,

    pub attestation_qualification: AttestationQualification,

    #[serde(flatten)]
    pub attributes: Attributes,
}

impl PreviewableCredentialPayload {
    pub fn matches_existing(
        &self,
        existing: &PreviewableCredentialPayload,
        time: &impl Generator<DateTime<Utc>>,
    ) -> bool {
        // Compare all fields except `not_before`
        if self.attestation_type == existing.attestation_type
            && self.issuer == existing.issuer
            && self.attestation_qualification == existing.attestation_qualification
            && self.attributes == existing.attributes
        {
            // If `not_before` matches as well, they definitely match
            if self.not_before == existing.not_before {
                return true;
            }

            // If not, it is only considered a match if `not_before` from the new preview (self) is in the past
            if let Some(self_nbf) = self.not_before {
                let is_nbf_in_the_past = self_nbf.as_ref() <= &time.generate();
                return is_nbf_in_the_past;
            }
        }

        false
    }
}

pub trait IntoCredentialPayload {
    type Error;
    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error>;
}

impl TryFrom<CredentialPayload> for SdJwtVcClaims {
    type Error = ClaimNameError;

    fn try_from(value: CredentialPayload) -> Result<Self, Self::Error> {
        Ok(SdJwtVcClaims {
            vct: value.previewable_payload.attestation_type,
            vct_integrity: Some(value.vct_integrity),
            iss: value.previewable_payload.issuer,
            iat: value.issued_at,
            exp: value.previewable_payload.expires,
            nbf: value.previewable_payload.not_before,
            cnf: value.confirmation_key,
            attestation_qualification: Some(value.previewable_payload.attestation_qualification),
            _sd_alg: None, // to be set by SdJwtBuilder

            claims: value.previewable_payload.attributes.try_into()?,
        })
    }
}

impl IntoCredentialPayload for &VerifiedSdJwt {
    type Error = SdJwtCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        let payload = CredentialPayload::from_sd_jwt(self)?;
        metadata.validate(&serde_json::to_value(&payload)?)?;
        Ok(payload)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum MdocCredentialPayloadError {
    #[error("error converting to / from JSON: {0}")]
    #[category(pd)]
    JsonConversion(#[from] serde_json::Error),

    #[error("metadata validation error: {0}")]
    #[category(pd)]
    MetadataValidation(#[from] TypeMetadataValidationError),

    #[error("unable to convert mdoc TDate to DateTime<Utc>")]
    #[category(critical)]
    DateConversion(#[from] chrono::ParseError),

    #[error("mdoc is missing issuer URI")]
    #[category(critical)]
    MissingIssuerUri,

    #[error("mdoc is missing attestation qualification")]
    #[category(critical)]
    MissingAttestationQualification,

    #[error("mdoc is missing metadata integrity")]
    #[category(critical)]
    MissingMetadataIntegrity,

    #[error("attributes error: {0}")]
    #[category(pd)]
    Attributes(#[from] AttributesError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[from] JwkConversionError),

    #[error("error converting holder public CoseKey to a VerifyingKey: {0}")]
    #[category(pd)]
    CoseKeyConversion(#[from] CryptoError),
}

impl IntoCredentialPayload for Mdoc {
    type Error = MdocCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        MdocParts::from(self).into_credential_payload(metadata)
    }
}

#[derive(derive_more::Constructor)]
pub struct MdocParts {
    attributes: IndexMap<NameSpace, Vec<Entry>>,
    mso: MobileSecurityObject,
}

impl From<Mdoc> for MdocParts {
    fn from(value: Mdoc) -> Self {
        let (mso, _, issuer_signed) = value.into_components();

        Self::new(issuer_signed.into_entries_by_namespace(), mso)
    }
}

impl IntoCredentialPayload for MdocParts {
    type Error = MdocCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        let payload = CredentialPayload::from_mdoc_parts_unvalidated(self, metadata)?;

        metadata.validate(&serde_json::to_value(&payload)?)?;

        Ok(payload)
    }
}

impl CredentialPayload {
    pub fn from_previewable_credential_payload(
        previewable_payload: PreviewableCredentialPayload,
        issued_at: DateTimeSeconds,
        holder_pubkey: &VerifyingKey,
        metadata: &NormalizedTypeMetadata,
        metadata_integrity: Integrity,
    ) -> Result<Self, SdJwtCredentialPayloadError> {
        let payload = CredentialPayload {
            issued_at,
            confirmation_key: RequiredKeyBinding::Jwk(jwk_from_p256(holder_pubkey)?),
            vct_integrity: metadata_integrity,
            status: None,
            previewable_payload,
        };

        metadata.validate(&serde_json::to_value(&payload)?)?;
        Ok(payload)
    }

    pub fn from_sd_jwt(sd_jwt: &VerifiedSdJwt) -> Result<Self, SdJwtCredentialPayloadError> {
        let credential_payload = CredentialPayload {
            issued_at: sd_jwt.claims().iat,
            confirmation_key: sd_jwt.claims().cnf.to_owned(),
            vct_integrity: sd_jwt
                .claims()
                .vct_integrity
                .as_ref()
                .ok_or(SdJwtCredentialPayloadError::MissingMetadataIntegrity)?
                .to_owned(),
            status: None,
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: sd_jwt.claims().vct.clone(),
                issuer: sd_jwt.claims().iss.clone(),
                expires: sd_jwt.claims().exp,
                not_before: sd_jwt.claims().nbf,
                attestation_qualification: sd_jwt
                    .claims()
                    .attestation_qualification
                    .ok_or(SdJwtCredentialPayloadError::MissingAttestationQualification)?,
                attributes: sd_jwt.decoded_claims()?.try_into()?,
            },
        };

        Ok(credential_payload)
    }

    fn from_mdoc_parts_unvalidated(
        MdocParts { attributes, mso }: MdocParts,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<Self, MdocCredentialPayloadError> {
        let holder_pub_key = VerifyingKey::try_from(mso.device_key_info)?;

        let payload = CredentialPayload {
            issued_at: (&mso.validity_info.signed).try_into()?,
            confirmation_key: jwk_from_p256(&holder_pub_key).map(RequiredKeyBinding::Jwk)?,
            vct_integrity: mso
                .type_metadata_integrity
                .ok_or(MdocCredentialPayloadError::MissingMetadataIntegrity)?,
            status: None,
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: mso.doc_type,
                issuer: mso.issuer_uri.ok_or(MdocCredentialPayloadError::MissingIssuerUri)?,
                expires: Some((&mso.validity_info.valid_until).try_into()?),
                not_before: Some((&mso.validity_info.valid_from).try_into()?),
                attestation_qualification: mso
                    .attestation_qualification
                    .ok_or(MdocCredentialPayloadError::MissingAttestationQualification)?,
                attributes: Attributes::from_mdoc_attributes(metadata, attributes)?,
            },
        };

        Ok(payload)
    }

    pub fn from_mdoc_unvalidated(
        mdoc: Mdoc,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<Self, MdocCredentialPayloadError> {
        Self::from_mdoc_parts_unvalidated(mdoc.into(), metadata)
    }

    pub async fn into_sd_jwt(
        self,
        type_metadata: &NormalizedTypeMetadata,
        issuer_keypair: &KeyPair<impl EcdsaKey>,
    ) -> Result<SignedSdJwt, SdJwtCredentialPayloadError> {
        let sd_by_claims = type_metadata
            .claims()
            .iter()
            .map(|claim| (&claim.path, claim.sd))
            .collect::<HashMap<_, _>>();

        let sd_jwt = self
            .previewable_payload
            .attributes
            .claim_paths(AttributesTraversalBehaviour::AllPaths)
            .into_iter()
            .try_fold(SdJwtBuilder::new(self.try_into()?), |builder, claims| {
                let should_be_selectively_discloseable = match sd_by_claims.get(&claims) {
                    Some(sd) => !matches!(sd, ClaimSelectiveDisclosureMetadata::Never),
                    None => true,
                };

                if !should_be_selectively_discloseable {
                    return Ok(builder);
                }

                builder
                    .make_concealable(claims)
                    .map_err(SdJwtCredentialPayloadError::SdJwtEncoding)
            })?
            .finish(issuer_keypair)
            .await?;

        Ok(sd_jwt)
    }
}

#[cfg(any(test, feature = "example_credential_payloads"))]
mod examples {
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;
    use p256::ecdsa::VerifyingKey;
    use ssri::Integrity;

    use jwt::jwk::jwk_from_p256;
    use sd_jwt::key_binding_jwt::RequiredKeyBinding;
    use utils::generator::Generator;

    use crate::attributes::AttributeValue;
    use crate::attributes::Attributes;
    use crate::pid_constants::PID_ATTESTATION_TYPE;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;

    impl CredentialPayload {
        pub(super) fn example_with_preview(
            previewable_payload: PreviewableCredentialPayload,
            verifying_key: &VerifyingKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let time = time_generator.generate();

            let confirmation_key = jwk_from_p256(verifying_key).unwrap();

            Self {
                issued_at: time.into(),
                confirmation_key: RequiredKeyBinding::Jwk(confirmation_key.clone()),
                vct_integrity: Integrity::from(""),
                status: None,
                previewable_payload,
            }
        }

        pub fn example_with_attributes(
            attestation_type: &str,
            attributes: Attributes,
            verifying_key: &VerifyingKey,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            let previewable_payload =
                PreviewableCredentialPayload::example_with_attributes(attestation_type, attributes, time_generator);

            Self::example_with_preview(previewable_payload, verifying_key, time_generator)
        }
    }

    impl PreviewableCredentialPayload {
        pub fn example_empty(attestation_type: &str, time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            let time = time_generator.generate();

            Self {
                attestation_type: attestation_type.to_string(),
                issuer: "https://cert.issuer.example.com".parse().unwrap(),
                expires: Some((time + Duration::days(365)).into()),
                not_before: Some((time - Duration::days(1)).into()),
                attestation_qualification: Default::default(),
                attributes: Attributes::default(),
            }
        }

        pub fn example_family_name(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                PID_ATTESTATION_TYPE,
                Attributes::example([(["family_name"], AttributeValue::Text(String::from("De Bruijn")))]),
                time_generator,
            )
        }

        pub fn example_with_attributes(
            attestation_type: &str,
            attributes: Attributes,
            time_generator: &impl Generator<DateTime<Utc>>,
        ) -> Self {
            Self {
                attributes,
                ..Self::example_empty(attestation_type, time_generator)
            }
        }
    }
}

#[cfg(feature = "mock")]
mod mock {
    use chrono::DateTime;
    use chrono::Utc;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use utils::generator::Generator;

    use crate::attributes::Attributes;
    use crate::pid_constants::ADDRESS_ATTESTATION_TYPE;
    use crate::pid_constants::PID_ATTESTATION_TYPE;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;

    impl CredentialPayload {
        pub fn nl_pid_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            let previewable_payload = PreviewableCredentialPayload::nl_pid_example(time_generator);

            Self::example_with_preview(
                previewable_payload,
                SigningKey::random(&mut OsRng).verifying_key(),
                time_generator,
            )
        }
    }

    impl PreviewableCredentialPayload {
        pub fn nl_pid_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(PID_ATTESTATION_TYPE, Attributes::nl_pid_example(), time_generator)
        }

        pub fn nl_pid_address_example(time_generator: &impl Generator<DateTime<Utc>>) -> Self {
            Self::example_with_attributes(
                ADDRESS_ATTESTATION_TYPE,
                Attributes::nl_pid_address_example(),
                time_generator,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use assert_matches::assert_matches;
    use chrono::TimeZone;
    use chrono::Utc;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;
    use ssri::Integrity;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::qualification::AttestationQualification;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteWscd;
    use crypto::server_keys::generate::Ca;
    use jwt::jwk::jwk_from_p256;
    use sd_jwt::builder::SdJwtBuilder;
    use sd_jwt::key_binding_jwt::KbVerificationOptions;
    use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
    use sd_jwt::key_binding_jwt::RequiredKeyBinding;
    use sd_jwt::sd_jwt::SdJwtVcClaims;
    use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::attributes::Attributes;
    use crate::attributes::test::complex_attributes;
    use crate::auth::issuer_auth::IssuerRegistration;
    use crate::credential_payload::IntoCredentialPayload;
    use crate::credential_payload::SdJwtCredentialPayloadError;
    use crate::pid_constants::PID_ATTESTATION_TYPE;
    use crate::x509::CertificateType;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;

    #[test]
    fn test_serialize_deserialize_and_validate() {
        let confirmation_key = jwk_from_p256(SigningKey::random(&mut OsRng).verifying_key()).unwrap();

        let payload = CredentialPayload {
            issued_at: Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap().into(),
            confirmation_key: RequiredKeyBinding::Jwk(confirmation_key.clone()),
            vct_integrity: Integrity::from(""),
            status: None,
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: String::from("com.example.pid"),
                issuer: "https://com.example.org/pid/issuer".parse().unwrap(),
                expires: None,
                not_before: None,
                attestation_qualification: "QEAA".parse().unwrap(),
                attributes: complex_attributes().into(),
            },
        };

        let expected_json = json!({
            "vct": "com.example.pid",
            "vct#integrity": "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
            "iss": "https://com.example.org/pid/issuer",
            "iat": 61,
            "attestation_qualification": "QEAA",
            "cnf": {
                "jwk": confirmation_key
            },
            "birth_date": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 33
                }
            },
            "financial": {
                "has_debt": true,
                "has_job": false,
                "debt_amount": -10000
            }
        });

        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json, expected_json);

        let metadata = NormalizedTypeMetadata::example();
        metadata.validate(&json).expect("CredentialPayload should be valid");
    }

    #[test]
    fn test_from_previewable_credential_payload() {
        let holder_key = SigningKey::random(&mut OsRng);

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            PID_ATTESTATION_TYPE,
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        ));

        let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let payload = CredentialPayload::from_previewable_credential_payload(
            preview_payload.clone(),
            Utc::now().into(),
            holder_key.verifying_key(),
            &metadata,
            Integrity::from(""),
        )
        .unwrap();

        assert_eq!(
            payload.previewable_payload.attestation_type,
            preview_payload.attestation_type,
        );
    }

    #[test]
    fn test_from_previewable_credential_payload_invalid() {
        let holder_key = SigningKey::random(&mut OsRng);

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            PID_ATTESTATION_TYPE,
            "family_name",
            JsonSchemaPropertyType::Number,
            None,
        ));

        let preview_payload = PreviewableCredentialPayload::example_family_name(&MockTimeGenerator::default());

        let error = CredentialPayload::from_previewable_credential_payload(
            preview_payload,
            Utc::now().into(),
            holder_key.verifying_key(),
            &metadata,
            Integrity::from(""),
        )
        .expect_err("wrong family_name type should fail validation");

        assert_matches!(error, SdJwtCredentialPayloadError::MetadataValidation(_));
    }

    #[test]
    fn test_from_sd_jwt() {
        let holder_key = SigningKey::random(&mut OsRng);

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_keypair = ca.generate_issuer_mock().unwrap();

        let claims = SdJwtVcClaims::example_from_json(
            holder_key.verifying_key(),
            json!({
                "birth_date": "1963-08-12",
                "place_of_birth": {
                    "locality": "The Hague",
                    "country": {
                        "name": "The Netherlands",
                        "area_code": 33
                    }
                },
            }),
            &MockTimeGenerator::default(),
        );

        let sd_jwt = SdJwtBuilder::new(claims)
            .make_concealable(
                vec![ClaimPath::SelectByKey(String::from("birth_date"))]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .make_concealable(
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("locality")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap()
            .make_concealable(
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("name")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap()
            .make_concealable(
                vec![
                    ClaimPath::SelectByKey(String::from("place_of_birth")),
                    ClaimPath::SelectByKey(String::from("country")),
                    ClaimPath::SelectByKey(String::from("area_code")),
                ]
                .try_into()
                .unwrap(),
            )
            .unwrap()
            .add_decoys(&[ClaimPath::SelectByKey(String::from("place_of_birth"))], 1)
            .unwrap()
            .add_decoys(&[], 2)
            .unwrap()
            .finish(&issuer_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into_verified();

        let metadata =
            NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::credential_payload_sd_jwt_metadata());
        let payload = sd_jwt
            .clone()
            .into_credential_payload(&metadata)
            .expect("creating and validating CredentialPayload from SD-JWT should succeed");

        assert_eq!(payload.previewable_payload.attestation_type, sd_jwt.claims().vct);

        let unverified_payload = CredentialPayload::from_sd_jwt(&sd_jwt)
            .expect("creating a CredentialPayload from SD-JWT while not validating metdata should succeed");

        assert_eq!(payload, unverified_payload);
    }

    #[test]
    fn test_to_sd_jwt() {
        let time_generator = MockTimeGenerator::default();

        let holder_key = MockRemoteEcdsaKey::new_random("holder_key".to_string());
        let wscd = MockRemoteWscd::new(vec![holder_key.clone()]);

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            PID_ATTESTATION_TYPE,
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        ));

        let credential_payload = CredentialPayload::example_with_attributes(
            PID_ATTESTATION_TYPE,
            Attributes::example([(["family_name"], AttributeValue::Text(String::from("De Bruijn")))]),
            holder_key.verifying_key(),
            &time_generator,
        );

        let sd_jwt = credential_payload
            .into_sd_jwt(&metadata, &issuer_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        let (presented_sd_jwts, _poa) = UnsignedSdJwtPresentation::sign_multiple(
            vec_nonempty![(
                sd_jwt.into_verified().into_presentation_builder().finish(),
                "holder_key"
            )],
            KeyBindingJwtBuilder::new(String::from("https://aud.example.com"), String::from("nonce123")),
            &wscd,
            (),
            &MockTimeGenerator::default(),
        )
        .now_or_never()
        .unwrap()
        .expect("signing a single SdJwtPresentation using the WSCD should succeed");

        let kb_verification_options = KbVerificationOptions {
            expected_aud: "https://aud.example.com",
            expected_nonce: "nonce123",
            iat_leeway: 5,
            iat_acceptance_window: Duration::from_secs(60),
        };

        let presented_sd_jwt = presented_sd_jwts.into_iter().exactly_one().unwrap().into_unverified();
        presented_sd_jwt
            .into_verified_against_trust_anchors(&[ca.to_trust_anchor()], &kb_verification_options, &time_generator)
            .unwrap();
    }

    #[test]
    fn test_matches_existing() {
        let epoch_generator = MockTimeGenerator::epoch();

        let mut new = PreviewableCredentialPayload {
            attestation_type: String::from("att_type_1"),
            issuer: "https://issuer.example.com".parse().unwrap(),
            expires: Some(Utc.with_ymd_and_hms(2000, 1, 1, 0, 1, 1).unwrap().into()),
            not_before: Some(Utc.with_ymd_and_hms(1969, 1, 1, 0, 1, 1).unwrap().into()),
            attestation_qualification: AttestationQualification::PubEAA,
            attributes: IndexMap::from([(
                String::from("attr1"),
                Attribute::Single(AttributeValue::Text(String::from("val1"))),
            )])
            .into(),
        };

        let mut existing = new.clone();
        assert!(new.matches_existing(&existing, &epoch_generator));

        existing.attestation_type = String::from("att_type_2");
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.issuer = "https://other_issuer.example.com".parse().unwrap();
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.attestation_qualification = AttestationQualification::QEAA;
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.attributes = IndexMap::from([(
            String::from("attr1"),
            Attribute::Single(AttributeValue::Text(String::from("val2"))),
        )])
        .into();
        assert!(!new.matches_existing(&existing, &epoch_generator));

        let mut existing = new.clone();
        existing.not_before = Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap().into());
        assert!(
            new.matches_existing(&existing, &epoch_generator),
            "the payloads should match if the nbf of the new payload is in the past and the rest is the same"
        );

        let existing = new.clone();
        new.not_before = Some(Utc.with_ymd_and_hms(1980, 1, 1, 0, 1, 1).unwrap().into());
        assert!(
            !new.matches_existing(&existing, &epoch_generator),
            "the payloads should not match if the nbf of the new payload is in the future and different from the \
             existing payload"
        );
    }
}
