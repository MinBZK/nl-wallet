use std::time::Duration;

use indexmap::IndexSet;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::{serde_as, skip_serializing_none};
use serde_with::{DurationSeconds, StringWithSeparator};
use url::Url;

use crate::{authorization::AuthorizationDetails, ErrorStatusCode};

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

impl TokenRequest {
    /// Retrieve either the authorization code or the pre-authorized code, depending on the authorization grant type.
    pub fn code(&self) -> &str {
        match &self.grant_type {
            TokenRequestGrantType::AuthorizationCode { code } => code,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } => pre_authorized_code,
        }
    }
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
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,
    pub c_nonce: Option<String>,

    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, String>>")]
    pub scope: Option<IndexSet<String>>,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub c_nonce_expires_in: Option<Duration>, // lifetime of `c_nonce`

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub expires_in: Option<Duration>,

    /// "REQUIRED when authorization_details parameter is used to request issuance of a certain Credential type
    /// as defined in Section 5.1.1. MUST NOT be used otherwise."
    pub authorization_details: Option<AuthorizationDetails>,
}

/// A [`TokenRespone`] with an extra field for the attestation previews.
/// This is an custom field so other implementations might not send it. For now however we assume that it is always
/// present so it is not an [`Option`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponseWithPreviews<T> {
    #[serde(flatten)]
    pub token_response: TokenResponse,
    pub attestation_previews: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum TokenType {
    #[default]
    Bearer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TokenErrorType {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    ServerError,
    AuthorizationPending, // OpenID4VCI-specific error type
    SlowDown,             // OpenID4VCI-specific error type
}

impl ErrorStatusCode for TokenErrorType {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            TokenErrorType::InvalidRequest => StatusCode::BAD_REQUEST,
            TokenErrorType::InvalidClient => StatusCode::UNAUTHORIZED,
            TokenErrorType::InvalidGrant => StatusCode::BAD_REQUEST,
            TokenErrorType::UnauthorizedClient => StatusCode::BAD_REQUEST,
            TokenErrorType::UnsupportedGrantType => StatusCode::BAD_REQUEST,
            TokenErrorType::InvalidScope => StatusCode::BAD_REQUEST,
            TokenErrorType::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            TokenErrorType::AuthorizationPending => StatusCode::BAD_REQUEST,
            TokenErrorType::SlowDown => StatusCode::BAD_REQUEST,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use indexmap::IndexSet;

    use crate::token::{TokenRequest, TokenRequestGrantType, TokenResponse};

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
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Apre-authorized_code&pre-authorized_code=123&code_verifier=myverifier&client_id=myclient&redirect_uri=https%3A%2F%2Fexample.com%2F",
        )
    }

    #[test]
    fn token_response_serialization() {
        assert_eq!(
            serde_json::to_string(&TokenResponse {
                access_token: "access_token".to_string(),
                token_type: crate::token::TokenType::Bearer,
                c_nonce: Some("c_nonce".to_string()),
                scope: Some(IndexSet::from_iter(["scope1".to_string(), "scope2".to_string()])),
                c_nonce_expires_in: Some(Duration::from_secs(10)),
                expires_in: None,
                refresh_token: None,
                authorization_details: None,
            })
            .unwrap(),
            r#"{"access_token":"access_token","token_type":"Bearer","c_nonce":"c_nonce","scope":"scope1 scope2","c_nonce_expires_in":10}"#.to_string(),
        )
    }
}
