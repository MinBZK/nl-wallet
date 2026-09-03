use std::collections::HashSet;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;

use crypto::utils::random_string;
use crypto::utils::sha256;
use derive_more::From;
use http_utils::reqwest::HttpClient;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::DurationSeconds;
use serde_with::StringWithSeparator;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

use crate::errors::RemoteErrorResponse;
use crate::metadata::oauth_metadata::OidcProviderMetadata;
use crate::scope::Scope;

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

impl AccessToken {
    /// Construct a new random access token, with the specified authorization code appended to it.
    pub fn new(code: &AuthorizationCode) -> Self {
        Self(random_string(32) + code.as_ref())
    }

    /// Returns the authorization code appended to this access token.
    pub fn code(&self) -> Option<AuthorizationCode> {
        self.as_ref().get(32..).map(|code| AuthorizationCode(code.to_string()))
    }

    pub fn sha256(&self) -> Vec<u8> {
        sha256(self.as_ref().as_bytes())
    }
}

/// The token type of an access token, as defined by
/// [RFC 6749 §7.1](https://www.rfc-editor.org/rfc/rfc6749.html#section-7.1) (`Bearer`) and
/// [RFC 9449](https://www.rfc-editor.org/rfc/rfc9449.html) (`DPoP`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    #[default]
    Bearer,
    DPoP,
}

/// An OAuth 2.0 Token Request as defined by [RFC 6749 §4.1.3](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3),
/// generic over the `grant_type` value, since the set of supported grant types (and their accompanying fields) vary
/// per profile.
///
/// Sent URL-encoded in the request body to `POST /token`.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest<G> {
    #[serde(flatten)]
    pub grant_type: G,

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
}

/// An OAuth 2.0 Token Response as defined by
/// [RFC 6749 §5.1](https://www.rfc-editor.org/rfc/rfc6749.html#section-5.1).
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,

    /// Section 3.3 of RFC 6749 states that if the issued access token scope is different than the one requested by the
    /// client, the server MUST include a scope response parameter to inform the client of the actual scope granted.
    /// Conversely, this means that if the scope is absent, access token scope is what the client requested.
    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, Scope>>")]
    pub scope: Option<HashSet<Scope>>,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub expires_in: Option<Duration>,
}

impl TokenResponse {
    pub fn new(access_token: AccessToken, token_type: TokenType) -> Self {
        Self {
            access_token,
            token_type,
            expires_in: None,
            refresh_token: None,
            scope: None,
        }
    }
}

/// The `authorization_code` grant type, as defined by
/// [RFC 6749 §4.1.3](https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3).
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum AuthorizationCodeGrantType {
    AuthorizationCode { code: AuthorizationCode },
}

/// Error type for token endpoint requests (see [`request_token`]).
#[derive(Debug, thiserror::Error)]
pub enum TokenEndpointError<TE>
where
    TE: Debug,
{
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("error response from token endpoint: {0:?}")]
    ErrorResponse(Box<RemoteErrorResponse<TE>>),
}

/// Exchange `token_request` for an access token at the provider's token endpoint.
///
/// Generic over the response type `R`, since profiles that extend the token response (such as OpenID4VCI's
/// `VciTokenResponse`) need their extended fields deserialized too; pass [`TokenResponse`] for a plain OAuth 2.0
/// token response.
pub async fn request_token<G, R, TE>(
    http_client: &HttpClient,
    config: &OidcProviderMetadata,
    token_request: TokenRequest<G>,
) -> Result<R, TokenEndpointError<TE>>
where
    G: Serialize,
    R: DeserializeOwned,
    TE: Debug + FromStr,
{
    let response = http_client
        .post(config.oauth_metadata.token_endpoint.clone(), |request| {
            request.form(&token_request)
        })
        .await?;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let error = response.json::<RemoteErrorResponse<TE>>().await?;
        return Err(TokenEndpointError::ErrorResponse(Box::new(error)));
    }

    Ok(response.json::<R>().await?)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpClient;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use url::Url;

    use super::*;
    use crate::errors::ErrorResponse;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::oauth_metadata::OidcProviderMetadata;

    #[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    enum TestTokenErrorCode {
        InvalidRequest,
    }

    fn create_token_request() -> TokenRequest<AuthorizationCodeGrantType> {
        TokenRequest {
            grant_type: AuthorizationCodeGrantType::AuthorizationCode {
                code: AuthorizationCode::from("test-code".to_string()),
            },
            client_id: None,
            redirect_uri: Some("https://example.com/callback".parse::<Url>().unwrap()),
            scope: None,
            code_verifier: Some("test-verifier".to_string()),
        }
    }

    fn create_metadata(server: &MockServer) -> OidcProviderMetadata {
        let issuer_identifier: IssuerIdentifier = server.base_url().parse().unwrap();
        OidcProviderMetadata::new_mock(issuer_identifier)
    }

    #[tokio::test]
    async fn request_token_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);

        let token_response_body =
            TokenResponse::new(AccessToken::from("test-access-token".to_string()), TokenType::Bearer);
        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&token_response_body).unwrap());
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await;

        assert_eq!(result.unwrap().access_token.as_ref(), "test-access-token");
    }

    #[tokio::test]
    async fn request_token_endpoint_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let error_response = ErrorResponse {
            error: TestTokenErrorCode::InvalidRequest,
            error_description: Some("invalid code".to_string()),
            error_uri: None,
        };

        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(400)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&error_response).unwrap());
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await;

        assert_matches!(result, Err(TokenEndpointError::ErrorResponse(_)));
    }
}
