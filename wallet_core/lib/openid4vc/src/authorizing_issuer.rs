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
use url::Url;

use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::dpop::Dpop;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::issuer::TokenRequestError;
use crate::par;
use crate::par::PAR_TTL;
use crate::server_state::SessionStore;
use crate::store::Store;
use crate::token::TokenRequest;
use crate::token::TokenResponse;

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(derive_more::Debug, thiserror::Error)]
pub enum ParError {
    #[error("unknown client_id: {0}")]
    InvalidClient(String),

    #[error("storing PAR request failed: {0}")]
    Store(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// Errors that can occur during processing of an authorization request.
#[derive(derive_more::Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("unknown client_id: {0}")]
    InvalidClient(String),

    #[error("request_uri not found or expired: {0}")]
    UnknownRequestUri(String),

    #[error("consuming PAR request failed: {0}")]
    ParStore(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("the authorization request has no redirect_uri")]
    MissingRedirectUri,

    #[error("encoding authorization request as query string failed: {0}")]
    Encode(#[source] serde_urlencoded::ser::Error),
}

/// Authorization Phase wrapper around an Issuance Phase [`Issuer`].
pub struct AuthorizingIssuer<K, L, S, N, PAS, AF> {
    issuer: Arc<Issuer<K, L, S, N>>,
    par_store: Arc<PAS>,
    flow: AF,
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF> {
    pub fn new(issuer: Arc<Issuer<K, L, S, N>>, par_store: Arc<PAS>, flow: AF) -> Self {
        Self {
            issuer,
            par_store,
            flow,
        }
    }

    pub fn inner(&self) -> &Arc<Issuer<K, L, S, N>> {
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
            return Err(ParError::InvalidClient(request.oauth_request.client_id));
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
            return Err(AuthorizeError::InvalidClient(client_id.to_string()));
        }

        let authorization_request = self
            .par_store
            .consume(request_uri)
            .await
            .map_err(|error| AuthorizeError::ParStore(Box::new(error)))?
            .ok_or_else(|| AuthorizeError::UnknownRequestUri(request_uri.to_string()))?;

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
                let mut redirect_url = wallet_redirect_uri.ok_or(AuthorizeError::MissingRedirectUri)?;
                let query = serde_urlencoded::to_string([
                    ("code", code.as_ref()),
                    ("state", wallet_state.as_deref().unwrap_or("")),
                ])
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
        let code = token_request.code().clone();

        let issuables = self
            .flow
            .issuables(token_request.clone())
            .await
            .map_err(|error| TokenRequestError::AuthorizationCodeFlow(Box::new(error)))?;

        self.issuer
            .new_session_with_token(code.into(), issuables)
            .await
            .map_err(|error| TokenRequestError::IssuanceError(crate::issuer::IssuanceError::SessionStore(error)))?;

        self.issuer.process_token_request(token_request, dpop).await
    }
}
