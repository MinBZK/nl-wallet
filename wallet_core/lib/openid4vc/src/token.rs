use std::collections::HashSet;
use std::num::NonZeroU8;
use std::time::Duration;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::x509::CertificateType;
use attestation_data::x509::CertificateTypeError;
use attestation_types::credential_format::Format;
use crypto::trust_anchor::TrustAnchors;
use crypto::utils::random_string;
use crypto::utils::sha256;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use derive_more::Debug;
use derive_more::From;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::StringWithSeparator;
use serde_with::TryFromInto;
use serde_with::base64::Base64;
use serde_with::formats::SpaceSeparator;
use serde_with::json::JsonString;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization_details::IssuerAuthorizationDetails;
use crate::authorization_details::IssuerAuthorizationDetailsEntries;
use crate::authorization_details::WalletAuthorizationDetails;
use crate::authorization_details::WalletAuthorizationDetailsEntries;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::scope::Scope;
use crate::server_state::SessionToken;

#[derive(Serialize, Deserialize, Debug, Clone, From)]
pub struct AuthorizationCode(String);

impl AsRef<str> for AuthorizationCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, From)]
pub struct AccessToken(String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<AuthorizationCode> for SessionToken {
    fn from(value: AuthorizationCode) -> Self {
        SessionToken::from(value.0)
    }
}

impl AccessToken {
    /// Construct a new random access token, with the specified authorization code appended to it.
    pub(crate) fn new(code: &AuthorizationCode) -> Self {
        Self(random_string(32) + code.as_ref())
    }

    /// Returns the authorization code appended to this access token.
    pub(crate) fn code(&self) -> Option<AuthorizationCode> {
        self.as_ref().get(32..).map(|code| AuthorizationCode(code.to_string()))
    }

    pub(crate) fn sha256(&self) -> Vec<u8> {
        sha256(self.as_ref().as_bytes())
    }
}

/// An OAuth 2.0 Token Request as defined by RFC 6749.
///
/// Sent URL-encoded in request body to POST /token. A DPoP HTTP header may be included.
///
/// See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.1> and
/// <https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3>.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    #[serde(flatten)]
    pub grant_type: TokenRequestGrantType,

    /// OpenID4VCI states: "For the Pre-Authorized Code Grant Type, authentication of the Client is OPTIONAL".
    pub client_id: Option<String>,

    /// MUST be the redirect URI value as passed in the Authorization Request.
    pub redirect_uri: Option<Url>,

    /// Section 3.3 of RFC 6749 states that the client may include a scope value when sending the Token Request to the
    /// token endpoint. Note that, unlike the Authorization Request, we make a semantic distinction between this value
    /// not being included and the scope value set being empty.
    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, Scope>>")]
    pub scope: Option<HashSet<Scope>>,

    /// The PKCE code verifier as defined in RFC 7636.
    pub code_verifier: Option<String>,

    #[serde_as(as = "Option<JsonString<TryFromInto<WalletAuthorizationDetailsEntries>>>")]
    pub authorization_details: Option<WalletAuthorizationDetails>,
}

impl TokenRequest {
    pub fn new_authorization_code(
        authorization_code: AuthorizationCode,
        redirect_uri: Url,
        code_verifier: String,
    ) -> Self {
        Self::new_authorization_code_with_client_id(authorization_code, redirect_uri, code_verifier, None)
    }

    pub fn new_authorization_code_with_client_id(
        authorization_code: AuthorizationCode,
        redirect_uri: Url,
        code_verifier: String,
        client_id: Option<String>,
    ) -> Self {
        Self {
            grant_type: TokenRequestGrantType::AuthorizationCode {
                code: authorization_code,
            },
            client_id,
            redirect_uri: Some(redirect_uri),
            scope: None,
            code_verifier: Some(code_verifier),
            authorization_details: None,
        }
    }

    pub fn new_pre_authorized(pre_authorized_code: AuthorizationCode) -> Self {
        Self {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            client_id: None, // Not required as our implementation sends a WIA which contains the client_id in the sub
            redirect_uri: None,
            scope: None,
            code_verifier: None,
            authorization_details: None,
        }
    }

