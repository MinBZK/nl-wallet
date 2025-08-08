use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use serde::Deserialize;
use serde::Serialize;
use serde_with::TryFromInto;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use http_utils::urls::BaseUrl;
use jwt::Jwt;
use jwt::pop::JwtPopClaims;
use jwt::wte::WteDisclosure;
use mdoc::IssuerSigned;
use mdoc::utils::serialization::CborBase64;
use utils::spec::SpecOptional;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;

use crate::Format;
use crate::token::AuthorizationCode;

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#section-8.1>.
/// Sent JSON-encoded to `POST /batch_credential`.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialRequests {
    pub credential_requests: VecNonEmpty<CredentialRequest>,
    pub attestations: Option<WteDisclosure>,
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
    pub attestations: Option<WteDisclosure>,
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
    Jwt { jwt: Jwt<JwtPopClaims> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialResponses {
    pub credential_responses: Vec<CredentialResponse>,
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-credential-response>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialResponse {
    MsoMdoc { credential: Box<CborBase64<IssuerSigned>> },
    SdJwt { credential: String },
}

impl CredentialResponse {
    pub fn matches_format(&self, format: Format) -> bool {
        match &self {
            CredentialResponse::MsoMdoc { .. } => format == Format::MsoMdoc,
            CredentialResponse::SdJwt { .. } => format == Format::SdJwt,
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
    #[serde_as(as = "TryFromInto<String>")]
    pub credential_offer: CredentialOffer,
}

impl TryFrom<String> for CredentialOffer {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

impl TryInto<String> for CredentialOffer {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
    }
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
    use assert_matches::assert_matches;
    use serde_json::json;

    use crate::credential::Grants;

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
}
