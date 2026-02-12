use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::value::StringDeserializer;
use serde_with::DeserializeAs;
use serde_with::DurationSeconds;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use http_utils::urls::BaseUrl;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithJwk;
use jwt::pop::JwtPopClaims;
use jwt::wua::WuaDisclosure;
use mdoc::IssuerSigned;
use mdoc::utils::serialization::CborBase64;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wscd::Poa;

use crate::Format;
use crate::token::AuthorizationCode;

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-8.1>.
/// Sent JSON-encoded to `POST /batch_credential`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequests {
    pub credential_requests: VecNonEmpty<CredentialRequest>,
    pub attestations: Option<WuaDisclosure>,
    pub poa: Option<Poa>,
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-7.2>.
/// Sent JSON-encoded to `POST /credential`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequest {
    #[serde(flatten)]
    pub credential_type: SpecOptional<CredentialRequestType>,
    pub proof: Option<CredentialRequestProof>,
    pub attestations: Option<WuaDisclosure>,
    pub poa: Option<Poa>,
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
    pub fn from_format(format: Format, attestation_type: String) -> Option<Self> {
        match format {
            Format::MsoMdoc => Some(CredentialRequestType::MsoMdoc {
                doctype: attestation_type,
            }),
            Format::SdJwt => Some(CredentialRequestType::SdJwt { vct: attestation_type }),
            _ => None,
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

/// A Credential Response, see: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3>.
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialResponse {
    Immediate {
        // TODO (PVW-5554): Actually transport more than one credential in this field
        //                  by implementing batch issuance according to OpenID4VCI 1.0.
        credentials: VecNonEmpty<Credential>,
        notification_id: Option<String>,
    },
    Deferred {
        transaction_id: String,
        #[serde_as(as = "DurationSeconds<u64>")]
        interval: Duration,
    },
}

impl CredentialResponse {
    pub fn new_immediate(credential: Credential) -> Self {
        Self::Immediate {
            credentials: vec_nonempty![credential],
            notification_id: None,
        }
    }

    // TODO (PVW-5554): Replace this with into_immediate_credential().
    pub fn into_immediate_credential(self) -> Option<Credential> {
        match self {
            Self::Immediate { credentials, .. } => Some(credentials.into_first()),
            Self::Deferred { .. } => None,
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Credential {
    MsoMdoc {
        #[serde_as(as = "Box<CborBase64>")]
        credential: Box<IssuerSigned>,
    },
    SdJwt {
        credential: UnverifiedSdJwt,
    },
}

/// Manual implementation of [`Deserialize`] for [`Credential`] is necessary, in order to help `serde`
/// discern between the two enum variants without attempting to do a full Base64 and CBOR decode.
impl<'de> Deserialize<'de> for Credential {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StringCredential {
            credential: String,
        }

        let StringCredential { credential } = StringCredential::deserialize(deserializer)?;

        // Assume the credential is SD-JWT if its string representation contains
        // a tilde character, which does not occur in URL-safe Base64.
        let deserialized_credential = if credential.contains('~') {
            let sd_jwt = UnverifiedSdJwt::deserialize(StringDeserializer::new(credential))?;

            Self::SdJwt { credential: sd_jwt }
        } else {
            let issuer_signed = CborBase64::deserialize_as(StringDeserializer::new(credential))?;

            Self::MsoMdoc {
                credential: issuer_signed,
            }
        };

        Ok(deserialized_credential)
    }
}

impl Credential {
    pub fn new_mdoc(issuer_signed: IssuerSigned) -> Self {
        Self::MsoMdoc {
            credential: Box::new(issuer_signed),
        }
    }

    pub fn new_sd_jwt(sd_jwt: UnverifiedSdJwt) -> Self {
        Self::SdJwt { credential: sd_jwt }
    }

    pub fn format(&self) -> Format {
        match self {
            Self::MsoMdoc { .. } => Format::MsoMdoc,
            Self::SdJwt { .. } => Format::SdJwt,
        }
    }
}

pub const OPENID4VCI_VC_POP_JWT_TYPE: &str = "openid4vci-proof+jwt";
pub const OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME: &str = "openid-credential-offer";

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOffer {
    pub credential_issuer: BaseUrl,
    pub credential_configuration_ids: Vec<String>,
    pub grants: Option<Grants>,
}

/// OpenID4VCI protocol message containing the credential offer.
/// The Credential Offer is passed as a single URI-encoded parameter containing a JSON-encoded value.
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-offer>
/// Note: the spec says that this may contain a `credential_offer_uri` instead of a `credential_offer`, but we don't
/// support that (yet).
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialOfferContainer {
    #[serde_as(as = "JsonString")]
    pub credential_offer: CredentialOffer,
}

/// Grants for a Verifiable Credential.
/// May contain either or both. If it contains both, it is up to the wallet which one it uses.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Grants {
    Both {
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
        authorization_code: GrantAuthorizationCode,
    },
    AuthorizationCode {
        authorization_code: GrantAuthorizationCode,
    },
    PreAuthorizedCode {
        #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
        pre_authorized_code: GrantPreAuthorizedCode,
    },
}

impl Grants {
    pub fn authorization_code(&self) -> Option<AuthorizationCode> {
        match self {
            Grants::Both {
                pre_authorized_code, ..
            } => Some(pre_authorized_code.pre_authorized_code.clone()),
            Grants::PreAuthorizedCode { pre_authorized_code } => Some(pre_authorized_code.pre_authorized_code.clone()),
            Grants::AuthorizationCode { .. } => None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantAuthorizationCode {
    pub issuer_state: Option<String>,
    pub authorization_server: Option<BaseUrl>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrantPreAuthorizedCode {
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: AuthorizationCode,
    pub tx_code: Option<PreAuthTransactionCode>,
    pub authorization_server: Option<BaseUrl>,
}

impl GrantPreAuthorizedCode {
    pub fn new(pre_authorized_code: AuthorizationCode) -> Self {
        Self {
            pre_authorized_code,
            tx_code: None,
            authorization_server: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreAuthTransactionCode {
    pub input_mode: Option<String>,
    pub length: Option<u64>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use assert_matches::assert_matches;
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use serde_json::json;

    use mdoc::DeviceResponse;
    use mdoc::examples::Example;
    use mdoc::utils::serialization::cbor_serialize;
    use sd_jwt::examples::SD_JWT_VC;

    use crate::Format;

    use super::Credential;
    use super::CredentialResponse;
    use super::Grants;

    #[test]
    fn test_grants_serialization() {
        let json = json!({
            "authorization_code": { "issuer_state": "foo" },
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        assert_matches!(serde_json::from_value::<Grants>(json).unwrap(), Grants::Both { .. });

        let json = json!({
            "urn:ietf:params:oauth:grant-type:pre-authorized_code": { "pre-authorized_code": "bar" }
        });
        assert_matches!(
            serde_json::from_value::<Grants>(json).unwrap(),
            Grants::PreAuthorizedCode { .. }
        );

        let json = json!({
            "authorization_code": { "issuer_state": "foo" }
        });
        assert_matches!(
            serde_json::from_value::<Grants>(json).unwrap(),
            Grants::AuthorizationCode { .. }
        );
    }

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
            CredentialResponse::Immediate { credentials, notification_id: None } if credentials.len().get() == 1
        );

        let credential = response.clone().into_immediate_credential().unwrap();
        assert_eq!(credential.format(), Format::SdJwt);
        assert_matches!(credential, Credential::SdJwt { .. });

        let output_json = serde_json::to_value(response).expect("SD-JWT credential response should serialize to JSON");

        assert_eq!(json, output_json);
    }

    #[test]
    fn test_mdoc_credential_response_serialization() {
        let device_response = DeviceResponse::example();
        let issuer_signed = &device_response
            .documents
            .as_ref()
            .unwrap()
            .first()
            .unwrap()
            .issuer_signed;
        let credential = BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(issuer_signed).unwrap());

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
                credentials, notification_id: Some(notification_id)
            } if credentials.len().get() == 1 && notification_id == "3fwe98js"
        );

        let credential = response.clone().into_immediate_credential().unwrap();
        assert_eq!(credential.format(), Format::MsoMdoc);
        assert_matches!(credential, Credential::MsoMdoc { .. });

        let output_json = serde_json::to_value(response).expect("mdoc credential response should serialize to JSON");

        assert_eq!(json, output_json);
    }
}
