use std::time::Duration;

use attestation_types::credential_format::Format;
use derive_more::Constructor;
use jwk_simple::Key;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::pop::JwtPopClaims;
use mdoc::IssuerSigned;
use mdoc::utils::serialization::CborBase64;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::value::StringDeserializer;
use serde_with::DeserializeAs;
use serde_with::DurationSeconds;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::jwe::JweCompressionAlgorithm;
use crate::jwe::JweEncryptionAlgorithm;
use crate::metadata::issuer_metadata::CredentialConfigurationId;

pub mod draft {
    use std::fmt;
    use std::fmt::Display;
    use std::fmt::Formatter;

    use attestation_types::credential_format::Format;
    use serde::Deserialize;
    use serde::Serialize;
    use serde_with::skip_serializing_none;
    use utils::spec::SpecOptional;
    use utils::vec_at_least::VecNonEmpty;

    use super::CredentialResponse;
    use super::UnverifiedJwtProof;

    /// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-8.1>.
    /// Sent JSON-encoded to `POST /batch_credential`.
    #[skip_serializing_none]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CredentialRequests {
        pub credential_requests: VecNonEmpty<CredentialRequest>,
    }

    /// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-7.2>.
    /// Sent JSON-encoded to `POST /credential`.
    #[skip_serializing_none]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CredentialRequest {
        #[serde(flatten)]
        pub credential_type: SpecOptional<CredentialRequestType>,
        pub proof: Option<CredentialRequestProof>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(tag = "format", rename_all = "snake_case")]
    pub enum CredentialRequestType {
        MsoMdoc {
            doctype: String,
        },

        #[serde(rename = "dc+sd-jwt")]
        SdJwt {
            vct: String,
        },
    }

    impl CredentialRequestType {
        pub fn format(&self) -> Format {
            match self {
                CredentialRequestType::MsoMdoc { .. } => Format::MsoMdoc,
                CredentialRequestType::SdJwt { .. } => Format::SdJwt,
            }
        }
    }

    impl Display for CredentialRequestType {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                CredentialRequestType::MsoMdoc { doctype } => write!(f, "MsoMdoc({doctype})"),
                CredentialRequestType::SdJwt { vct } => write!(f, "SdJwt({vct})"),
            }
        }
    }

    impl CredentialRequestType {
        pub fn from_format(format: Format, attestation_type: String) -> Self {
            match format {
                Format::MsoMdoc => CredentialRequestType::MsoMdoc {
                    doctype: attestation_type,
                },
                Format::SdJwt => CredentialRequestType::SdJwt { vct: attestation_type },
            }
        }
    }

    /// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-endpoint>
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(tag = "proof_type", rename_all = "snake_case")]
    pub enum CredentialRequestProof {
        Jwt { jwt: UnverifiedJwtProof },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CredentialResponses {
        pub credential_responses: Vec<CredentialResponse>,
    }
}

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
    pub fn new_credential_id(credential_id: String, proofs: VecNonEmpty<UnverifiedJwtProof>) -> Self {
        Self {
            identifier: CredentialRequestIdentifier::CredentialIdentifier(credential_id),
            proofs: Some(CredentialRequestProofs::Jwt(proofs)),
            credential_response_encryption: None,
        }
    }

    pub fn new_config_id(config_id: CredentialConfigurationId, proofs: VecNonEmpty<UnverifiedJwtProof>) -> Self {
        Self {
            identifier: CredentialRequestIdentifier::CredentialConfigurationId(config_id),
            proofs: Some(CredentialRequestProofs::Jwt(proofs)),
            credential_response_encryption: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRequestIdentifier {
    /// REQUIRED when an Authorization Details of type `openid_credential` was returned from the Token Response. It MUST
    /// NOT be used otherwise. A string that identifies a Credential Dataset that is requested for issuance.
    CredentialIdentifier(String),

    /// REQUIRED if a credential_identifiers parameter was not returned from the Token Response as part of the
    /// `authorization_details` parameter. It MUST NOT be used otherwise. String that uniquely identifies one of the
    /// keys in the name/value pairs stored in the `credential_configurations_supported` Credential Issuer metadata. The
    /// corresponding object in the `credential_configurations_supported` map MUST contain one of the value(s) used in
    /// the scope parameter in the Authorization Request.
    CredentialConfigurationId(CredentialConfigurationId),
}

pub type UnverifiedJwtProof = UnverifiedJwt<JwtPopClaims, HeaderWithJwk>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialRequestProofs {
    // TODO (PVW-5548): Implement `attestation` proof type and update `jwt` proof type to OpenID4VCI 1.0.
    Jwt(VecNonEmpty<UnverifiedJwtProof>),
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
        // TODO (PVW-5554): Actually transport more than one credential in this field
        //                  by implementing batch issuance according to OpenID4VCI 1.0.
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
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct MdocCredential {
    #[serde_as(as = "CborBase64")]
    pub credential: IssuerSigned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
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
    pub fn new_single_mdoc(issuer_signed: IssuerSigned) -> Self {
        Self::MsoMdoc(vec_nonempty![MdocCredential::new(issuer_signed)])
    }

    pub fn new_single_sd_jwt(sd_jwt: UnverifiedSdJwt) -> Self {
        Self::SdJwt(vec_nonempty![SdJwtCredential::new(sd_jwt)])
    }

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

    #[test]
    fn test_credential_request_serialization_config_id_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2-12>,
        // modified using: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1-6>
        let example_json = json!({
            "credential_configuration_id": "org.iso.18013.5.1.mDL",
            "proofs": {
                "jwt": [
                        "eyJ0eXAiOiJvcGVuaWQ0dmNpLXByb29mK2p3dCIsImFsZyI6IkVTMjU2Iiwiand
                         rIjp7Imt0eSI6IkVDIiwiY3J2IjoiUC0yNTYiLCJ4IjoiblVXQW9BdjNYWml0aDh
                         FN2kxOU9kYXhPTFlGT3dNLVoyRXVNMDJUaXJUNCIsInkiOiJIc2tIVThCalVpMVU
                         5WHFpN1N3bWo4Z3dBS18weGtjRGpFV183MVNvc0VZIn19.eyJhdWQiOiJodHRwcz
                         ovL2NyZWRlbnRpYWwtaXNzdWVyLmV4YW1wbGUuY29tIiwiaWF0IjoxNzAxOTYwND
                         Q0LCJub25jZSI6IkxhclJHU2JtVVBZdFJZTzZCUTR5bjgifQ.-a3EDsxClUB4O3L
                         eDD5DVGEnNMT01FCQW4P6-2-BNBqc_Zxf0Qw4CWayLEpqkAomlkLb9zioZoipdP-
                         jvh1WlA"
                    ]
                }
        });

        let credential_request = serde_json::from_value::<CredentialRequest>(example_json.clone())
            .expect("deserializing CredentialRequest should succeed");

        let CredentialRequestIdentifier::CredentialConfigurationId(config_id) = &credential_request.identifier else {
            panic!("identifier in CredentialRequest should be Credential Configuration ID");
        };
        assert_eq!(config_id.as_ref(), "org.iso.18013.5.1.mDL");

        let proof_count = credential_request
            .proofs
            .as_ref()
            .map(|CredentialRequestProofs::Jwt(jwts)| jwts.len().get())
            .unwrap_or_default();
        assert_eq!(proof_count, 1);

        let json = serde_json::to_value(credential_request).expect("serializing CredentialRequest should succeed");

        assert_eq!(example_json, json);
    }

    #[test]
    fn test_credential_request_serialization_credential_id_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.2-14>,
        // modified using: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#appendix-F.1-6>
        let example_json = json!({
            "credential_identifier": "CivilEngineeringDegree-2023",
            "proofs": {
            "jwt": [
                        "eyJ0eXAiOiJvcGVuaWQ0dmNpLXByb29mK2p3dCIsImFsZyI6IkVTMjU2Iiwiand
                         rIjp7Imt0eSI6IkVDIiwiY3J2IjoiUC0yNTYiLCJ4IjoiblVXQW9BdjNYWml0aDh
                         FN2kxOU9kYXhPTFlGT3dNLVoyRXVNMDJUaXJUNCIsInkiOiJIc2tIVThCalVpMVU
                         5WHFpN1N3bWo4Z3dBS18weGtjRGpFV183MVNvc0VZIn19.eyJhdWQiOiJodHRwcz
                         ovL2NyZWRlbnRpYWwtaXNzdWVyLmV4YW1wbGUuY29tIiwiaWF0IjoxNzAxOTYwND
                         Q0LCJub25jZSI6IkxhclJHU2JtVVBZdFJZTzZCUTR5bjgifQ.-a3EDsxClUB4O3L
                         eDD5DVGEnNMT01FCQW4P6-2-BNBqc_Zxf0Qw4CWayLEpqkAomlkLb9zioZoipdP-
                         jvh1WlA",
                        "eyJ0eXAiOiJvcGVuaWQ0dmNpLXByb29mK2p3dCIsImFsZyI6IkVTMjU2Iiwiand
                         rIjp7Imt0eSI6IkVDIiwiY3J2IjoiUC0yNTYiLCJ4IjoiblVXQW9BdjNYWml0aDh
                         FN2kxOU9kYXhPTFlGT3dNLVoyRXVNMDJUaXJUNCIsInkiOiJIc2tIVThCalVpMVU
                         5WHFpN1N3bWo4Z3dBS18weGtjRGpFV183MVNvc0VZIn19.eyJhdWQiOiJodHRwcz
                         ovL2NyZWRlbnRpYWwtaXNzdWVyLmV4YW1wbGUuY29tIiwiaWF0IjoxNzAxOTYwND
                         Q0LCJub25jZSI6IkxhclJHU2JtVVBZdFJZTzZCUTR5bjgifQ.-a3EDsxClUB4O3L
                         eDD5DVGEnNMT01FCQW4P6-2-BNBqc_Zxf0Qw4CWayLEpqkAomlkLb9zioZoipdP-
                         jvh1WlA"
                ]
            }
        });

        let credential_request = serde_json::from_value::<CredentialRequest>(example_json.clone())
            .expect("deserializing CredentialRequest should succeed");

        let CredentialRequestIdentifier::CredentialIdentifier(credential_id) = &credential_request.identifier else {
            panic!("identifier in CredentialRequest should be Credential ID");
        };
        assert_eq!(credential_id, "CivilEngineeringDegree-2023");

        let proof_count = credential_request
            .proofs
            .as_ref()
            .map(|CredentialRequestProofs::Jwt(jwts)| jwts.len().get())
            .unwrap_or_default();
        assert_eq!(proof_count, 2);

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
