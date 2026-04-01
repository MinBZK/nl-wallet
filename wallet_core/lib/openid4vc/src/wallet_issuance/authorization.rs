use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use rustls_pki_types::TrustAnchor;
use url::Url;

use error_category::ErrorCategory;
use http_utils::reqwest::HttpJsonClient;

use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::authorization::AuthorizationRequest;
use crate::authorization::AuthorizationResponse;
use crate::authorization::PkceCodeChallenge;
use crate::authorization::ResponseType;
use crate::metadata::issuer_metadata::IssuerMetadata;
use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
use crate::pkce::PkcePair;
use crate::pkce::S256PkcePair;
use crate::token::AuthorizationCode;
use crate::token::TokenRequest;
use crate::token::TokenRequestGrantType;
use crate::wallet_issuance::AuthorizationSession;
use crate::wallet_issuance::WalletIssuanceError;
use crate::wallet_issuance::issuance_session::HttpIssuanceSession;
use crate::wallet_issuance::issuance_session::HttpVcMessageClient;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum OAuthError {
    #[error("URL encoding error: {0}")]
    UrlEncoding(#[from] serde_urlencoded::ser::Error),

    #[error("URL decoding error: {0}")]
    UrlDecoding(#[from] serde::de::value::Error),

    #[error("error requesting authorization code: {0:?}")]
    RedirectUriError(Box<ErrorResponse<AuthorizationErrorCode>>),

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

    #[error("user denied authentication")]
    #[category(expected)]
    Denied,
}

/// The state of an in-progress OAuth authorization code flow.
#[derive(Debug)]
pub struct HttpAuthorizationSession<P = S256PkcePair> {
    issuer_metadata: IssuerMetadata,
    oauth_metadata: AuthorizationServerMetadata,
    http_client: HttpJsonClient,

    pub auth_url: Url,
    client_id: String,
    redirect_uri: Url,
    pkce_pair: P,
    state: String,
}

impl<P: PkcePair> HttpAuthorizationSession<P> {
    /// Create a new authorization server session and compute the authorization URL.
    /// Returns an error if the provider has no authorization endpoint or the URL cannot be encoded.
    pub fn try_new(
        http_client: HttpJsonClient,
        issuer_metadata: IssuerMetadata,
        oauth_metadata: AuthorizationServerMetadata,
        client_id: String,
        redirect_uri: Url,
    ) -> Result<Self, OAuthError> {
        let pkce_pair = P::generate();
        let state = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));
        let nonce = BASE64_URL_SAFE_NO_PAD.encode(crypto::utils::random_bytes(16));

        let params = AuthorizationRequest {
            response_type: ResponseType::Code.into(),
            client_id: client_id.clone(),
            redirect_uri: Some(redirect_uri.clone()),
            state: Some(state.clone()),
            authorization_details: None,
            request_uri: None,
            code_challenge: Some(PkceCodeChallenge::S256 {
                code_challenge: pkce_pair.code_challenge().to_string(),
            }),
            scope: oauth_metadata.scopes_supported.clone(),
            nonce: Some(nonce),
            response_mode: None,
        };

        let mut auth_url = oauth_metadata
            .authorization_endpoint
            .clone()
            .ok_or(OAuthError::NoAuthorizationEndpoint)?;

        auth_url.set_query(Some(&serde_urlencoded::to_string(params)?));

        Ok(Self {
            issuer_metadata,
            oauth_metadata,
            http_client,
            auth_url,
            client_id,
            redirect_uri,
            pkce_pair,
            state,
        })
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

impl AuthorizationSession for HttpAuthorizationSession {
    type Issuance = HttpIssuanceSession;

    fn auth_url(&self) -> &Url {
        &self.auth_url
    }

    async fn start_issuance(
        self,
        received_redirect_uri: &Url,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self::Issuance, WalletIssuanceError> {
        let pre_authorized_code = self.authorization_code(received_redirect_uri)?;
        let message_client = HttpVcMessageClient::new(self.http_client);
        let token_endpoint = self.oauth_metadata.token_endpoint;

        let token_request = TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: Some(self.pkce_pair.into_code_verifier()),
            client_id: Some(self.client_id),
            redirect_uri: Some(self.redirect_uri),
        };

        HttpIssuanceSession::start_issuance_inner(
            message_client,
            self.issuer_metadata,
            token_endpoint,
            token_request,
            trust_anchors,
        )
        .await
    }
}
