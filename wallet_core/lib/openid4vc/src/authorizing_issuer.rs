//! Authorization Phase wrapper around the Issuance Phase [`Issuer`].
//!
//! [`AuthorizingIssuer`] owns the PAR store and an [`AuthorizationCodeFlow`] implementation.
//! It handles the Pushed Authorization Request and authorize endpoints, and exposes
//! [`complete_authorization`] for the configured flow to call once it has authenticated the holder
//! and determined the issuables. Once `complete_authorization` has
//! written the `AuthCodeIssued` session, the `openid4vc` layer's `/token` on the inner [`Issuer`]) is used in the next
//! step of the flow. Deployments that only do the pre-authorized-code grant (no flow) use the bare [`Issuer`]
//! directly and never construct an [`AuthorizingIssuer`].

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use derive_more::Constructor;
use futures::join;
use itertools::Itertools;
use jwt::wia::WiaDisclosure;
use jwt::wia::WiaError;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::AuthorizationErrorCode;
use crate::BoxedErrorWithCode;
use crate::RedirectError;
use crate::authorization::PushedAuthorizationResponse;
use crate::authorization::VciAuthorizationRequest;
use crate::authorization_code_flow::AuthorizationCodeFlow;
use crate::authorization_code_flow::AuthorizeOutcome;
use crate::authorization_code_flow::InvalidAuthorizationRequest;
use crate::authorization_code_flow::WalletAuthorizationContext;
use crate::cleanup::PeriodicCleanup;
use crate::cleanup::log_cleanup_error;
use crate::credential_offer::CredentialOffer;
use crate::issuable_document::IssuableDocument;
use crate::issuer::AuthCodeIssued;
use crate::issuer::AuthRequestValues;
use crate::issuer::Grant;
use crate::issuer::IssuableDocumentError;
use crate::issuer::IssuanceData;
use crate::issuer::Issuer;
use crate::nonce::store::NonceStore;
use crate::par;
use crate::par::PAR_TTL;
use crate::scope::Scope;
use crate::server_state::SessionStore;
use crate::server_state::SessionStoreError;
use crate::store::Store;
use crate::token::AuthorizationCode;

/// Errors that can occur during processing of a Pushed Authorization Request.
#[derive(Debug, thiserror::Error)]
pub enum ParError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("error verifying WIA: {0}")]
    Wia(#[source] WiaError),

    #[error("a PAR containing authorization_details is not supported")]
    AuthorizationDetailsUnsupported,

    #[error("redirect_uri not allowed: {0}")]
    InvalidRedirectUri(Url),

    #[error("storing PAR request failed: {0}")]
    Store(#[source] Box<dyn Error + Send + Sync + 'static>),
}

/// Errors that can occur during calls to the `authorize` endpoint, before the PAR has been retrieved from storage.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("unknown client_id: {0}")]
    UnknownClient(String),

    #[error("consuming PAR request failed: {0}")]
    ParStore(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("request_uri not found or expired: {0}")]
    UnknownRequestUri(String),

    #[error("expected client_id from Authorization Request: {expected}, but was: {actual}")]
    MismatchedClient { expected: String, actual: String },

    #[error("{0}")]
    AuthorizationRequest(RedirectError<AuthorizationRequestError>),
}

