use indexmap::IndexMap;
use jsonwebtoken::Algorithm;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use ssri::Integrity;

use crypto::server_keys::KeyPair;
use crypto::EcdsaKeySend;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_from_p256;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataError;
use sd_jwt_vc_metadata::TypeMetadataValidationError;
use utils::date_time_seconds::DateTimeSeconds;

use crate::attributes::Attribute;
use crate::qualification::AttestationQualification;

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
    SdJwtSerialization(#[source] sd_jwt::error::Error),

    #[error("error converting to SD-JWT: {0}")]
    #[category(pd)]
    SdJwtCreation(#[from] sd_jwt::error::Error),

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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
    pub attributes: IndexMap<String, Attribute>,
}

pub trait IntoCredentialPayload {
    type Error;
    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error>;
}

impl IntoCredentialPayload for SdJwt {
    type Error = SdJwtCredentialPayloadError;

    fn into_credential_payload(self, metadata: &NormalizedTypeMetadata) -> Result<CredentialPayload, Self::Error> {
        let json = serde_json::Value::Object(
            self.into_disclosed_object()
                .map_err(SdJwtCredentialPayloadError::SdJwtSerialization)?,
        );

        metadata.validate(&json)?;
        let payload = serde_json::from_value(json)?;

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
            previewable_payload,
        };

        metadata.validate(&serde_json::to_value(&payload)?)?;
        Ok(payload)
    }

    pub async fn into_sd_jwt(
        self,
        type_metadata: &NormalizedTypeMetadata,
        holder_pubkey: &VerifyingKey,
        issuer_key: &KeyPair<impl EcdsaKeySend>,
    ) -> Result<SdJwt, SdJwtCredentialPayloadError> {
        let vct_integrity = self.vct_integrity.clone();
        let sd_jwt = type_metadata
            .claims()
            .iter()
            .try_fold(SdJwtBuilder::new(self)?, |builder, claim| match claim.sd {
                ClaimSelectiveDisclosureMetadata::Always | ClaimSelectiveDisclosureMetadata::Allowed => {
                    let json_path = claim
                        .to_json_path()
                        .map_err(SdJwtCredentialPayloadError::ClaimPathConversion)?;
                    builder
                        .make_concealable(&json_path)
                        .map_err(SdJwtCredentialPayloadError::SdJwtCreation)
                }
                _ => Ok(builder),
            })?
            .finish(
                Algorithm::ES256,
                vct_integrity,
                issuer_key.private_key(),
                vec![issuer_key.certificate().clone()],
                holder_pubkey,
            )
            .await?;

        Ok(sd_jwt)
    }
}

#[cfg(any(test, feature = "example_credential_payloads"))]
mod examples {
    use chrono::Duration;
    use chrono::Utc;
    use indexmap::IndexMap;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;
    use ssri::Integrity;

    use jwt::jwk::jwk_from_p256;
    use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;

    impl CredentialPayload {
        pub fn example_empty(verifying_key: &VerifyingKey) -> Self {
            let now = Utc::now();
            let confirmation_key = jwk_from_p256(verifying_key).unwrap();

            Self {
                issued_at: now.into(),
                confirmation_key: RequiredKeyBinding::Jwk(confirmation_key.clone()),
                vct_integrity: Integrity::from(""),
                previewable_payload: PreviewableCredentialPayload {
                    attestation_type: String::from("urn:eudi:pid:nl:1"),
                    issuer: "https://cert.issuer.example.com".parse().unwrap(),
                    expires: Some((now + Duration::days(365)).into()),
                    not_before: Some((now - Duration::days(1)).into()),
                    attestation_qualification: Default::default(),
                    attributes: IndexMap::new(),
                },
            }
        }

        pub fn example_family_name() -> Self {
            Self::example_with_attribute(
                "family_name",
                AttributeValue::Text(String::from("De Bruijn")),
                SigningKey::random(&mut OsRng).verifying_key(),
            )
        }

        pub fn example_with_attribute(key: &str, attr_value: AttributeValue, verifying_key: &VerifyingKey) -> Self {
            Self::example_with_attributes(vec![(key, attr_value)], verifying_key)
        }

