pub use jsonwebtoken::jwk::JwkSet;

use base64::prelude::*;
use url::Url;

use error_category::ErrorCategory;

use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::TokenErrorCode;
use crate::authorization::AuthorizationRequest;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::ResponseType;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::well_known::WellKnownError;

use super::AuthorizationServerMetadata;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OAuthError {
    #[error("transport error: {0}")]
    #[category(expected)]
    Http(#[from] reqwest::Error),

    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),

    #[error("URL decoding error: {0}")]
    UrlDecoding(#[from] serde::de::value::Error),

    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(Box<ErrorResponse<AuthorizationErrorCode>>),

    #[error("error requesting access token: {0:?}")]
    RequestingAccessToken(Box<ErrorResponse<TokenErrorCode>>),

    #[error("invalid state token received in redirect URI")]
    #[category(critical)]
    StateTokenMismatch,

    #[error("no authorization code received in redirect URI")]
    #[category(critical)]
    NoAuthCode,

    #[error("invalid redirect URI received")]
    #[category(critical)]
    RedirectUriMismatch,

    #[error("config has no authorization endpoint")]
    #[category(critical)]
    NoAuthorizationEndpoint,

    #[error("config has no JWKS URI")]
    #[category(critical)]
    NoJwksUri,

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,

    #[error("error fetching well-known metadata: {0}")]
    #[category(critical)]
    WellKnown(#[from] WellKnownError),
}

/// Holds the state of an in-progress OAuth authorization code flow
/// and is consumed by [`into_token_request`] once the user redirects back.
///
/// [`into_token_request`]: AuthorizationServer::into_token_request
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AuthorizationServer {
    /// Create an OpenID Token Request based on the contents of the redirect URI received.
    ///
    /// Note that this consumes the [`AuthorizationServer`], either on success or failure.
    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OAuthError>;
}

/// The state of an in-progress OAuth authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationServer<P = S256PkcePair> {
    provider: AuthorizationServerMetadata,
    client_id: String,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
    nonce: String,
}

impl<P: PkcePair> HttpAuthorizationServer<P> {
    pub fn new(provider: AuthorizationServerMetadata, client_id: String, redirect_uri: Url) -> Self {
        Self {
            provider,
            client_id,
            redirect_uri,
            pkce_pair: P::generate(),
            state: BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16)),
            nonce: BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16)),
        }
    }

    /// Returns the authorization URL to redirect the user to, with all PKCE/CSRF/nonce parameters encoded.
    pub fn auth_url(&self) -> Result<Url, OAuthError> {
        let params = AuthorizationRequest {
            response_type: ResponseType::Code.into(),
            client_id: self.client_id.clone(),
            redirect_uri: Some(self.redirect_uri.clone()),
            state: Some(self.state.clone()),
            authorization_details: None,
            request_uri: None,
            code_challenge: Some(PkceCodeChallenge::S256 {
                code_challenge: self.pkce_pair.code_challenge().to_string(),
            }),
            scope: self.provider.scopes_supported.clone(),
            nonce: Some(self.nonce.clone()),
            response_mode: None,
        };

        let mut url = self
            .provider
            .authorization_endpoint
            .clone()
            .ok_or(OAuthError::NoAuthorizationEndpoint)?;
        url.set_query(Some(&serde_urlencoded::to_string(params)?));
        Ok(url)
    }

    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool {
        received_redirect_uri.as_str().starts_with(self.redirect_uri.as_str())
    }

    fn authorization_code(&self, received_redirect_uri: &Url) -> Result<AuthorizationCode, OAuthError> {
        if !self.matches_received_redirect_uri(received_redirect_uri) {
            return Err(OAuthError::RedirectUriMismatch);
        }

        let auth_response = received_redirect_uri.query().ok_or(OAuthError::NoAuthCode)?;

        // First see if we received an error
        if received_redirect_uri.query_pairs().any(|(key, _)| key == "error") {
            let err_response: ErrorResponse<AuthorizationErrorCode> = serde_urlencoded::from_str(auth_response)?;

            if err_response.error == AuthorizationErrorCode::AccessDenied {
                return Err(OAuthError::Denied);
            } else {
                return Err(OAuthError::RedirectUriError(Box::new(err_response)));
            }
        }

        let auth_response: AuthorizationResponse = serde_urlencoded::from_str(auth_response)?;
        if auth_response.state.as_ref() != Some(&self.state) {
            return Err(OAuthError::StateTokenMismatch);
        }

        Ok(auth_response.code.into())
    }
}

impl<P: PkcePair> AuthorizationServer for HttpAuthorizationServer<P> {
    fn into_token_request(self, received_redirect_uri: &Url) -> Result<TokenRequest, OAuthError> {
        let pre_authorized_code = self.authorization_code(received_redirect_uri)?;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: Some(self.pkce_pair.into_code_verifier()),
            client_id: Some(self.client_id),
            redirect_uri: Some(self.redirect_uri),
        };

        Ok(token_request)
    }
}

#[cfg(any(test, feature = "mock"))]
impl<P: PkcePair> HttpAuthorizationServer<P> {
    /// Returns the CSRF state token. Available in test/mock builds to allow constructing valid redirect URIs.
    pub fn csrf_state(&self) -> &str {
        &self.state
    }
}