/// Errors that can occur during calls to the `authorize` endpoint, after the PAR has been retrieved from storage and
/// the `redirect_uri` is known.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationRequestError {
    #[error("invalid authorization request: {0}")]
    InvalidAuthorizationRequest(#[source] InvalidAuthorizationRequest),

    #[error("none of the scopes requested reference a known credential configuration: {}", .0.iter().join(" "))]
    NoValidScope(HashSet<Scope>),

    #[error("authorization code flow error: {0}")]
    AuthorizationCodeFlow(#[source] BoxedErrorWithCode<AuthorizationErrorCode>),

    #[error("error completing authorization for the authorized outcome: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteAuthorizationError {
    #[error("issuable document is not valid: {0}")]
    IssuableDocument(#[source] IssuableDocumentError),

    #[error("failed to store new session: {0}")]
    SessionStore(#[source] SessionStoreError),
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
            .cloned()
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

impl<K, L, S, N, PAS, AF> PeriodicCleanup for AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    K: Send + Sync,
    L: Send + Sync,
    S: SessionStore<IssuanceData> + Send + Sync,
    N: NonceStore + Send + Sync,
    PAS: Store<String, VciAuthorizationRequest> + Send + Sync,
    AF: AuthorizationCodeFlow + Send + Sync,
{
    /// Removes expired entries from every store beneath this authorization-phase issuer.
    ///
    /// Cleans the stores it owns directly (the PAR store and the flow's own storage) plus those of the inner
    /// [`Issuer`] (sessions and proof nonces). Scheduled by the server via
    /// [`start_cleanup_task`](crate::cleanup::start_cleanup_task).
    async fn cleanup(&self) {
        let _ = join!(
            self.issuer.cleanup(),
            log_cleanup_error("PAR store", self.par_store.cleanup()),
            log_cleanup_error("authorization-code flow", self.flow.cleanup()),
        );
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    PAS: Store<String, VciAuthorizationRequest>,
{
    pub async fn process_pushed_authorization_request(
        &self,
        request: VciAuthorizationRequest,
        wia_disclosure: &WiaDisclosure,
    ) -> Result<PushedAuthorizationResponse, ParError> {
        if !self
            .issuer
            .accepted_wallet_client_ids()
            .contains(request.oauth_request.client_id.as_str())
        {
            return Err(ParError::UnknownClient(request.oauth_request.client_id));
        }

        // `verify_wia()` checks that the WIA's `sub` matches `client_id`.
        // <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-7.1-2.7.1>
        self.issuer
            .verify_wia(wia_disclosure, Some(&request.oauth_request.client_id))
            .map_err(ParError::Wia)?;

        if request.authorization_details.is_some() {
            return Err(ParError::AuthorizationDetailsUnsupported);
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
        if !self.issuer.accepted_wallet_client_ids().contains(client_id) {
            return Err(AuthorizeError::UnknownClient(client_id.to_string()));
        }

        let authorization_request = self
            .par_store
            .consume(request_uri)
            .await
            .map_err(|error| AuthorizeError::ParStore(Box::new(error)))?
            .live()
            .ok_or_else(|| AuthorizeError::UnknownRequestUri(request_uri.to_string()))?;

        if authorization_request.oauth_request.client_id != client_id {
            return Err(AuthorizeError::MismatchedClient {
                expected: authorization_request.oauth_request.client_id,
                actual: client_id.to_string(),
            });
        }

        let url = self
            .process_stored_authorization_request(authorization_request)
            .await
            .map_err(AuthorizeError::AuthorizationRequest)?;

        Ok(url)
    }

    /// Process a PAR that was retrieved from storage. Any error is returned along with the `redirect_uri` and optional
    /// `state` from the PAR, so that the error can be added to the `redirect_uri` and returned to the wallet.
    ///
    /// Note that this should be a 303 (See Other) redirect, the semantics of which are better defined than the older
    /// 302 (Found) status code. Additionally, for OAuth status code 303 seems a little more approprate than 307
    /// (Temporary Redirect), as this forces the client to make a GET request and not re-post any body.
    async fn process_stored_authorization_request(
        &self,
        authorization_request: VciAuthorizationRequest,
    ) -> Result<Url, RedirectError<AuthorizationRequestError>> {
        let redirect_uri = authorization_request.redirect_uri.as_ref().clone();
        let state = authorization_request.oauth_request.state.clone();

        // Extract the wallet-side parameters the flow must retain (rejecting an unsupported
        // code_challenge_method here, for every flow at once). Keep the redirect_uri + state so we
        // can build the wallet-facing redirect ourselves on the IssuedCode path.
        let context =
            match WalletAuthorizationContext::try_from_request(authorization_request, self.issuer.credential_configs())
            {
                Ok(context) => context,
                Err(error) => {
                    return Err(RedirectError::new(
                        AuthorizationRequestError::InvalidAuthorizationRequest(error),
                        redirect_uri,
                        state,
                    ));
                }
            };

        let outcome = match self.flow.authorize(context).await {
            Ok(outcome) => outcome,
            Err(error) => {
                return Err(RedirectError::new(
                    AuthorizationRequestError::AuthorizationCodeFlow(BoxedErrorWithCode::new(error)),
                    redirect_uri,
                    state,
                ));
            }
        };

        match outcome {
            AuthorizeOutcome::RedirectTo(url) => Ok(url),
            AuthorizeOutcome::Authorized(issuables, context) => {
                let WalletAuthorizationContext {
                    state, request_values, ..
                } = *context;

                let code = match self.complete_authorization(issuables, request_values).await {
                    Ok(code) => code,
                    Err(error) => {
                        return Err(RedirectError::new(
                            AuthorizationRequestError::CompleteAuthorization(error),
                            redirect_uri,
                            state,
                        ));
                    }
                };

                let url = RedirectQuery::encode(redirect_uri, &code, state.as_deref());

                Ok(url)
            }
        }
    }
}

impl<K, L, S, N, PAS, AF> AuthorizingIssuer<K, L, S, N, PAS, AF>
where
    S: SessionStore<IssuanceData>,
{
    /// Called by the configured [`AuthorizationCodeFlow`] once it has authenticated the holder and
    /// produced the issuables. Generates a fresh authorization code, writes an `AuthCodeIssued`
    /// session keyed by it, and returns the code. The caller is responsible for building the
    /// wallet-facing redirect URL from the returned code.
    pub async fn complete_authorization(
        &self,
        issuable_documents: VecNonEmpty<IssuableDocument>,
        auth_request_values: AuthRequestValues,
    ) -> Result<AuthorizationCode, CompleteAuthorizationError> {
        let credential_ids_and_documents = self
            .issuer
            .validate_issuable_documents(issuable_documents)
            .map_err(CompleteAuthorizationError::IssuableDocument)?;

        let token = self
            .issuer
            .write_auth_code_issued_session(AuthCodeIssued {
                grant: Grant::AuthorizationCode(auth_request_values),
                credential_ids_and_documents,
            })
            .await
            .map_err(CompleteAuthorizationError::SessionStore)?;

        Ok(token.into())
    }
}

/// Represents the contents of the query parameters of a successful redirect back to the wallet, i.e. the `code` and
/// optional `state` parameter, echoing the `state` from the Authorization Request.
#[derive(Debug)]
pub struct RedirectQuery<'a> {
    code: &'a str,
    state: Option<&'a str>,
}

impl<'a> RedirectQuery<'a> {
    pub fn encode(redirect_uri: Url, code: &'a AuthorizationCode, state: Option<&'a str>) -> Url {
        let query = Self {
            code: code.as_ref(),
            state,
        };

        query.append_to_uri(redirect_uri)
    }

    fn append_to_uri(&self, mut redirect_uri: Url) -> Url {
        {
            let mut query_pairs = redirect_uri.query_pairs_mut();

            query_pairs.append_pair("code", self.code);

            if let Some(state) = self.state {
                query_pairs.append_pair("state", state);
            }
        }

        redirect_uri
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::convert::Infallible;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use attestation_types::credential_format::Format;
    use crypto::server_keys::KeyPair;
    use futures::FutureExt;
    use jwt::wia::WiaDisclosure;
    use p256::ecdsa::SigningKey;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use url::Url;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;
    use wscd::mock_remote::MockWiaClient;
    use wscd::wscd::WiaClient;

    use super::AuthorizationRequestError;
    use super::AuthorizeError;
    use super::AuthorizingIssuer;
    use super::ParError;
    use super::RedirectQuery;
    use crate::AuthorizationErrorCode;
    use crate::ErrorWithCode;
    use crate::RedirectError;
    use crate::authorization::VciAuthorizationRequest;
    use crate::authorization_code_flow::AuthorizationCodeFlow;
    use crate::authorization_code_flow::AuthorizeOutcome;
    use crate::authorization_code_flow::InvalidAuthorizationRequest;
    use crate::authorization_code_flow::WalletAuthorizationContext;
    use crate::authorization_details::AuthorizationDetailsEntry;
    use crate::issuable_document::CredentialKind;
    use crate::issuer::AuthRequestValues;
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
    use crate::test::MOCK_ATTESTATION_TYPES;
    use crate::test::MockIssuer;
    use crate::test::mock_issuable_documents;
    use crate::test::setup_mock_issuer;
    use crate::token::AuthorizationCode;

    const OTHER_CLIENT_ID: &str = "definitely-not-the-wallet";
    const REQUEST_URI: &str = "urn:ietf:params:oauth:request_uri:test";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";
    const WALLET_STATE: &str = "wallet-state";
    // Match the credential config id / scope configured in the mock issuer.
    const WALLET_SCOPE: &str = "com.example.pid_dc+sd-jwt";

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
            HashSet::from([WALLET_SCOPE.parse().unwrap()]),
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
        KeyPair,
    ) {
        let issuer_identifier = IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap();
        let sessions = Arc::new(MemorySessionStore::default());
        let (issuer, _, wia_keypair) = setup_mock_issuer(
            issuer_identifier,
            MOCK_ATTESTATION_TYPES.len().try_into().unwrap(),
            Arc::clone(&sessions),
        );

        let par_store = MemoryStore::new(PAR_TTL);
        for (request_uri, request) in entries {
            par_store.store_inner(request_uri, request);
        }

        (Arc::new(issuer), par_store, sessions, wia_keypair)
    }

    /// Builds an [`AuthorizingIssuer`], building on [`issuer_and_par`] and resulting in the provided
    /// [`AuthorizeOutcome`].
    fn create_authorizing_issuer(
        entries: Vec<(String, VciAuthorizationRequest)>,
        outcome: AuthorizeOutcome,
    ) -> (TestAuthorizingIssuer, Arc<MemorySessionStore<IssuanceData>>, KeyPair) {
        let (issuer, par_store, sessions, wia_keypair) = issuer_and_par(entries);
        let auth_issuer = AuthorizingIssuer::new(issuer, par_store, FixedOutcomeFlow(outcome), allowed_redirect_uris());
        (auth_issuer, sessions, wia_keypair)
    }

    /// Minimal [`AuthorizationCodeFlow`] that returns a preconfigured outcome, so `process_authorize`
    /// can be exercised independently of any real flow's behaviour.
    #[derive(Debug, Clone)]
    struct FixedOutcomeFlow(AuthorizeOutcome);

    #[derive(Debug, thiserror::Error)]
    #[error(transparent)]
    struct FixedOutcomeFlowError(Infallible);

    impl ErrorWithCode for FixedOutcomeFlowError {
        type ErrorCode = AuthorizationErrorCode;

        fn error_code(&self) -> Self::ErrorCode {
            unreachable!()
        }
    }

    impl AuthorizationCodeFlow for FixedOutcomeFlow {
        type Error = FixedOutcomeFlowError;

        async fn authorize(&self, _context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn process_par_stores_request_and_returns_response() {
        let (authorizing_issuer, _sessions, wia_keypair) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let response = authorizing_issuer
            .process_pushed_authorization_request(
                vci_request(MOCK_WALLET_CLIENT_ID),
                &MockWiaClient::new_with_wia_keypair(wia_keypair)
                    .issue_wia(authorizing_issuer.issuer.issuer_identifier().to_string(), None)
                    .await
                    .unwrap(),
            )
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
            .live()
            .expect("request should be stored under the returned request_uri");
        assert_eq!(stored.oauth_request.client_id, MOCK_WALLET_CLIENT_ID);
    }

    #[tokio::test]
    async fn process_par_rejects_unknown_client_id() {
        let (authorizing_issuer, _sessions, _) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let error = authorizing_issuer
            .process_pushed_authorization_request(
                vci_request(OTHER_CLIENT_ID),
                &WiaDisclosure::new("a.b.c".parse().unwrap(), "a.b.c".parse().unwrap()),
            )
            .await
            .unwrap_err();

        assert_matches!(error, ParError::UnknownClient(client_id) if client_id == OTHER_CLIENT_ID);
        // The rejected request must not have been stored.
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_par_rejects_authorization_details() {
        let (authorizing_issuer, _sessions, wia_keypair) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let mut request = vci_request(MOCK_WALLET_CLIENT_ID);
        request.authorization_details = Some(
            vec_nonempty![AuthorizationDetailsEntry::new_vci(
                "credential_config_id".to_string().into()
            )]
            .try_into()
            .unwrap(),
        );

        let wia = MockWiaClient::new_with_wia_keypair(wia_keypair)
            .issue_wia(authorizing_issuer.issuer.issuer_identifier().to_string(), None)
            .now_or_never()
            .unwrap()
            .unwrap();

        let error = authorizing_issuer
            .process_pushed_authorization_request(request, &wia)
            .await
            .unwrap_err();

        assert_matches!(error, ParError::AuthorizationDetailsUnsupported);
        // The rejected request must not have been stored.
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_par_rejects_disallowed_redirect_uri() {
        let (issuer, par_store, _sessions, wia_keypair) = issuer_and_par(vec![]);
        // The client_id is accepted, but the allowlist does not contain WALLET_REDIRECT_URI, so the
        // request must be rejected on the redirect_uri check alone.
        let authorizing_issuer = AuthorizingIssuer::new(
            issuer,
            par_store,
            FixedOutcomeFlow(AuthorizeOutcome::RedirectTo(upstream_url())),
            vec_nonempty!["https://other.example.com/callback".parse().unwrap()],
        );

        let error = authorizing_issuer
            .process_pushed_authorization_request(
                vci_request(MOCK_WALLET_CLIENT_ID),
                &MockWiaClient::new_with_wia_keypair(wia_keypair)
                    .issue_wia(authorizing_issuer.issuer.issuer_identifier().to_string(), None)
                    .await
                    .unwrap(),
            )
            .await
            .unwrap_err();

        assert_matches!(error, ParError::InvalidRedirectUri(uri) if uri.as_str() == WALLET_REDIRECT_URI);
        // The rejected request must not have been stored.
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_par_rejects_wia_from_untrusted_issuer() {
        let (authorizing_issuer, _sessions, _wia_keypair) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        // MockWiaClient::new() issues a WIA signed by a freshly generated CA that is not in the
        // issuer's trust anchors, so verification must fail.
        let wia = MockWiaClient::new()
            .issue_wia(authorizing_issuer.issuer.issuer_identifier().to_string(), None)
            .await
            .unwrap();

        let error = authorizing_issuer
            .process_pushed_authorization_request(vci_request(MOCK_WALLET_CLIENT_ID), &wia)
            .await
            .unwrap_err();

        assert_matches!(error, ParError::Wia(_));
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_par_rejects_wia_with_wrong_audience() {
        let (authorizing_issuer, _sessions, wia_keypair) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        // The WIA is signed by the trusted key pair but targets a different audience, so the
        // audience check inside verify_wia must reject it.
        let wia = MockWiaClient::new_with_wia_keypair(wia_keypair)
            .issue_wia("https://wrong-issuer.example.com".to_string(), None)
            .await
            .unwrap();

        let error = authorizing_issuer
            .process_pushed_authorization_request(vci_request(MOCK_WALLET_CLIENT_ID), &wia)
            .await
            .unwrap_err();

        assert_matches!(error, ParError::Wia(_));
        assert!(authorizing_issuer.par_store.is_empty());
    }

    #[tokio::test]
    async fn process_authorize_rejects_unknown_client_id() {
        let (authorizing_issuer, _sessions, _) =
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
        let (authorizing_issuer, _sessions, _) =
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
        let (authorizing_issuer, _sessions, _) = create_authorizing_issuer(
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
    async fn process_authorize_propagates_invalid_authorization_request() {
        let mut request = vci_request(MOCK_WALLET_CLIENT_ID);
        request.scope = HashSet::from(["scope1".parse().unwrap(), "scope2".parse().unwrap()]);
        let (authorizing_issuer, _sessions, _) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), request)],
            AuthorizeOutcome::RedirectTo(upstream_url()),
        );

        let error = authorizing_issuer
            .process_authorize(REQUEST_URI, MOCK_WALLET_CLIENT_ID)
            .await
            .unwrap_err();

        assert_matches!(
            error,
            AuthorizeError::AuthorizationRequest(
                RedirectError {
                error: AuthorizationRequestError::InvalidAuthorizationRequest(
                    InvalidAuthorizationRequest::NoValidScope(scope),
                ),
                redirect_uri,
                state: Some(state),
            }
            ) if scope == HashSet::from(["scope1".parse().unwrap(), "scope2".parse().unwrap()]) &&
                redirect_uri.as_str() == WALLET_REDIRECT_URI &&
                state == WALLET_STATE
        );
    }

    #[tokio::test]
    async fn process_authorize_passes_through_redirect_outcome() {
        let (authorizing_issuer, _sessions, _) = create_authorizing_issuer(
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
        let (authorizing_issuer, sessions, _) = create_authorizing_issuer(
            vec![(REQUEST_URI.to_string(), vci_request(MOCK_WALLET_CLIENT_ID))],
            AuthorizeOutcome::Authorized(
                documents.clone(),
                Box::new(WalletAuthorizationContext {
                    state: Some(WALLET_STATE.to_string()),
                    credential_kinds: HashSet::from_iter([CredentialKind::new(
                        Format::SdJwt,
                        String::from(MOCK_ATTESTATION_TYPES[0]),
                    )]),
                    request_values: AuthRequestValues::new(
                        MOCK_WALLET_CLIENT_ID.to_string(),
                        WALLET_REDIRECT_URI.parse().unwrap(),
                        WALLET_CODE_CHALLENGE.to_string(),
                        HashSet::from([WALLET_SCOPE.parse().unwrap()]),
                    ),
                    issuer_state: None,
                }),
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
        assert_matches!(
            auth_code_issued.grant,
            Grant::AuthorizationCode(
                AuthRequestValues {
                    client_id,
                    redirect_uri,
                    code_challenge,
                    scope,
            }) if client_id == MOCK_WALLET_CLIENT_ID
                && redirect_uri.as_str() == WALLET_REDIRECT_URI
                && code_challenge == WALLET_CODE_CHALLENGE
                && scope == HashSet::from([WALLET_SCOPE.parse().unwrap()])
        );
        assert_eq!(auth_code_issued.credential_ids_and_documents.len(), documents.len());
    }

    #[tokio::test]
    async fn complete_authorization_writes_session_keyed_by_returned_code() {
        let (authorizing_issuer, sessions, _) =
            create_authorizing_issuer(vec![], AuthorizeOutcome::RedirectTo(upstream_url()));

        let documents = mock_issuable_documents(NonZeroUsize::MIN);

        let code = authorizing_issuer
            .complete_authorization(
                documents.clone(),
                AuthRequestValues::new(
                    MOCK_WALLET_CLIENT_ID.to_string(),
                    WALLET_REDIRECT_URI.parse().unwrap(),
                    WALLET_CODE_CHALLENGE.to_string(),
                    HashSet::from([WALLET_SCOPE.parse().unwrap()]),
                ),
            )
            .await
            .unwrap();

        // An AuthCodeIssued session was written, keyed by the returned code, carrying the documents and
        // the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(code))
            .await
            .unwrap()
            .expect("a session should have been written under the returned code");
        let IssuanceData::AuthCodeIssued(auth_code_issued) = session.data else {
            panic!("expected an AuthCodeIssued session");
        };
        assert_matches!(
            auth_code_issued.grant,
            Grant::AuthorizationCode(
                AuthRequestValues {
                    client_id,
                    redirect_uri,
                    code_challenge,
                    scope,
            }) if client_id == MOCK_WALLET_CLIENT_ID
                && redirect_uri.as_str() == WALLET_REDIRECT_URI
                && code_challenge == WALLET_CODE_CHALLENGE
                && scope == HashSet::from([WALLET_SCOPE.parse().unwrap()])
        );
        assert_eq!(auth_code_issued.credential_ids_and_documents.len(), documents.len());
    }

    #[test]
    fn redirect_query_encode_omits_state_when_absent() {
        let code = AuthorizationCode::from("the-code".to_string());

        let url = RedirectQuery::encode(WALLET_REDIRECT_URI.parse().unwrap(), &code, None);

        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert!(params.contains_key("code"));
        assert!(!params.contains_key("state"));
    }

    #[test]
    fn redirect_query_encode_includes_state_when_present() {
        let code = AuthorizationCode::from("the-code".to_string());

        let url = RedirectQuery::encode(WALLET_REDIRECT_URI.parse().unwrap(), &code, Some(WALLET_STATE));

        assert!(url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("code").map(String::as_str), Some("the-code"));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
    }
}