    /// Retrieve either the authorization code or the pre-authorized code, depending on the authorization grant type.
    pub fn code(&self) -> &AuthorizationCode {
        match &self.grant_type {
            TokenRequestGrantType::AuthorizationCode { code } => code,
            TokenRequestGrantType::PreAuthorizedCode {
                pre_authorized_code, ..
            } => pre_authorized_code,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
#[serde(rename = "snake_case")]
#[serde(tag = "grant_type")]
pub enum TokenRequestGrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode { code: AuthorizationCode },
    #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
    PreAuthorizedCode {
        #[serde(rename = "pre-authorized_code")]
        pre_authorized_code: AuthorizationCode,
        // According to OpenID4VCI, a Token Request containing a pre-authorized code also must contain a `tx_code` if
        // the inciting Credential Offer contains a `tx_code` field itself. However, as the wallet does not support the
        // concept of a transaction code, we do not include it in the data structure here.
        //
        // See: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.1-3.2>
    },
}

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.2>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.1>.
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,

    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, Scope>")]
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub scope: HashSet<Scope>,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub expires_in: Option<Duration>,

    /// REQUIRED when the authorization_details parameter, as defined in Section 5.1.1, is used in either the
    /// Authorization Request or Token Request. OPTIONAL when scope parameter was used to request issuance of a
    /// Credential of a certain Credential Configuration.
    #[serde_as(as = "Option<TryFromInto<IssuerAuthorizationDetailsEntries>>")]
    pub authorization_details: Option<IssuerAuthorizationDetails>,
}

impl TokenResponse {
    pub fn new(access_token: AccessToken) -> Self {
        Self::new_vci(access_token, None)
    }

