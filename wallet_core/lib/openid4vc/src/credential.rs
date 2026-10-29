use std::collections::HashMap;
use std::time::Duration;

use attestation_types::credential_format::Format;
use derive_more::Constructor;
use jwe::algorithm::CompressionAlgorithm;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::ExpectedEncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::encryption::JweEncrypter;
use jwe::error::JweJsonDecryptionError;
use jwe::error::JweJsonEncryptionError;
use jwk_simple::Key;
use mdoc::IssuerSigned;
use mdoc::utils::serialization::CborBase64;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::IgnoredAny;
use serde::de::value::StringDeserializer;
use serde_with::DeserializeAs;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use wscd::payload::key_attestation::KeyAttestation;

use crate::authorization_details::CredentialId;
use crate::jwe::JweCompressionAlgorithm;
use crate::jwe::JweEncryptionAlgorithm;
use crate::metadata::issuer_metadata::CredentialConfigurationId;

/// A request sent to the issuer's Credential Endpoint.
///
/// See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRequest {
    /// Either a Credential or Credential Configuration Identifier, depending on whether the Token Response contained
    /// the `authorization_details` field.
    #[serde(flatten)]
    pub identifier: CredentialRequestIdentifier,

    /// Object providing one or more proof of possessions of the cryptographic key material to which the issued
    /// Credential instances will be bound to. The `proofs` parameter contains exactly one parameter named as the proof
    /// type, the value set for this parameter is a non-empty array containing parameters as defined by the
    /// corresponding proof type.
    pub proofs: Option<CredentialRequestProofs>,

    /// Object containing information for encrypting the Credential Response. If this request element is not present,
    /// the corresponding credential response returned is not encrypted.
    ///
    /// TODO (PVW-5538): Implement credential request and response encryption.
    pub credential_response_encryption: Option<CredentialResponseEncryption>,
}

impl CredentialRequest {
    pub fn new(identifier: CredentialRequestIdentifier, key_attestation: KeyAttestation) -> Self {
        Self {
            identifier,
            proofs: Some(CredentialRequestProofs::Attestation([key_attestation])),
            credential_response_encryption: None,
        }
    }

    pub fn decrypt(jwe: &str, decrypter: &JweDecrypter) -> Result<Self, JweJsonDecryptionError> {
        decrypter.decrypt_json(jwe, ExpectedEncryptionAlgorithm::Any)
    }

