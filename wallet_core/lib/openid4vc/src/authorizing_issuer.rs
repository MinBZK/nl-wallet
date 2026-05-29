//! Authorization Phase wrapper around the Issuance Phase [`Issuer`].
//!
//! [`AuthorizingIssuer`] owns the PAR store and an [`AuthorizationCodeFlow`] implementation.
//! It handles the Pushed Authorization Request and authorize endpoints, calls the flow at
//! both `/authorize` and `/token` (to resolve the authorization code into issuables), and
//! provisions those issuables into the inner [`Issuer`]'s session store before delegating the
//! actual token issuance. Deployments that only do the pre-authorized-code grant (no flow)
//! use the bare [`Issuer`] directly and never construct an [`AuthorizingIssuer`].

use std::error::Error as StdError;
use std::sync::Arc;

use crypto::EcdsaKey;
use serde::Serialize;
use url::Url;

use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::dpop::Dpop;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::par;
use crate::par::PAR_TTL;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::store::Store;
use crate::token::TokenRequest;
use crate::token::TokenResponse;

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(derive_more::Debug, thiserror::Error)]
pub enum ParError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("storing PAR request failed: {0}")]
    Store(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// Errors that can occur during processing of an authorization request.
#[derive(derive_more::Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("expected client_id from Authorization Request: {expected}, but was: {actual}")]
    MismatchedClient { expected: String, actual: String },

    #[error("request_uri not found or expired: {0}")]
    UnknownRequestUri(String),

    #[error("consuming PAR request failed: {0}")]
    ParStore(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("encoding authorization request as query string failed: {0}")]
    Encode(#[source] serde_urlencoded::ser::Error),
}

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(derive_more::Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("error writing to the issuer's session store: {0}")]
    SessionStoreWrite(#[source] SessionStoreError),

    #[error("error when delegating handling the token request to the issuer: {0}")]
    IssuerTokenRequest(#[source] crate::issuer::TokenRequestError),
}

/// Authorization Phase wrapper around an Issuance Phase [`Issuer`].
#[derive(derive_more::Constructor)]
pub struct AuthorizingIssuer<K, L, S, N, PAS, AF> {
    issuer: Arc<Issuer<K, L, S, N>>,
    par_store: PAS,
    flow: AF,
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF> {
    pub fn issuer(&self) -> &Arc<Issuer<K, L, S, N>> {
        &self.issuer
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    PAS: Store<String, VciAuthorizationRequest>,
{
    pub async fn process_pushed_authorization_request(
        &self,
        request: VciAuthorizationRequest,
    ) -> Result<PushedAuthorizationResponse, ParError> {
        if !self
            .issuer
            .accepted_wallet_client_ids()
            .any(|id| id == request.oauth_request.client_id.as_str())
        {
            return Err(ParError::UnknownClient(request.oauth_request.client_id));
        }

        let request_uri = par::generate_request_uri();

        self.par_store
            .store(request_uri.clone(), request)
            .await
            .map_err(|error| ParError::Store(Box::new(error)))?;

        Ok(PushedAuthorizationResponse {
            request_uri,
            expires_in: PAR_TTL,
        })
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    PAS: Store<String, VciAuthorizationRequest>,
    AF: AuthorizationCodeFlow,
{
    /// Consume the PAR, hand the resolved authorization request to the configured
    /// [`AuthorizationCodeFlow`], and translate the [`AuthorizeOutcome`] into the URL the
    /// wallet should be redirected to.
    pub async fn process_authorize(&self, request_uri: &str, client_id: &str) -> Result<Url, AuthorizeError> {
        if !self.issuer.accepted_wallet_client_ids().any(|id| id == client_id) {
            return Err(AuthorizeError::UnknownClient(client_id.to_string()));
        }

        let authorization_request = self
            .par_store
            .consume(request_uri)
            .await
            .map_err(|error| AuthorizeError::ParStore(Box::new(error)))?
            .ok_or_else(|| AuthorizeError::UnknownRequestUri(request_uri.to_string()))?;

        // TODO (PVW-5953): unit test these checks
        if authorization_request.oauth_request.client_id != client_id {
            return Err(AuthorizeError::MismatchedClient {
                expected: authorization_request.oauth_request.client_id,
                actual: client_id.to_string(),
            });
        }

        // Capture the wallet's redirect_uri + state up front so we can build the wallet-facing
        // redirect URL when the flow yields an authorization code.
        let wallet_redirect_uri = authorization_request.redirect_uri.clone();
        let wallet_state = authorization_request.oauth_request.state.clone();

        let outcome = self
            .flow
            .authorize(authorization_request)
            .await
            .map_err(|error| AuthorizeError::AuthorizationCodeFlow(Box::new(error)))?;

        match outcome {
            AuthorizeOutcome::RedirectTo(url) => Ok(url),
            AuthorizeOutcome::IssuedCode(code) => {
                #[derive(Serialize)]
                struct RedirectQuery<'a> {
                    code: &'a str,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    state: Option<&'a str>,
                }

                let mut redirect_url = wallet_redirect_uri.into_inner();
                let query = serde_urlencoded::to_string(RedirectQuery {
                    code: code.as_ref(),
                    state: wallet_state.as_deref(),
                })
                .map_err(AuthorizeError::Encode)?;
                redirect_url.set_query(Some(&query));
                Ok(redirect_url)
            }
        }
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    K: EcdsaKey,
    S: SessionStore<IssuanceData>,
    AF: AuthorizationCodeFlow,
{
    /// Call the flow to resolve the token request's code into issuables, write those into the
    /// inner issuer's session store keyed by the code, then delegate to the inner `/token`
    /// path which will read the newly created session and issue.
    pub async fn process_token_request(
        &self,
        token_request: TokenRequest,
        dpop: Dpop,
    ) -> Result<(TokenResponse, String), TokenRequestError> {
        // TODO (PVW-5953): implicitly accepts the authorization code if issuables resolves (using upstream to verify
        // code). Make explicit
        let code = token_request.code().clone();

        let issuables = self
            .flow
            .issuables(token_request.clone())
            .await
            .map_err(|error| TokenRequestError::AuthorizationCodeFlow(Box::new(error)))?;

        // TODO (PVW-5953): the code below creates a new session and then immediately fetches it in
        // process_token_request. Cleanup necessary
        self.issuer
            .new_session_with_token(code.into(), issuables)
            .await
            .map_err(TokenRequestError::SessionStoreWrite)?;

        let result = self
            .issuer
            .process_token_request(token_request, dpop)
            .await
            .map_err(TokenRequestError::IssuerTokenRequest)?;
        Ok(result)
    }
}