    pub fn new_vci(access_token: AccessToken, authorization_details: Option<IssuerAuthorizationDetails>) -> Self {
        Self {
            access_token,
            token_type: TokenType::DPoP,
            expires_in: None,
            refresh_token: None,
            scope: HashSet::new(),
            authorization_details,
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialPreview {
    pub config_id: CredentialConfigurationId,

    pub format: Format,

    // TODO (PVW-5634): Use the `batch_credential_issuance` field in the issuer metadata instead.
    pub batch_size: NonZeroU8,

    pub credential_payload: PreviewableCredentialPayload,

    #[serde_as(as = "Base64")]
    pub issuer_certificate: BorrowingCertificate,
}

impl CredentialPreview {
    pub fn verify(&self, trust_anchors: &TrustAnchors) -> Result<(), CredentialPreviewError> {
        // Verify the issuer certificates that the issuer presents for each credential to be issued.
        // NB: this only proves the authenticity of the data inside the certificates (the
        // [`IssuerRegistration`]s), but does not authenticate the issuer that presents them.
        // Anyone that has ever seen these certificates (such as other wallets that received them during
        // issuance) could present them here in the protocol without needing the corresponding
        // issuer private key. This is not a problem, because at the end of the issuance
        // protocol each mdoc is verified against the corresponding certificate in the
        // credential preview, which implicitly authenticates the issuer because only it could
        // have produced an mdoc against that certificate.
        self.issuer_certificate
            .verify(Some(CertificateUsage::Mdl), &[], &TimeGenerator, trust_anchors)?;

        // Verify that the issuer_uri is among the SAN DNS names or URIs in the issuer_certificate
        if !self
            .issuer_certificate
            .san_dns_name_or_uris()?
            .as_ref()
            .contains(&self.credential_payload.issuer)
        {
            return Err(CredentialPreviewError::IssuerUriNotFoundInSan(
                self.credential_payload.issuer.clone(),
                self.issuer_certificate.san_dns_name_or_uris()?,
            ));
        }

        Ok(())
    }

    pub fn issuer_registration(&self) -> Result<IssuerRegistration, CredentialPreviewError> {
        let CertificateType::Mdl(issuer) = CertificateType::from_certificate(&self.issuer_certificate)? else {
            Err(CredentialPreviewError::NoIssuerCertificate)?
        };
        Ok(issuer)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPreviewError {
    #[error("certificate error: {0}")]
    #[category(defer)]
    Certificate(#[from] CertificateError),

    #[error("certificate type error: {0}")]
    #[category(defer)]
    CertificateType(#[from] CertificateTypeError),

    #[error("certificate is not an issuer certificate")]
    #[category(critical)]
    NoIssuerCertificate,

    #[error("issuer URI {0} not found in SAN {1:?}")]
    #[category(pd)]
    IssuerUriNotFoundInSan(HttpsUri, VecNonEmpty<HttpsUri>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    #[default]
    Bearer,
    DPoP,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Duration;

    use itertools::Itertools;
    use serde_json::json;
    use utils::vec_nonempty;

    use super::TokenRequest;
    use super::TokenRequestGrantType;
    use super::TokenResponse;
    use super::TokenType;
    use crate::authorization_details::TypedAuthorizationDetailsEntry;

    #[test]
    fn token_request_serialization() {
        #[rustfmt::skip]
        assert_eq!(
            serde_qs::to_string(&TokenRequest {
                grant_type: TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: "123".to_string().into(),
                },
                client_id: Some("myclient".to_string()),
                redirect_uri: Some("https://example.com".parse().unwrap()),
                scope: None,
                code_verifier: Some("myverifier".to_string()),
                authorization_details: None,
            })
            .unwrap(),
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Apre-authorized_code\
                &pre-authorized_code=123\
                &client_id=myclient\
                &redirect_uri=https%3A%2F%2Fexample.com%2F\
                &code_verifier=myverifier",
        );
    }

    #[test]
    fn token_response_serialization() {
        let token_response = TokenResponse {
            access_token: "access_token".to_string().into(),
            token_type: TokenType::Bearer,
            scope: HashSet::from(["scope1".parse().unwrap(), "scope2".parse().unwrap()]),
            expires_in: None,
            refresh_token: None,
            authorization_details: None,
        };

        let mut json =
            serde_json::to_value(token_response).expect("should be able to serialize TokenResponse to JSON value");

        // Sort scope values, as their order is not deterministic.
        json["scope"] = json
            .get("scope")
            .unwrap()
            .as_str()
            .unwrap()
            .split(' ')
            .sorted()
            .join(" ")
            .into();

        let expected_json = json!({
            "access_token": "access_token",
            "token_type": "Bearer",
            "scope": "scope1 scope2"
        });

        assert_eq!(json, expected_json);
    }

    #[test]
    fn token_response_deserialization() {
        let json = json!({
            "access_token": "token",
            "token_type": "DPoP"
        });

        let token_response = serde_json::from_value::<TokenResponse>(json.clone())
            .expect("should be able to deserialize TokenResponse from JSON value");

        assert_eq!(token_response.access_token.as_ref(), "token");
        assert_eq!(token_response.token_type, TokenType::DPoP);
        assert!(token_response.refresh_token.is_none());
        assert!(token_response.scope.is_empty());
        assert!(token_response.expires_in.is_none());

        let serialized_json =
            serde_json::to_value(token_response).expect("should be able to serialize TokenResponse to JSON value");

        assert_eq!(json, serialized_json);
    }

    #[test]
    fn token_request_deserialize_pre_authorized_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.1.1-3>
        let example = "grant_type=urn:ietf:params:oauth:grant-type:pre-authorized_code&\
                       pre-authorized_code=SplxlOBeZQQYbYS6WxSbIA&tx_code=493536&authorization_details=%5B%7B%22type%\
                       22%3A%20%22openid_credential%22%2C%20%22credential_configuration_id%22%3A%20%\
                       22UniversityDegreeCredential%22%7D%5D";

        let token_request =
            serde_qs::from_str::<TokenRequest>(example).expect("deserializing TokenRequest should succeed");

        let TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } = &token_request.grant_type else {
            panic!("grant type should be pre-authorized");
        };

        assert_eq!(pre_authorized_code.as_ref(), "SplxlOBeZQQYbYS6WxSbIA");

        assert!(token_request.client_id.is_none());
        assert!(token_request.redirect_uri.is_none());
        assert!(token_request.scope.is_none());

        let authorization_details = token_request
            .authorization_details
            .as_ref()
            .expect("authorization_details should be present in Authorization Request");

        let entry = authorization_details
            .as_ref()
            .iter()
            .exactly_one()
            .expect("there should exactly one authorization_details entry");

        assert!(entry.locations.is_none());

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_entry) = &entry.typed_entry else {
            panic!("authorization details entry should be of type openid_credential");
        };

        assert_eq!(
            vci_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
    }

    #[test]
    fn token_response_deserialize_example() {
        // Source: <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.2-6>
        let example_json = json!({
            "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6Ikp..sHQ",
            "token_type": "Bearer",
            "expires_in": 86400,
            "authorization_details": [
                {
                    "type": "openid_credential",
                    "credential_configuration_id": "UniversityDegreeCredential",
                    "credential_identifiers": [
                        "CivilEngineeringDegree-2023",
                        "ElectricalEngineeringDegree-2023"
                    ]
                }
            ]
        });

        let token_response =
            serde_json::from_value::<TokenResponse>(example_json).expect("deserializing TokenResponse should succeed");

        assert_eq!(
            token_response.access_token.as_ref(),
            "eyJhbGciOiJSUzI1NiIsInR5cCI6Ikp..sHQ"
        );
        assert_eq!(token_response.token_type, TokenType::Bearer);
        assert!(token_response.refresh_token.is_none());
        assert_eq!(token_response.scope, HashSet::new());
        assert_eq!(token_response.expires_in, Some(Duration::from_hours(24)));

        let authorization_details = token_response
            .authorization_details
            .as_ref()
            .expect("authorization_details should be present in Authorization Request");

        let entry = authorization_details
            .as_ref()
            .iter()
            .exactly_one()
            .expect("there should exactly one authorization_details entry");

        assert!(entry.locations.is_none());

        let TypedAuthorizationDetailsEntry::OpenidCredential(vci_id_entry) = &entry.typed_entry else {
            panic!("authorization details entry should be of type openid_credential");
        };

        assert_eq!(
            vci_id_entry.vci_entry.credential_configuration_id.as_ref(),
            "UniversityDegreeCredential"
        );
        assert_eq!(
            vci_id_entry.credential_identifiers,
            vec_nonempty![
                "CivilEngineeringDegree-2023".to_string(),
                "ElectricalEngineeringDegree-2023".to_string()
            ]
            .into()
        );
    }
}