    pub fn encrypt(
        &self,
        encrypter: &JweEncrypter,
        encryption_algorithm: EncryptionAlgorithm,
        use_compression: Option<CompressionAlgorithm>,
    ) -> Result<String, JweJsonEncryptionError> {
        encrypter.encrypt_json(self, encryption_algorithm, None, None, use_compression)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRequestIdentifier {
    /// REQUIRED when an Authorization Details of type `openid_credential` was returned from the Token Response. It MUST
    /// NOT be used otherwise. A string that identifies a Credential Dataset that is requested for issuance.
    CredentialIdentifier(CredentialId),

    /// REQUIRED if a credential_identifiers parameter was not returned from the Token Response as part of the
    /// `authorization_details` parameter. It MUST NOT be used otherwise. String that uniquely identifies one of the
    /// keys in the name/value pairs stored in the `credential_configurations_supported` Credential Issuer metadata. The
    /// corresponding object in the `credential_configurations_supported` map MUST contain one of the value(s) used in
    /// the scope parameter in the Authorization Request.
    CredentialConfigurationId(CredentialConfigurationId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRequestProofs {
    Attestation([KeyAttestation; 1]),

    // Allow parsing of proof types not supported by this implementation, such as `jwt` and `di_vp`.
    #[serde(untagged, skip_serializing)]
    Other(HashMap<String, IgnoredAny>),
}

/// Object containing information for encrypting the Credential Response.
///
/// See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2-2.4.1>
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialResponseEncryption {
    /// Object containing a single public key as a JWK used for encrypting the Credential Response.
    pub jwk: Key,

    /// JWE (RFC7516) enc algorithm (RFC7518) for encrypting Credential Responses.
    pub enc: JweEncryptionAlgorithm,

    /// JWE (RFC7516) zip algorithm (RFC7518) for compressing Credential Responses prior to encryption. If absent then
    /// compression MUST not be used.
    pub zip: Option<JweCompressionAlgorithm>,
}

/// A Credential Response, see: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3>.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialResponse {
    Immediate {
        credentials: Credentials,
        notification_id: Option<String>,
    },
    Deferred {
        transaction_id: String,
        #[serde_as(as = "DurationSeconds<u64>")]
        interval: Duration,
    },
}

impl CredentialResponse {
    pub fn new_immediate(credentials: Credentials) -> Self {
        Self::Immediate {
            credentials,
            notification_id: None,
        }
    }

    pub fn into_immediate_credentials(self) -> Option<Credentials> {
        match self {
            Self::Immediate { credentials, .. } => Some(credentials),
            Self::Deferred { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Credentials {
    MsoMdoc(VecNonEmpty<MdocCredential>),
    SdJwt(VecNonEmpty<SdJwtCredential>),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Constructor)]
pub struct MdocCredential {
    #[serde_as(as = "CborBase64")]
    pub credential: IssuerSigned,
}

#[derive(Debug, Clone, Serialize, Constructor)]
pub struct SdJwtCredential {
    pub credential: UnverifiedSdJwt,
}

/// Manual implementation of [`Deserialize`] for [`Credentials`] is necessary, in order to help `serde` discern between
/// the two enum variants without attempting to do a full Base64 and CBOR decode.
impl<'de> Deserialize<'de> for Credentials {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StringCredential {
            credential: String,
        }

        let string_credentials = VecNonEmpty::<StringCredential>::deserialize(deserializer)?;
        let StringCredential {
            credential: first_credential,
        } = string_credentials.first();

        // Assume the credentials are all SD-JWT if the first credential's string representation contains a tilde
        // character, which does not occur in URL-safe Base64.
        let deserialized_credentials = if first_credential.contains('~') {
            let sd_jwt_credentials = string_credentials
                .into_nonempty_iter()
                .map(|StringCredential { credential }| {
                    let sd_jwt = UnverifiedSdJwt::deserialize(StringDeserializer::new(credential))?;

                    Ok(SdJwtCredential::new(sd_jwt))
                })
                .collect::<Result<_, _>>()?;

            Self::SdJwt(sd_jwt_credentials)
        } else {
            let mdoc_credentials = string_credentials
                .into_nonempty_iter()
                .map(|StringCredential { credential }| {
                    let issuer_signed = CborBase64::deserialize_as(StringDeserializer::new(credential))?;

                    Ok(MdocCredential::new(issuer_signed))
                })
                .collect::<Result<_, _>>()?;

            Self::MsoMdoc(mdoc_credentials)
        };

        Ok(deserialized_credentials)
    }
}

impl Credentials {
    pub fn format(&self) -> Format {
        match self {
            Self::MsoMdoc { .. } => Format::MsoMdoc,
            Self::SdJwt { .. } => Format::SdJwt,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::time::Duration;

    use attestation_types::credential_format::Format;
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use mdoc::DeviceResponse;
    use mdoc::examples::Example;
    use mdoc::utils::serialization::cbor_serialize;
    use sd_jwt::examples::SD_JWT_VC;
    use serde_json::json;

    use super::CredentialRequest;
    use super::CredentialRequestIdentifier;
    use super::CredentialRequestProofs;
    use super::CredentialResponse;
    use super::Credentials;

    // Key Attestation in JWT format, containing the example claims from
    // <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-D.1-6> and a dummy signature.
    const KEY_ATTESTATION_JWT: &str = "eyJ0eXAiOiJrZXktYXR0ZXN0YXRpb24rand0IiwiYWxnIjoiRVMyNTYiLCJ4NWMiOlsiTUlJQiJdfQ.\
        eyJpc3MiOiJodHRwczovL3dhbGxldC1wcm92aWRlci5leGFtcGxlLmNvbSIsImlhdCI6MTUxNjI0NzAyMiwiZXhwIjoxNTQxNDkzNzI0LCJrZX\
        lfc3RvcmFnZSI6WyJpc29fMTgwNDVfbW9kZXJhdGUiXSwidXNlcl9hdXRoZW50aWNhdGlvbiI6WyJpc29fMTgwNDVfbW9kZXJhdGUiXSwiYXR0\
        ZXN0ZWRfa2V5cyI6W3sia3R5IjoiRUMiLCJjcnYiOiJQLTI1NiIsIngiOiJUQ0FFUjE5WnZ1M09IRjRqNFc0dmZTVm9ISVAxSUxpbERsczd2Q2\
        VHZW1jIiwieSI6Ilp4amlXV2JaTVFHSFZXS1ZRNGhiU0lpcnNWZnVlY0NFNnQ0alQ5RjJIWlEifV0sIm5vbmNlIjoiTGFyUkdTYm1VUFl0UllP\
        NkJRNHluOCJ9.c2lnbmF0dXJl";

    fn assert_key_attestation_proof(credential_request: &CredentialRequest) {
        let Some(CredentialRequestProofs::Attestation([key_attestation])) = &credential_request.proofs else {
            panic!("proofs in CredentialRequest should be of the \"attestation\" type");
        };
        assert_eq!(key_attestation.as_ref().serialization(), KEY_ATTESTATION_JWT);
    }

    #[test]
    fn test_credential_request_serialization_config_id_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2-12>,
        // modified using: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.3-5>
        let example_json = json!({
            "credential_configuration_id": "org.iso.18013.5.1.mDL",
            "proofs": {
                "attestation": [KEY_ATTESTATION_JWT]
            }
        });

        let credential_request = serde_json::from_value::<CredentialRequest>(example_json.clone())
            .expect("deserializing CredentialRequest should succeed");

        let CredentialRequestIdentifier::CredentialConfigurationId(config_id) = &credential_request.identifier else {
            panic!("identifier in CredentialRequest should be Credential Configuration ID");
        };
        assert_eq!(config_id.as_ref(), "org.iso.18013.5.1.mDL");

        assert_key_attestation_proof(&credential_request);

        let json = serde_json::to_value(credential_request).expect("serializing CredentialRequest should succeed");

        assert_eq!(example_json, json);
    }

    #[test]
    fn test_credential_request_serialization_credential_id_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2-14>,
        // modified using: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.3-5>
        let example_json = json!({
            "credential_identifier": "CivilEngineeringDegree-2023",
            "proofs": {
                "attestation": [KEY_ATTESTATION_JWT]
            }
        });

        let credential_request = serde_json::from_value::<CredentialRequest>(example_json.clone())
            .expect("deserializing CredentialRequest should succeed");

        let CredentialRequestIdentifier::CredentialIdentifier(credential_id) = &credential_request.identifier else {
            panic!("identifier in CredentialRequest should be Credential ID");
        };
        assert_eq!(credential_id.as_ref(), "CivilEngineeringDegree-2023");

        assert_key_attestation_proof(&credential_request);

        let json = serde_json::to_value(credential_request).expect("serializing CredentialRequest should succeed");

        assert_eq!(example_json, json);
    }

    #[test]
    fn test_deferred_credential_response_serialization() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-12>
        let json = json!({
            "transaction_id": "8xLOxBtZp8",
            "interval" : 3600
        });

        let response = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect("deferred credential response JSON should parse correctly");

        assert_matches!(
            &response,
            CredentialResponse::Deferred {
                transaction_id,
                interval
            } if transaction_id == "8xLOxBtZp8" && *interval == Duration::from_hours(1)
        );

        let output_json =
            serde_json::to_value(response).expect("deferred credential response should serialize to JSON");

        assert_eq!(json, output_json);
    }

    #[test]
    fn test_sd_jwt_credential_response_serialization() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-8>
        let json = json!({
            "credentials": [
                {
                    "credential": SD_JWT_VC
                }
            ]
        });

        let response = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect("SD-JWT credential response JSON should parse correctly");

        assert_matches!(
            &response,
            CredentialResponse::Immediate {
                notification_id: None,
                ..
            }
        );

        let credentials = response.clone().into_immediate_credentials().unwrap();
        assert_eq!(credentials.format(), Format::SdJwt);
        assert_matches!(credentials, Credentials::SdJwt(sd_jwt_credentials) if sd_jwt_credentials.len().get() == 1);

        let output_json = serde_json::to_value(response).expect("SD-JWT credential response should serialize to JSON");

        assert_eq!(json, output_json);
    }

    #[test]
    fn test_mdoc_credential_response_serialization() {
        let device_response = DeviceResponse::example();
        let issuer_signed = device_response.documents.unwrap().into_first().issuer_signed;
        let credential = BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap());

        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-8>
        let json = json!({
            "credentials": [
                {
                    "credential": credential,
                }
            ],
            "notification_id": "3fwe98js"
        });

        let response = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect("mdoc credential response JSON should parse correctly");

        assert_matches!(
            &response,
            CredentialResponse::Immediate {
                notification_id: Some(notification_id),
                ..
            } if notification_id == "3fwe98js"
        );

        let credentials = response.clone().into_immediate_credentials().unwrap();
        assert_eq!(credentials.format(), Format::MsoMdoc);
        assert_matches!(credentials, Credentials::MsoMdoc(mdoc_credentials) if mdoc_credentials.len().get() == 1);

        let output_json = serde_json::to_value(response).expect("mdoc credential response should serialize to JSON");

        assert_eq!(json, output_json);
    }

    #[test]
    fn test_mixed_credential_response_deserialization_error() {
        let device_response = DeviceResponse::example();
        let issuer_signed = device_response.documents.unwrap().into_first().issuer_signed;
        let credential = BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap());

        let json = json!({
            "credentials": [
                {
                    "credential": credential,
                },
                {
                    "credential": SD_JWT_VC
                }
            ],
            "notification_id": "3fwe98js"
        });

        let _ = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect_err("mixed credential response JSON should not parse correctly");
    }

    #[test]
    fn test_multi_sd_jwt_credential_response_deserialization() {
        let json = json!({
            "credentials": [
                {
                    "credential": SD_JWT_VC
                },
                {
                    "credential": SD_JWT_VC
                },
                {
                    "credential": SD_JWT_VC
                }
            ]
        });

        let response = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect("SD-JWT credential response JSON should parse correctly");

        let credentials = response
            .clone()
            .into_immediate_credentials()
            .expect("SD-JWT credential response should be immediate");
        assert_matches!(credentials, Credentials::SdJwt(sd_jwt_credentials) if sd_jwt_credentials.len().get() == 3);
    }

    #[test]
    fn test_multi_mdoc_credential_response_deserialization() {
        let device_response = DeviceResponse::example();
        let issuer_signed = device_response.documents.unwrap().into_first().issuer_signed;
        let credential = BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap());

        let json = json!({
            "credentials": [
                {
                    "credential": credential
                },
                {
                    "credential": credential
                },
                {
                    "credential": credential
                }
            ]
        });

        let response = serde_json::from_value::<CredentialResponse>(json.clone())
            .expect("mdoc credential response JSON should parse correctly");

        let credentials = response
            .clone()
            .into_immediate_credentials()
            .expect("mdoc credential response should be immediate");
        assert_matches!(credentials, Credentials::MsoMdoc(mdoc_credentials) if mdoc_credentials.len().get() == 3);
    }
}
