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

    #[error("redirect_uri not allowed: {0}")]
    InvalidRedirectUri(Url),

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
    /// Exact-match allowlist of `redirect_uri` values the wallet may use in a Pushed Authorization
    /// Request. Validated at `/par`.
    wallet_redirect_uris: VecNonEmpty<Url>,
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

        // Exact-match the wallet's redirect_uri against the configured allowlist.
        if !self
            .wallet_redirect_uris
            .iter()
            .any(|uri| uri == request.redirect_uri.as_ref())
        {
            return Err(ParError::InvalidRedirectUri(request.redirect_uri.as_ref().clone()));
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
    /// [`AuthorizationCodeFlow`], and translate the [`AuthorizeOutcome`] into the URL
    /// the wallet should be redirected to.
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
            .write_new_session(state)
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

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use url::Url;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::AuthorizeError;
    use super::AuthorizingIssuer;
    use super::ParError;
    use crate::authorization::VciAuthorizationRequest;
    use crate::authorization_code_flow::AuthorizationCodeFlow;
    use crate::authorization_code_flow::AuthorizeOutcome;
    use crate::issuer::Grant;
    use crate::issuer::IssuanceData;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::par::PAR_TTL;
    use crate::pkce::PkcePair;
    use crate::pkce::S256PkcePair;
    use crate::server_state::MemorySessionStore;
    use crate::server_state::SessionStore;
    use crate::server_state::SessionToken;
    use crate::store::MemoryStore;
    use crate::store::Store;
    use crate::test::MockIssuer;
    use crate::test::mock_issuable_documents;
    use crate::test::setup_mock_issuer;
    use crate::token::AuthorizationCode;

    const OTHER_CLIENT_ID: &str = "definitely-not-the-wallet";
    const REQUEST_URI: &str = "urn:ietf:params:oauth:request_uri:test";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_STATE: &str = "wallet-state";

    fn upstream_url() -> Url {
        "https://auth.example.com/oauth2/authorize".parse().unwrap()
    }

    /// Allowlist containing exactly the `WALLET_REDIRECT_URI` that `vci_request` uses, so the PAR
    /// redirect_uri check passes in tests that aren't exercising redirect_uri rejection.
    fn allowed_redirect_uris() -> VecNonEmpty<Url> {
        vec_nonempty![WALLET_REDIRECT_URI.parse().unwrap()]
    }

    fn vci_request(client_id: &str) -> VciAuthorizationRequest {
        VciAuthorizationRequest::for_auth_code(
            client_id.to_string(),
            WALLET_REDIRECT_URI.parse().unwrap(),
            WALLET_STATE.to_string(),
            None,
            &S256PkcePair::generate(),
        )
    }

    /// Build an inner [`Issuer`] (accepting only `MOCK_WALLET_CLIENT_ID`) and a PAR store seeded with
    /// the given entries. Seeding the store directly lets us plant a PAR whose `client_id` differs
    /// from the one presented at `/authorize`. Also returns the session store so tests can read back
    /// the session written by `complete_authorization`.
    fn issuer_and_par(
        entries: Vec<(String, VciAuthorizationRequest)>,
    ) -> (
        Arc<MockIssuer>,
        MemoryStore<String, VciAuthorizationRequest>,
        Arc<MemorySessionStore<IssuanceData>>,
    ) {
        let issuer_identifier = IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap();
        let sessions = Arc::new(MemorySessionStore::default());
        let (issuer, _, _) = setup_mock_issuer(issuer_identifier, NonZeroUsize::MIN, Arc::clone(&sessions));

        let par_store = MemoryStore::new(PAR_TTL);
        for (request_uri, request) in entries {
            par_store.store_inner(request_uri, request);
        }

        (Arc::new(issuer), par_store, sessions)
    }

    /// Minimal [`AuthorizationCodeFlow`] that returns a preconfigured outcome, so `process_authorize`
    /// can be exercised independently of any real flow's behaviour.
    struct FixedOutcomeFlow(AuthorizeOutcome);

    impl AuthorizationCodeFlow for FixedOutcomeFlow {
        type Error = Infallible;

        async fn authorize(&self, _request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
            Ok(match &self.0 {
                AuthorizeOutcome::RedirectTo(url) => AuthorizeOutcome::RedirectTo(url.clone()),
                AuthorizeOutcome::IssuedCode(code) => AuthorizeOutcome::IssuedCode(code.clone()),
            })
        }
    }

    #[tokio::test]
    async fn process_par_rejects_unknown_client_id() {
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let error = authorizing_issuer
            .process_pushed_authorization_request(vci_request(OTHER_CLIENT_ID))
            .await
            .unwrap_err();

        assert_matches!(error, ParError::UnknownClient(client_id) if client_id == OTHER_CLIENT_ID);
        // The rejected request must not have been stored.
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_par_stores_request_and_returns_response() {
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let response = authorizing_issuer
            .process_pushed_authorization_request(vci_request(MOCK_WALLET_CLIENT_ID))
            .await
            .unwrap();

        assert_eq!(response.expires_in, PAR_TTL);
        assert!(response.request_uri.starts_with("urn:ietf:params:oauth:request_uri:"));
        assert_eq!(authorizing_issuer.par_store.len(), 1);

        // The stored request is retrievable under the returned request_uri.
        let stored = authorizing_issuer
            .par_store
            .consume(response.request_uri.as_str())
            .await
            .unwrap()
            .expect("request should be stored under the returned request_uri");
        assert_eq!(stored.oauth_request.client_id, MOCK_WALLET_CLIENT_ID);
    }

    #[tokio::test]
    async fn process_par_rejects_disallowed_redirect_uri() {
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        // The client_id is accepted, but the allowlist does not contain WALLET_REDIRECT_URI, so the
        // request must be rejected on the redirect_uri check alone.
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            vec_nonempty!["https://other.example.com/callback".parse().unwrap()],
        );

        let error = authorizing_issuer
            .process_pushed_authorization_request(vci_request(MOCK_WALLET_CLIENT_ID))
            .await
            .unwrap_err();

        assert_matches!(error, ParError::InvalidRedirectUri(uri) if uri.as_str() == WALLET_REDIRECT_URI);
        // The rejected request must not have been stored.
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_authorize_rejects_unknown_client_id() {
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, OTHER_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(error, AuthorizeError::UnknownClient(client_id) if client_id == OTHER_CLIENT_ID);
    }

    #[tokio::test]
    async fn process_authorize_rejects_unknown_request_uri() {
        // Caller is accepted, but the PAR store is empty, so the request_uri can't be resolved.
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(error, AuthorizeError::UnknownRequestUri(request_uri) if request_uri == REQUEST_URI);
    }

    #[tokio::test]
    async fn process_authorize_rejects_mismatched_client_id() {
        // The PAR was pushed under a different client_id than the one presented at /authorize.
        let (issuer, par_store, _sessions) =
            issuer_and_par(vec![(REQUEST_URI.to_string(), vci_request(OTHER_CLIENT_ID))]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(
            error,
            AuthorizeError::MismatchedClient { expected, actual }
                if expected == OTHER_CLIENT_ID && actual == MOCK_WALLET_CLIENT_ID
        );
    }

    #[tokio::test]
    async fn process_authorize_passes_through_redirect_outcome() {
        let (issuer, par_store, _sessions) =
            issuer_and_par(vec![(REQUEST_URI.to_string(), vci_request(MOCK_WALLET_CLIENT_ID))]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let redirect_url = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap();

        assert_eq!(redirect_url, upstream_url());
    }

    #[tokio::test]
    async fn process_authorize_builds_wallet_redirect_for_issued_code() {
        let code = AuthorizationCode::from("the-authorization-code".to_string());
        let (issuer, par_store, _sessions) =
            issuer_and_par(vec![(REQUEST_URI.to_string(), vci_request(MOCK_WALLET_CLIENT_ID))]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::IssuedCode(code.clone())),
            allowed_redirect_uris(),
        );

        let redirect_url = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap();

        // The wallet is redirected back to its own redirect_uri, carrying the code and echoed state.
        assert!(redirect_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert_eq!(params.get("code").map(String::as_str), Some(code.as_ref()));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
    }

    #[tokio::test]
    async fn complete_authorization_writes_session_and_builds_redirect() {
        let (issuer, par_store, sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let documents = mock_issuable_documents(NonZeroUsize::MIN);
        let wallet_code_challenge = "wallet-code-challenge".to_string();

        let (code, redirect_url) = authorizing_issuer
            .complete_authorization(
                documents.clone(),
                wallet_code_challenge.clone(),
                WALLET_REDIRECT_URI.parse().unwrap(),
                Some(WALLET_STATE.to_string()),
            )
            .await
            .unwrap();

        // The wallet is redirected back to its own redirect_uri, carrying the generated code and state.
        assert!(redirect_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert_eq!(params.get("code").map(String::as_str), Some(code.as_ref()));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));

        // An AuthCodeIssued session was written, keyed by the generated code, carrying the documents and
        // the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(code))
            .await
            .unwrap()
            .expect("a session should have been written under the generated code");
        let IssuanceData::AuthCodeIssued(auth_code_issued) = session.data else {
            panic!("expected an AuthCodeIssued session");
        };
        assert_eq!(
            auth_code_issued.grant,
            Grant::AuthorizationCode { wallet_code_challenge }
        );
        assert_eq!(auth_code_issued.issuable_documents.len(), documents.len());
    }

    #[tokio::test]
    async fn complete_authorization_omits_state_when_absent() {
        let (issuer, par_store, _sessions) = issuer_and_par(vec![]);
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            allowed_redirect_uris(),
        );

        let (_code, redirect_url) = authorizing_issuer
            .complete_authorization(
                mock_issuable_documents(NonZeroUsize::MIN),
                "wallet-code-challenge".to_string(),
                WALLET_REDIRECT_URI.parse().unwrap(),
                None,
            )
            .await
            .unwrap();

        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert!(params.contains_key("code"));
        assert!(!params.contains_key("state"));
    }
}