        pub fn example_with_attributes(attrs: Vec<(&str, AttributeValue)>, verifying_key: &VerifyingKey) -> Self {
            let mut payload = CredentialPayload::example_empty(verifying_key);
            for (key, attr_value) in attrs {
                payload
                    .previewable_payload
                    .attributes
                    .insert(String::from(key), Attribute::Single(attr_value));
            }
            payload
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::TimeZone;
    use chrono::Utc;
    use futures::FutureExt;
    use indexmap::IndexMap;
    use jsonwebtoken::Algorithm;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;
    use ssri::Integrity;

    use crypto::server_keys::generate::Ca;
    use crypto::EcdsaKey;
    use jwt::jwk::jwk_from_p256;
    use jwt::EcdsaDecodingKey;
    use sd_jwt::builder::SdJwtBuilder;
    use sd_jwt::hasher::Sha256Hasher;
    use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
    use sd_jwt::sd_jwt::SdJwtPresentation;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::auth::issuer_auth::IssuerRegistration;
    use crate::credential_payload::IntoCredentialPayload;
    use crate::credential_payload::SdJwtCredentialPayloadError;
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
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: String::from("com.example.pid"),
                issuer: "https://com.example.org/pid/issuer".parse().unwrap(),
                expires: None,
                not_before: None,
                attestation_qualification: "QEAA".parse().unwrap(),
                attributes: IndexMap::from([
                    (
                        String::from("birth_date"),
                        Attribute::Single(AttributeValue::Text(String::from("1963-08-12"))),
                    ),
                    (
                        String::from("place_of_birth"),
                        Attribute::Nested(IndexMap::from([
                            (
                                String::from("locality"),
                                Attribute::Single(AttributeValue::Text(String::from("The Hague"))),
                            ),
                            (
                                String::from("country"),
                                Attribute::Nested(IndexMap::from([
                                    (
                                        String::from("name"),
                                        Attribute::Single(AttributeValue::Text(String::from("The Netherlands"))),
                                    ),
                                    (
                                        String::from("area_code"),
                                        Attribute::Single(AttributeValue::Integer(33)),
                                    ),
                                ])),
                            ),
                        ])),
                    ),
                    (
                        String::from("financial"),
                        Attribute::Nested(IndexMap::from([
                            (String::from("has_debt"), Attribute::Single(AttributeValue::Bool(true))),
                            (String::from("has_job"), Attribute::Single(AttributeValue::Bool(false))),
                            (
                                String::from("debt_amount"),
                                Attribute::Single(AttributeValue::Integer(-10_000)),
                            ),
                        ])),
                    ),
                ]),
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
            "urn:eudi:pid:nl:1",
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        ));

        let example_payload = CredentialPayload::example_family_name();

        let payload = CredentialPayload::from_previewable_credential_payload(
            example_payload.previewable_payload.clone(),
            Utc::now().into(),
            holder_key.verifying_key(),
            &metadata,
            Integrity::from(""),
        )
        .unwrap();

        assert_eq!(
            payload.previewable_payload.attestation_type,
            example_payload.previewable_payload.attestation_type,
        );
    }

    #[test]
    fn test_from_previewable_credential_payload_invalid() {
        let holder_key = SigningKey::random(&mut OsRng);

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            "urn:eudi:pid:nl:1",
            "family_name",
            JsonSchemaPropertyType::Number,
            None,
        ));

        let example_payload = CredentialPayload::example_family_name();

        let error = CredentialPayload::from_previewable_credential_payload(
            example_payload.previewable_payload.clone(),
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
        let confirmation_key = jwk_from_p256(holder_key.verifying_key()).unwrap();

        let issuer_key = SigningKey::random(&mut OsRng);

        let claims = json!({
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
            }
        });

        let sd_jwt = SdJwtBuilder::new(claims)
            .unwrap()
            .make_concealable("/birth_date")
            .unwrap()
            .make_concealable("/place_of_birth/locality")
            .unwrap()
            .make_concealable("/place_of_birth/country/name")
            .unwrap()
            .make_concealable("/place_of_birth/country/area_code")
            .unwrap()
            .add_decoys("/place_of_birth", 1)
            .unwrap()
            .add_decoys("", 2)
            .unwrap()
            .finish(
                Algorithm::ES256,
                Integrity::from(""),
                &issuer_key,
                vec![],
                holder_key.verifying_key(),
            )
            .now_or_never()
            .unwrap()
            .unwrap();

        let metadata =
            NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::credential_payload_sd_jwt_metadata());
        let payload = sd_jwt
            .clone()
            .into_credential_payload(&metadata)
            .expect("creating and validating CredentialPayload from SD-JWT should succeed");

        assert_eq!(
            payload.previewable_payload.attestation_type,
            sd_jwt.claims().properties.get("vct").and_then(|c| c.as_str()).unwrap()
        );
    }

    #[tokio::test]
    async fn test_to_sd_jwt() {
        let holder_key = SigningKey::random(&mut OsRng);

        let ca = Ca::generate("myca", Default::default()).unwrap();
        let cert_type = CertificateType::from(IssuerRegistration::new_mock());
        let issuer_key_pair = ca.generate_key_pair("mycert", cert_type, Default::default()).unwrap();

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_name(
            "urn:eudi:pid:nl:1",
            "family_name",
            JsonSchemaPropertyType::String,
            None,
        ));

        let credential_payload = CredentialPayload::example_with_attribute(
            "family_name",
            AttributeValue::Text(String::from("De Bruijn")),
            holder_key.verifying_key(),
        );

        let sd_jwt = credential_payload
            .into_sd_jwt(&metadata, holder_key.verifying_key(), &issuer_key_pair)
            .await
            .unwrap();

        let hasher = Sha256Hasher::new();
        let (presented_sd_jwt, _) = sd_jwt
            .into_presentation(
                &hasher,
                DateTime::from_timestamp_millis(1458304832).unwrap(),
                String::from("https://aud.example.com"),
                String::from("nonce123"),
                Algorithm::ES256,
            )
            .unwrap()
            .finish(&holder_key)
            .await
            .unwrap();

        SdJwtPresentation::parse_and_verify(
            &presented_sd_jwt.to_string(),
            &EcdsaDecodingKey::from(&issuer_key_pair.verifying_key().await.unwrap()),
            &Sha256Hasher,
            "https://aud.example.com",
            "nonce123",
            Duration::days(36500),
        )
        .unwrap();
    }
}
