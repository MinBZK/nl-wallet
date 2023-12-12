use nl_wallet_mdoc::basic_sa_ext::UnsignedMdoc;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::authorization::AuthorizationDetails;

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-token-request
/// and https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3.
/// Sent URL-encoded in request body to POST /token.
/// A DPoP HTTP header may be included.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequest {
    #[serde(flatten)]
    pub grant_type: TokenRequestGrantType,

    pub code_verifier: Option<String>,
    pub client_id: Option<String>,

    /// MUST be the redirect URI value as passed to the authorization request
    pub redirect_uri: Option<Url>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "snake_case")]
#[serde(tag = "grant_type")]
pub enum TokenRequestGrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode { code: String },
    #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
    PreAuthorizedCode {
        #[serde(rename = "pre-authorized_code")]
        pre_authorized_code: String,
    },
}

/// https://openid.github.io/OpenID4VCI/openid-4-verifiable-credential-issuance-wg-draft.html#name-successful-token-response
/// and https://www.rfc-editor.org/rfc/rfc6749.html#section-5.1
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: TokenType,
    pub expires_in: Option<u64>, // amount of seconds from now
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub c_nonce: Option<String>,
    pub c_nonce_expires_in: Option<u64>, // lifetime of `c_nonce` in seconds

    /// "REQUIRED when authorization_details parameter is used to request issuance of a certain Credential type
    /// as defined in Section 5.1.1. MUST NOT be used otherwise."
    pub authorization_details: Option<AuthorizationDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponseWithPreviews {
    #[serde(flatten)]
    pub token_response: TokenResponse,
    pub attestation_previews: Vec<UnsignedMdoc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TokenType {
    Bearer,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenErrorResponse {
    pub error: TokenErrorType,
    pub error_description: Option<String>,
    pub error_uri: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TokenErrorType {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    AuthorizationPending, // OpenID4VCI-specific error type
    SlowDown,             // OpenID4VCI-specific error type
}

#[cfg(test)]
mod tests {
    use crate::token::{TokenRequest, TokenRequestGrantType};

    #[test]
    fn token_request_serialization() {
        assert_eq!(
            serde_urlencoded::to_string(TokenRequest {
                grant_type: TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: "123".to_string()
                },
                code_verifier: Some("myverifier".to_string()),
                client_id: Some("myclient".to_string()),
                redirect_uri: Some("https://example.com".parse().unwrap())
            })
            .unwrap(),
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Apre-authorized_code&pre-authorized_code=123&code_verifier=myverifier&client=myclient&redirect_uri=https%3A%2F%2Fexample.com%2F",
        )
    }
}
