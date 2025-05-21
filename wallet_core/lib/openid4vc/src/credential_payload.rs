use indexmap::IndexMap;
use jsonwebtoken::Algorithm;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use crypto::server_keys::KeyPair;
use crypto::EcdsaKeySend;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use jwt::error::JwkConversionError;
use jwt::jwk::jwk_from_p256;
use mdoc::holder::Mdoc;
use mdoc::unsigned::Entry;
use mdoc::unsigned::UnsignedMdoc;
use mdoc::utils::crypto::CryptoError;
use mdoc::AttestationQualification;
use mdoc::MobileSecurityObject;
use mdoc::NameSpace;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataError;
use sd_jwt_vc_metadata::TypeMetadataValidationError;
use utils::date_time_seconds::DateTimeSeconds;

use crate::attributes::Attribute;
use crate::attributes::AttributeError;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPayloadError {
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

    #[error("attribute error: {0}")]
    #[category(pd)]
    Attribute(#[from] AttributeError),

    #[error("error converting holder VerifyingKey to JWK: {0}")]
    #[category(pd)]
    JwkConversion(#[from] JwkConversionError),

    #[error("error converting holder public CoseKey to a VerifyingKey: {0}")]
    #[category(pd)]
    CoseKeyConversion(#[from] CryptoError),

    #[error("no attributes present in PreviewableCredentialPayload")]
    #[category(critical)]
    NoAttributes,

    #[error("missing either the \"exp\" or \"nbf\" timestamp")]
    #[category(critical)]
    MissingValidityTimestamp,

    #[error("error converting from SD-JWT: {0}")]
    #[category(pd)]
    SdJwtSerialization(#[source] sd_jwt::error::Error),

    #[error("error converting to SD-JWT: {0}")]
    #[category(pd)]
    SdJwtCreation(#[from] sd_jwt::error::Error),

    #[error("error converting claim path to JSON path: {0}")]
    #[category(pd)]
    ClaimPathConversion(#[source] TypeMetadataError),
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

impl CredentialPayload {
    pub fn from_mdoc_parts(
        attributes: IndexMap<NameSpace, Vec<Entry>>,
        mso: MobileSecurityObject,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<Self, CredentialPayloadError> {
        let holder_pub_key = VerifyingKey::try_from(mso.device_key_info)?;

        let payload = Self {
            issued_at: (&mso.validity_info.signed).try_into()?,
            confirmation_key: jwk_from_p256(&holder_pub_key).map(RequiredKeyBinding::Jwk)?,
            previewable_payload: PreviewableCredentialPayload {
                attestation_type: mso.doc_type,
                issuer: mso.issuer_uri.ok_or(CredentialPayloadError::MissingIssuerUri)?,
                expires: Some((&mso.validity_info.valid_until).try_into()?),
                not_before: Some((&mso.validity_info.valid_from).try_into()?),
                attestation_qualification: mso
                    .attestation_qualification
                    .ok_or(CredentialPayloadError::MissingAttestationQualification)?,
                attributes: Attribute::from_mdoc_attributes(metadata, attributes)?,
            },
        };

        Self::validate(&serde_json::to_value(&payload)?, metadata)?;

        Ok(payload)
    }

    pub fn from_mdoc(mdoc: Mdoc, metadata: &NormalizedTypeMetadata) -> Result<Self, CredentialPayloadError> {
        Self::from_mdoc_parts(mdoc.issuer_signed.into_entries_by_namespace(), mdoc.mso, metadata)
    }

    pub fn from_previewable_credential_payload(
        previewable_payload: PreviewableCredentialPayload,
        issued_at: DateTimeSeconds,
        holder_pubkey: &VerifyingKey,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<Self, CredentialPayloadError> {
        let payload = CredentialPayload {
            issued_at,
            confirmation_key: RequiredKeyBinding::Jwk(jwk_from_p256(holder_pubkey)?),
            previewable_payload,
        };

        Self::validate(&serde_json::to_value(&payload)?, metadata)?;

        Ok(payload)
    }

    pub fn from_sd_jwt(sd_jwt: SdJwt, metadata: &NormalizedTypeMetadata) -> Result<Self, CredentialPayloadError> {
        let json = serde_json::Value::Object(
            sd_jwt
                .into_disclosed_object()
                .map_err(CredentialPayloadError::SdJwtSerialization)?,
        );

        Self::validate(&json, metadata)?;

        let payload = serde_json::from_value(json)?;

        Ok(payload)
    }

    pub async fn into_sd_jwt(
        self,
        type_metadata: &NormalizedTypeMetadata,
        holder_pubkey: &VerifyingKey,
        issuer_key: &KeyPair<impl EcdsaKeySend>,
    ) -> Result<SdJwt, CredentialPayloadError> {
        let sd_jwt = type_metadata
            .claims()
            .iter()
            .try_fold(SdJwtBuilder::new(self)?, |builder, claim| match claim.sd {
                ClaimSelectiveDisclosureMetadata::Always | ClaimSelectiveDisclosureMetadata::Allowed => {
                    let json_path = claim
                        .to_json_path()
                        .map_err(CredentialPayloadError::ClaimPathConversion)?;
                    builder
                        .make_concealable(&json_path)
                        .map_err(CredentialPayloadError::SdJwtCreation)
                }
                _ => Ok(builder),
            })?
            .finish(
                Algorithm::ES256,
                issuer_key.private_key(),
                issuer_key.certificate().to_vec(),
                holder_pubkey,
            )
            .await?;

        Ok(sd_jwt)
    }

    fn validate(
        credential_payload: &serde_json::Value,
        metadata: &NormalizedTypeMetadata,
    ) -> Result<(), CredentialPayloadError> {
        metadata.validate(credential_payload)?;

        Ok(())
    }
}

impl PreviewableCredentialPayload {
    pub fn into_unsigned_mdoc(self) -> Result<UnsignedMdoc, CredentialPayloadError> {
        let attributes = Attribute::from_attributes(&self.attestation_type, self.attributes);

        let unsigned_mdoc = UnsignedMdoc {
            doc_type: self.attestation_type,
            attributes: attributes
                .try_into()
                .map_err(|_| CredentialPayloadError::NoAttributes)?,
            valid_from: self
                .not_before
                .ok_or(CredentialPayloadError::MissingValidityTimestamp)?
                .into(),
            valid_until: self
                .expires
                .ok_or(CredentialPayloadError::MissingValidityTimestamp)?
                .into(),
            issuer_uri: self.issuer,
            attestation_qualification: self.attestation_qualification,
        };

        Ok(unsigned_mdoc)
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
    use itertools::Itertools;
    use jsonwebtoken::Algorithm;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::CertificateType;
    use crypto::server_keys::generate::Ca;
    use crypto::EcdsaKey;
    use jwt::jwk::jwk_from_p256;
    use jwt::EcdsaDecodingKey;
    use mdoc::holder::Mdoc;
    use sd_jwt::builder::SdJwtBuilder;
    use sd_jwt::hasher::Sha256Hasher;
    use sd_jwt::key_binding_jwt_claims::RequiredKeyBinding;
    use sd_jwt::sd_jwt::SdJwtPresentation;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attributes::Attribute;
    use crate::attributes::AttributeValue;
    use crate::credential_payload::CredentialPayloadError;

    use super::CredentialPayload;
    use super::PreviewableCredentialPayload;

    #[test]
    fn test_serialize_deserialize_and_validate() {
        let confirmation_key = jwk_from_p256(SigningKey::random(&mut OsRng).verifying_key()).unwrap();

        let payload = CredentialPayload {
            issued_at: Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap().into(),
            confirmation_key: RequiredKeyBinding::Jwk(confirmation_key.clone()),
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
        CredentialPayload::validate(&json, &metadata).expect("CredentialPayload should be valid");
    }

    #[test]
    fn test_from_mdoc() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::pid_example());

        let payload = CredentialPayload::from_mdoc(mdoc, &metadata)
            .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload.previewable_payload.attributes.into_values().collect_vec(),
            vec![
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
                Attribute::Single(AttributeValue::Text("999999999".to_string()))
            ]
        );
    }

    #[test]
    fn test_from_mdoc_parts() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::pid_example());

        let payload =
            CredentialPayload::from_mdoc_parts(mdoc.issuer_signed.into_entries_by_namespace(), mdoc.mso, &metadata)
                .expect("creating and validating CredentialPayload from Mdoc should succeed");

        assert_eq!(
            payload.previewable_payload.attributes.into_values().collect_vec(),
            vec![
                Attribute::Single(AttributeValue::Text("De Bruijn".to_string())),
                Attribute::Single(AttributeValue::Text("Willeke Liselotte".to_string())),
                Attribute::Single(AttributeValue::Text("999999999".to_string()))
            ]
        );
    }

    #[test]
    fn test_from_mdoc_parts_invalid() {
        let mdoc = Mdoc::new_mock().now_or_never().unwrap();
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            "urn:eudi:pid:nl:1",
            &[
                ("family_name", JsonSchemaPropertyType::Number, None),
                ("bsn", JsonSchemaPropertyType::String, None),
                ("given_name", JsonSchemaPropertyType::String, None),
            ],
        ));

        let error =
            CredentialPayload::from_mdoc_parts(mdoc.issuer_signed.into_entries_by_namespace(), mdoc.mso, &metadata)
                .expect_err("wrong family_name type should fail validation");

        assert_matches!(error, CredentialPayloadError::MetadataValidation(_));
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
        )
        .expect_err("wrong family_name type should fail validation");

        assert_matches!(error, CredentialPayloadError::MetadataValidation(_));
    }

    #[test]
    fn test_from_sd_jwt() {
        let holder_key = SigningKey::random(&mut OsRng);
        let confirmation_key = jwk_from_p256(holder_key.verifying_key()).unwrap();

        let issuer_key = SigningKey::random(&mut OsRng);

        let claims = json!({
            "vct": "com.example.pid",
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
                &issuer_key,
                String::from("x5c").into_bytes(),
                holder_key.verifying_key(),
            )
            .now_or_never()
            .unwrap()
            .unwrap();

        let metadata =
            NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::credential_payload_sd_jwt_metadata());
        let payload = CredentialPayload::from_sd_jwt(sd_jwt.clone(), &metadata)
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
