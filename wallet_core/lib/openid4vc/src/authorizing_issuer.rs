//! Authorization Phase wrapper around the Issuance Phase [`Issuer`].
//!
//! [`AuthorizingIssuer`] owns the PAR store and an [`AuthorizationCodeFlow`] implementation.
//! It handles the Pushed Authorization Request and authorize endpoints, and exposes
//! [`complete_authorization`] for the configured flow to call once it has authenticated the holder
//! and determined the issuables. Once `complete_authorization` has
//! written the `AuthCodeIssued` session, the `openid4vc` layer's `/token` on the inner [`Issuer`]) is used in the next
//! step of the flow. Deployments that only do the pre-authorized-code grant (no flow) use the bare [`Issuer`]
//! directly and never construct an [`AuthorizingIssuer`].

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use crypto::utils::random_string;
use derive_more::Constructor;
use itertools::Itertools;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::authorization_code_flow::InvalidAuthorizationRequest;
use crate::authorization_code_flow::WalletAuthorizationContext;
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

    #[error("invalid authorization request: {0}")]
    InvalidAuthorizationRequest(#[source] InvalidAuthorizationRequest),

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
#[derive(Constructor)]
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
            .contains(request.oauth_request.client_id.as_str())
        {
            return Err(ParError::UnknownClient(request.oauth_request.client_id));
        }

        // Exact-match the wallet's redirect_uri against the configured allowlist.
        if !self.wallet_redirect_uris.iter().contains(request.redirect_uri.as_ref()) {
            return Err(ParError::InvalidRedirectUri(request.redirect_uri.into_inner()));
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
    S: SessionStore<IssuanceData>,
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

        // Extract the wallet-side parameters the flow must retain (rejecting an unsupported
        // code_challenge_method here, for every flow at once). Keep the redirect_uri + state so we
        // can build the wallet-facing redirect ourselves on the IssuedCode path.
        let context = WalletAuthorizationContext::try_from_request(authorization_request)
            .map_err(AuthorizeError::InvalidAuthorizationRequest)?;

        let outcome = self
            .flow
            .authorize(context)
            .await
            .map_err(|error| AuthorizeError::AuthorizationCodeFlow(Box::new(error)))?;

        match outcome {
            AuthorizeOutcome::RedirectTo(url) => Ok(url),
            AuthorizeOutcome::Authorized(issuables, context) => self
                .finalize_authorization(
                    issuables,
                    context.code_challenge,
                    WalletRedirect::new(context.redirect_uri, context.state),
                )
                .await
                .map_err(|error| AuthorizeError::AuthorizationCodeFlow(Box::new(error))),
        }
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    S: SessionStore<IssuanceData>,
{
    /// Called by the configured [`AuthorizationCodeFlow`] once it has authenticated the holder and
    /// produced the issuables. How the flow does so (e.g. via an external identity provider) is an
    /// implementation detail of the flow and not modelled here.
    pub async fn complete_authorization(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
        context: WalletAuthorizationContext,
    ) -> Result<Url, CompleteAuthorizationError> {
        let WalletAuthorizationContext {
            redirect_uri,
            state,
            code_challenge,
        } = context;

        self.finalize_authorization(
            issuable_documents,
            code_challenge,
            WalletRedirect::new(redirect_uri, state),
        )
        .await
    }

    /// Generates a fresh authorization code, writes an `AuthCodeIssued` session keyed by it,
    /// and builds the wallet-facing redirect URL with that code and the wallet's original `state`. The
    /// `openid4vc` layer's `/token` handler will later load the session, verify the wallet's
    /// `code_verifier` against the stored challenge, and issue documents.
    ///
    /// Shared by async callback flows ([`Self::complete_authorization`]) and the synchronous
    /// `IssuedCode` path, so both produce identical sessions and redirects.
    async fn finalize_authorization(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
        wallet_code_challenge: String,
        wallet_redirect: WalletRedirect,
    ) -> Result<Url, CompleteAuthorizationError> {
        let code = AuthorizationCode::from(random_string(AUTH_CODE_LENGTH));

        let session_state = SessionState::new(
            code.clone().into(),
            IssuanceData::AuthCodeIssued(Box::new(AuthCodeIssued {
                issuable_documents,
                grant: Grant::AuthorizationCode { wallet_code_challenge },
            })),
        );

        self.issuer
            .write_new_session(session_state)
            .await
            .map_err(CompleteAuthorizationError::SessionStore)?;

        wallet_redirect
            .into_authorization_code_url(&code)
            .map_err(CompleteAuthorizationError::EncodeRedirectQuery)
    }
}

/// The wallet-facing redirect target (`redirect_uri` plus the wallet's original `state`), which can be turned into
/// either a success or an error redirect.
#[derive(Debug, Clone, Constructor)]
pub struct WalletRedirect {
    redirect_uri: Url,
    state: Option<String>,
}

impl WalletRedirect {
    pub fn into_authorization_code_url(self, code: &AuthorizationCode) -> Result<Url, serde_urlencoded::ser::Error> {
        #[derive(Serialize)]
        struct RedirectQuery<'a> {
            code: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            state: Option<&'a str>,
        }

        let Self {
            mut redirect_uri,
            state,
        } = self;

        let query = serde_urlencoded::to_string(RedirectQuery {
            code: code.as_ref(),
            state: state.as_deref(),
        })?;

        redirect_uri.set_query(Some(&query));
        Ok(redirect_uri)
    }

    pub fn into_error_url(self, error: &'static str, error_description: &impl ToString) -> Url {
        #[derive(Serialize)]
        struct RedirectErrorQuery<'a> {
            error: &'a str,
            error_description: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            state: Option<&'a str>,
        }

        let Self {
            mut redirect_uri,
            state,
        } = self;
        let error_description = error_description.to_string();

        let query = serde_urlencoded::to_string(RedirectErrorQuery {
            error,
            error_description: &error_description,
            state: state.as_deref(),
        })
        .expect("encoding wallet error redirect query string should never fail");

        redirect_uri.set_query(Some(&query));
        redirect_uri
    }

    /// Resolve a completion result into the final wallet-facing success or error redirect URL.
    pub fn into_redirect_url(self, result: Result<Url, impl Display>, error_code: &'static str) -> Url {
        result.unwrap_or_else(|error| self.into_error_url(error_code, &error))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use p256::ecdsa::SigningKey;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use url::Url;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::AuthorizeError;
    use super::AuthorizingIssuer;
    use super::ParError;
    use super::WalletRedirect;
    use crate::authorization::PkceCodeChallenge;
    use crate::authorization::VciAuthorizationRequest;
    use crate::authorization_code_flow::AuthorizationCodeFlow;
    use crate::authorization_code_flow::AuthorizeOutcome;
    use crate::authorization_code_flow::InvalidAuthorizationRequest;
    use crate::authorization_code_flow::WalletAuthorizationContext;
    use crate::issuer::Grant;
    use crate::issuer::IssuanceData;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::nonce::memory_store::MemoryNonceStore;
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

    type TestAuthorizingIssuer = AuthorizingIssuer<
        SigningKey,
        MockStatusListService,
        MemorySessionStore<IssuanceData>,
        MemoryNonceStore,
        MemoryStore<String, VciAuthorizationRequest>,
        FixedOutcomeFlow,
    >;

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

    /// Builds an inner [`Issuer`] (accepting only `MOCK_WALLET_CLIENT_ID`) and a PAR store seeded with
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

    /// Builds an [`AuthorizingIssuer`], building on [`issuer_and_par`] and resulting in the provided
    /// [`AuthorizeOutcome`].
    fn create_authorizing_issuer(
        entries: Vec<(String, VciAuthorizationRequest)>,
        outcome: AuthorizeOutcome,
    ) -> (TestAuthorizingIssuer, Arc<MemorySessionStore<IssuanceData>>) {
        let (issuer, par_store, sessions) = issuer_and_par(entries);
        let auth_issuer = AuthorizingIssuer::new(issuer, par_store, FixedOutcomeFlow(outcome), allowed_redirect_uris());
        (auth_issuer, sessions)
    }

    /// Minimal [`AuthorizationCodeFlow`] that returns a preconfigured outcome, so `process_authorize`
    /// can be exercised independently of any real flow's behaviour.
    #[derive(Debug, Clone)]
    struct FixedOutcomeFlow(AuthorizeOutcome);

    impl AuthorizationCodeFlow for FixedOutcomeFlow {
        type Error = Infallible;

        async fn authorize(&self, _context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn process_par_rejects_unknown_client_id() {
        let (authorizing_issuer, _sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

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
        let (authorizing_issuer, _sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

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
        let (authorizing_issuer, _sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, OTHER_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(error, AuthorizeError::UnknownClient(client_id) if client_id == OTHER_CLIENT_ID);
    }

    #[tokio::test]
    async fn process_authorize_rejects_unknown_request_uri() {
        // Caller is accepted, but the PAR store is empty, so the request_uri can't be resolved.
        let (authorizing_issuer, _sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(error, AuthorizeError::UnknownRequestUri(request_uri) if request_uri == REQUEST_URI);
    }

    #[tokio::test]
    async fn process_authorize_rejects_mismatched_client_id() {
        // The PAR was pushed under a different client_id than the one presented at /authorize.
        let (authorizing_issuer, _sessions) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), vci_request(OTHER_CLIENT_ID))],
            AuthorizeOutcome::RedirectTo(upstream_url()),
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
    async fn process_authorize_rejects_unsupported_code_challenge_method() {
        // The PAR holds a request with a `plain` code_challenge_method, which we don't support.
        let mut request = vci_request(MOCK_WALLET_CLIENT_ID);
        request.code_challenge = PkceCodeChallenge::Plain {
            code_challenge: "plain-challenge".to_string(),
        };
        let (authorizing_issuer, _sessions) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), request)],
            AuthorizeOutcome::RedirectTo(upstream_url()),
        );

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(
            error,
            AuthorizeError::InvalidAuthorizationRequest(InvalidAuthorizationRequest::UnsupportedCodeChallenge)
        );
    }

    #[tokio::test]
    async fn process_authorize_passes_through_redirect_outcome() {
        let (authorizing_issuer, _sessions) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), vci_request(MOCK_WALLET_CLIENT_ID))],
            AuthorizeOutcome::RedirectTo(upstream_url()),
        );

        let redirect_url = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap();

        assert_eq!(redirect_url, upstream_url());
    }

    #[tokio::test]
    async fn process_authorize_builds_wallet_redirect_for_issued_code() {
        let documents = mock_issuable_documents(NonZeroUsize::MIN);
        let wallet_code_challenge = "wallet-code-challenge".to_string();
        let (authorizing_issuer, sessions) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), vci_request(MOCK_WALLET_CLIENT_ID))],
            AuthorizeOutcome::Authorized(
                documents.clone(),
                WalletAuthorizationContext {
                    redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
                    state: Some(WALLET_STATE.to_string()),
                    code_challenge: wallet_code_challenge.clone(),
                },
            ),
        );

        let redirect_url = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap();

        // The wallet is redirected back to its own redirect_uri, carrying a freshly generated code and
        // the echoed state.
        assert!(redirect_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert!(params.contains_key("code"));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));

        let authorization_code: AuthorizationCode = params.get("code").unwrap().clone().into();

        // An AuthCodeIssued session was written, keyed by the generated code, carrying the documents and
        // the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(authorization_code))
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
    async fn complete_authorization_writes_session_and_builds_redirect() {
        let (authorizing_issuer, sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let documents = mock_issuable_documents(NonZeroUsize::MIN);
        let wallet_code_challenge = "wallet-code-challenge".to_string();

        let redirect_url = authorizing_issuer
            .complete_authorization(
                documents.clone(),
                WalletAuthorizationContext {
                    redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
                    state: Some(WALLET_STATE.to_string()),
                    code_challenge: wallet_code_challenge.clone(),
                },
            )
            .await
            .unwrap();

        // The wallet is redirected back to its own redirect_uri, carrying the generated code and state.
        assert!(redirect_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert!(params.contains_key("code"));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));

        let authorization_code: AuthorizationCode = params.get("code").unwrap().clone().into();

        // An AuthCodeIssued session was written, keyed by the generated code, carrying the documents and
        // the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(authorization_code))
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
        let (authorizing_issuer, _sessions) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let redirect_url = authorizing_issuer
            .complete_authorization(
                mock_issuable_documents(NonZeroUsize::MIN),
                WalletAuthorizationContext {
                    redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
                    state: None,
                    code_challenge: "wallet-code-challenge".to_string(),
                },
            )
            .await
            .unwrap();

        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert!(params.contains_key("code"));
        assert!(!params.contains_key("state"));
    }

    #[test]
    fn into_redirect_url_passes_success_url_through() {
        let success_url: Url = format!("{WALLET_REDIRECT_URI}?code=the-code&state={WALLET_STATE}")
            .parse()
            .unwrap();
        let wallet_redirect = WalletRedirect::new(WALLET_REDIRECT_URI.parse().unwrap(), Some(WALLET_STATE.to_string()));

        let url = wallet_redirect.into_redirect_url(Ok::<_, Infallible>(success_url.clone()), "server_error");

        assert_eq!(url, success_url);
    }

    #[test]
    fn into_redirect_url_builds_error_redirect_on_failure() {
        let wallet_redirect = WalletRedirect::new(WALLET_REDIRECT_URI.parse().unwrap(), Some(WALLET_STATE.to_string()));

        let url = wallet_redirect.into_redirect_url(Err::<Url, _>("something broke"), "server_error");

        assert!(url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("error").map(String::as_str), Some("server_error"));
        assert_eq!(
            params.get("error_description").map(String::as_str),
            Some("something broke")
        );
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
    }
}
