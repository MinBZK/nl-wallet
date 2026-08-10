use std::time::Duration;

use attestation_types::credential_format::Format;
use derive_more::Constructor;
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

pub mod draft {
    use std::fmt;
    use std::fmt::Display;
    use std::fmt::Formatter;

    use attestation_types::credential_format::Format;
    use jwt::UnverifiedJwt;
    use jwt::headers::HeaderWithJwk;
    use jwt::pop::JwtPopClaims;
    use serde::Deserialize;
    use serde::Serialize;
    use serde_with::skip_serializing_none;
    use utils::spec::SpecOptional;
    use utils::vec_at_least::VecNonEmpty;

    use super::CredentialResponse;

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
        Jwt {
            jwt: UnverifiedJwt<JwtPopClaims, HeaderWithJwk>,
        },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CredentialResponses {
        pub credential_responses: Vec<CredentialResponse>,
    }
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

    use super::CredentialResponse;
    use super::Credentials;

    #[test]
    fn test_deferred_credential_response_serialization() {
        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-12
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
        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-8
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

        // Source: https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3-8
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
