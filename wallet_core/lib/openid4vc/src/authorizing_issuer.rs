//! Authorization Phase wrapper around the Issuance Phase [`Issuer`].
//!
//! [`AuthorizingIssuer`] owns the PAR store and an [`AuthorizationCodeFlow`] implementation.
//! It handles the Pushed Authorization Request and authorize endpoints, and exposes
//! [`complete_authorization`] for the configured flow to call once it has authenticated the holder
//! and determined the issuables. Once `complete_authorization` has
//! written the `AuthCodeIssued` session, the framework's `/token` on the inner [`Issuer`]) is used in the next step of
//! the flow. Deployments that only do the pre-authorized-code grant (no flow) use the bare [`Issuer`]
//! directly and never construct an [`AuthorizingIssuer`].

use std::error::Error;
use std::sync::Arc;

use crypto::utils::random_string;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::credential_offer::CredentialOffer;
use crate::issuable_document::IssuableDocument;
use crate::issuer::AuthCodeIssued;
use crate::issuer::Grant;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::par;
use crate::par::PAR_TTL;
use crate::server_state::SessionState;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::store::Store;
use crate::token::AuthorizationCode;

const AUTH_CODE_LENGTH: usize = 32;

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(Debug, thiserror::Error)]
pub enum ParError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("storing PAR request failed: {0}")]
    Store(#[source] Box<dyn Error + Send + Sync + 'static>),
}

/// Errors that can occur during processing of an authorization request.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("expected client_id from Authorization Request: {expected}, but was: {actual}")]
    MismatchedClient { expected: String, actual: String },

    #[error("request_uri not found or expired: {0}")]
    UnknownRequestUri(String),

    #[error("consuming PAR request failed: {0}")]
    ParStore(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("encoding authorization request as query string failed: {0}")]
    EncodeRedirectQuery(#[source] serde_urlencoded::ser::Error),
}

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(Debug, thiserror::Error)]
pub enum TokenRequestError {
    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("error writing to the issuer's session store: {0}")]
    SessionStoreWrite(#[source] SessionStoreError),

    #[error("error when delegating handling the token request to the issuer: {0}")]
    IssuerTokenRequest(#[source] crate::issuer::TokenRequestError),
}

/// Errors that can occur while writing the auth-code-grant session and building the
/// wallet-facing redirect URL from [`AuthorizingIssuer::complete_authorization`].
#[derive(derive_more::Debug, thiserror::Error)]
pub enum CompleteAuthorizationError {
    #[error("writing authorization-code session failed: {0}")]
    SessionStore(#[source] SessionStoreError),

    #[error("encoding wallet redirect query string failed: {0}")]
    EncodeRedirectQuery(#[source] serde_urlencoded::ser::Error),
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

    /// Creates a [`CredentialOffer`] for all Credential Configurations present in the issuer. This offer instructs a
    /// wallet to start an Autorization Code flow to obtain the credentials.
    pub fn authorization_code_credential_offer(&self) -> CredentialOffer {
        let credential_configuration_ids = self
            .issuer
            .credential_configs()
            .all_configuration_ids()
            .into_nonempty_iter()
            .map(Clone::clone)
            .collect();

        CredentialOffer::new_authorization(
            self.issuer.issuer_identifier().clone(),
            credential_configuration_ids,
            None,
        )
    }

    pub fn flow(&self) -> &AF {
        &self.flow
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
                let wallet_redirect_uri = wallet_redirect_uri.into_inner();
                build_wallet_redirect(wallet_redirect_uri, &code, wallet_state.as_deref())
                    .map_err(AuthorizeError::EncodeRedirectQuery)
            }
        }
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    S: SessionStore<IssuanceData>,
{
    /// Called by the configured [`AuthorizationCodeFlow`] once it has authenticated the holder and
    /// produced the issuables. How the flow does so (e.g. via an external identity provider) is an
    /// implementation detail of the flow and not modelled here. Generates a fresh authorization code, writes
    /// an `AuthCodeIssued` session keyed by it (with `Grant::AuthorizationCode` carrying both the
    /// issuables and the wallet's PKCE `code_challenge`), and builds the wallet-facing redirect
    /// URL with that code and the wallet's original `state`. The framework's `/token` handler
    /// will later load the session, verify the wallet's `code_verifier` against the stored
    /// challenge, and issue. Returns both the generated code (for test helpers using `/token`
    /// directly) and the redirect URL.
    pub async fn complete_authorization(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
        wallet_code_challenge: String,
        wallet_redirect_uri: Url,
        wallet_state: Option<String>,
    ) -> Result<(AuthorizationCode, Url), CompleteAuthorizationError> {
        let code = AuthorizationCode::from(random_string(AUTH_CODE_LENGTH));

        let state = SessionState::new(
            code.clone().into(),
            IssuanceData::AuthCodeIssued(Box::new(AuthCodeIssued {
                issuable_documents,
                grant: Grant::AuthorizationCode { wallet_code_challenge },
            })),
        );
        self.issuer
            .write_session(state)
            .await
            .map_err(CompleteAuthorizationError::SessionStore)?;

        let redirect_url = build_wallet_redirect(wallet_redirect_uri, &code, wallet_state.as_deref())
            .map_err(CompleteAuthorizationError::EncodeRedirectQuery)?;

        Ok((code, redirect_url))
    }
}

/// Build the wallet-facing redirect URL carrying the authorization code and the wallet's original `state`.
fn build_wallet_redirect(
    mut redirect_uri: Url,
    code: &AuthorizationCode,
    wallet_state: Option<&str>,
) -> Result<Url, serde_urlencoded::ser::Error> {
    #[derive(Serialize)]
    struct RedirectQuery<'a> {
        code: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<&'a str>,
    }

    let query = serde_urlencoded::to_string(RedirectQuery {
        code: code.as_ref(),
        state: wallet_state,
    })?;

    redirect_uri.set_query(Some(&query));
    Ok(redirect_uri)
}
