use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesHandlingError;
use attestation_types::claim_path::ClaimPath;
use attestation_types::credential_format::Format;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use crypto::utils::random_string;
use hsm::service::HsmError;
use http_utils::urls::BaseUrl;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use jwk_simple::Key;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
use openid4vc::authorizing_issuer::WalletRedirect;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::IssuanceData;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Store;
use openid4vc::token::AuthorizationCode;
use serde::Deserialize;
use serde::Serialize;
use server_utils::keys::SecretKeyVariant;
use server_utils::store::StoreConnection;
use tracing::warn;
use url::Url;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::client::HttpBrpClient;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_RECOVERY_CODE;
use crate::pid::digid;
use crate::pid::digid::DigidClient;
use crate::pid::digid::DigidMetadataCache;
use crate::pid::digid::HttpDigidClient;

const ISSUER_STATE_LENGTH: usize = 32;

const DIGID_CALLBACK_PATH: &str = "/digid/callback";

/// Errors raised by [`UpstreamOidcAuthorizationCodeFlow`] on either half of the flow.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[source] digid::Error),

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

    #[error("could not find attributes for BSN")]
    NoAttributesFound,

    #[error("error retrieving from BRP: {0}")]
    Brp(#[source] BrpError),

    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,

    #[error("could not find BSN attribute")]
    NoBsnFound,

    #[error("error retrieving BSN: {0}")]
    RetrievingBsn(#[source] AttributesHandlingError),

    #[error("BSN attribute had unexpected type (expected string)")]
    BsnUnexpectedType,

    #[error("failed to compute BSN HMAC: {0}")]
    Hmac(#[source] HsmError),

    #[error("error inserting recovery code: {0}")]
    InsertingRecoveryCode(#[source] AttributesHandlingError),

    #[error("error completing authorization: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

/// One state-bridge entry, written at `/authorize` and consumed by the upstream callback handler.
///
/// Linked to the upstream provider by the `issuer_state` random string we send as `state` in the
/// upstream redirect, which the upstream then echoes back to our callback. Carries everything the
/// callback needs to (a) complete the upstream `/token` + `/userinfo` exchange and (b) build the
/// wallet-facing redirect.
#[derive(Serialize, Deserialize)]
struct StateBridgeEntry {
    context: WalletAuthorizationContext,
    upstream_code_verifier: String,
}

/// Query parameters sent by the upstream provider when redirecting the user back to the issuer's
/// callback URL after a successful authentication.
#[derive(Deserialize)]
struct DigidCallbackQuery {
    code: AuthorizationCode,
    state: String,
}

/// Concrete [`AuthorizationCodeFlow`] for the pid_issuer's upstream-OIDC (DigiD) flow.
///
/// Owns:
/// - upstream OIDC discovery cache + client (for the authorize-endpoint URL and the `/userinfo`-based BSN exchange);
/// - the state-bridge store linking the issuer-generated `issuer_state` (sent to the upstream as `state`) to the
///   wallet's original `redirect_uri`, `state`, PKCE challenge and our upstream PKCE verifier;
/// - the BRP client (BSN → person attributes) and the recovery-code HMAC key;
/// - the issuer's own callback URL, used both as the upstream `redirect_uri` and as the `redirect_uri` parameter of the
///   upstream `/token` exchange.
pub struct UpstreamOidcAuthorizationCodeFlow<B = HttpBrpClient, O = HttpDigidClient> {
    brp_client: B,
    digid_client: O,
    recovery_code_secret_key: SecretKeyVariant,
    state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
    callback_url: Url,
    client_id: String,
}

impl UpstreamOidcAuthorizationCodeFlow {
    #[expect(clippy::too_many_arguments, reason = "Constructor wiring upstream-OIDC dependencies")]
    pub fn try_new(
        brp_client: HttpBrpClient,
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        digid_metadata_cache: DigidMetadataCache,
        recovery_code_secret_key: SecretKeyVariant,
        store_connection: StoreConnection,
        callback_base_url: &BaseUrl,
    ) -> Result<Self, Error> {
        let client_id: String = client_id.into();
        let digid_client =
            HttpDigidClient::try_new(bsn_privkey, client_id.clone(), digid_metadata_cache).map_err(Error::Digid)?;

        Ok(Self::new(
            brp_client,
            digid_client,
            recovery_code_secret_key,
            store_connection,
            callback_base_url,
            client_id,
        ))
    }
}

impl<B, O> UpstreamOidcAuthorizationCodeFlow<B, O> {
    /// Construct the flow, building its [`IssuerStateBridgeStore`] from `store_connection`. The
    /// store's entry type is an internal detail of this flow, so callers only supply the connection.
    pub fn new(
        brp_client: B,
        digid_client: O,
        recovery_code_secret_key: SecretKeyVariant,
        store_connection: StoreConnection,
        callback_base_url: &BaseUrl,
        client_id: String,
    ) -> Self {
        Self::new_with_store(
            brp_client,
            digid_client,
            recovery_code_secret_key,
            Arc::new(IssuerStateBridgeStore::new(store_connection)),
            callback_base_url,
            client_id,
        )
    }

    fn new_with_store(
        brp_client: B,
        digid_client: O,
        recovery_code_secret_key: SecretKeyVariant,
        state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
        callback_base_url: &BaseUrl,
        client_id: String,
    ) -> Self {
        Self {
            brp_client,
            digid_client,
            recovery_code_secret_key,
            state_bridge_store,
            callback_url: callback_base_url.join(DIGID_CALLBACK_PATH),
            client_id,
        }
    }

    /// Mount the `/digid/callback` route owned by this flow on a fresh [`Router`]. The
    /// pid_issuer's `server` module merges this with the `openid4vc` layer's authorization and
    /// issuance routers. The handler reads its flow state via [`AuthorizingIssuer::flow`].
    pub fn callback_router<K, L, S, N, PAS>(
        authorizing_issuer: DigidCallbackAuthorizingIssuer<K, L, S, N, PAS, B, O>,
    ) -> Router
    where
        K: Send + Sync + 'static,
        L: Send + Sync + 'static,
        S: SessionStore<IssuanceData> + Send + Sync + 'static,
        N: Send + Sync + 'static,
        PAS: Send + Sync + 'static,
        B: BrpClient + Send + Sync + 'static,
        O: DigidClient + Send + Sync + 'static,
    {
        Router::new()
            .route(DIGID_CALLBACK_PATH, get(digid_callback))
            .with_state(authorizing_issuer)
    }

    /// Exchange the upstream `code` for the user's BSN, look up attributes, build and return the issuable documents.
    async fn fetch_issuable_documents(
        &self,
        upstream_code: AuthorizationCode,
        upstream_code_verifier: String,
    ) -> Result<VecNonEmpty<IssuableDocument>, Error>
    where
        B: BrpClient,
        O: DigidClient,
    {
        let bsn = self
            .digid_client
            .bsn(upstream_code, upstream_code_verifier, self.callback_url.clone())
            .await
            .map_err(Error::Digid)?;

        let mut persons = self.brp_client.get_person_by_bsn(&bsn).await.map_err(Error::Brp)?;
        if persons.persons.len() != 1 {
            return Err(Error::NoAttributesFound);
        }
        let person = persons.persons.remove(0);
        let attributes = insert_recovery_code(person.into_attributes(), &self.recovery_code_secret_key).await?;

        let issuable_documents = vec_nonempty![
            IssuableDocument::try_new_with_random_id(
                Format::SdJwt,
                PID_ATTESTATION_TYPE.to_string(),
                attributes.clone(),
            )
            .map_err(|_| Error::InvalidIssuableDocuments)?,
            IssuableDocument::try_new_with_random_id(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string(), attributes)
                .map_err(|_| Error::InvalidIssuableDocuments)?,
        ];

        Ok(issuable_documents)
    }
}

impl<B, O> AuthorizationCodeFlow for UpstreamOidcAuthorizationCodeFlow<B, O>
where
    B: BrpClient + Send + Sync,
    O: DigidClient + Send + Sync,
{
    type Error = Error;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        // Generate the upstream PKCE pair and the random `issuer_state` we'll use as `state` in
        // the upstream redirect. The upstream provider will echo it back to our callback.
        let upstream_pkce = S256PkcePair::generate();
        let issuer_state = random_string(ISSUER_STATE_LENGTH);

        let redirect_url = self
            .digid_client
            .authorization_request(
                self.client_id.clone(),
                self.callback_url.clone(),
                issuer_state.clone(),
                &upstream_pkce,
            )
            .await
            .map_err(Error::Digid)?;

        // Retain the wallet-side context so the callback can redirect the user-agent back to the
        // wallet and the `openid4vc` layer's /token handler can verify the wallet's PKCE challenge.
        let entry = StateBridgeEntry {
            context,
            upstream_code_verifier: upstream_pkce.into_code_verifier(),
        };
        self.state_bridge_store
            .store(issuer_state, entry)
            .await
            .map_err(Error::StateBridge)?;

        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }
}

/// `GET /digid/callback`: termination point for the upstream OIDC redirect. Exchanges the upstream
/// `code` for a BSN, looks up attributes in the BRP, builds the [`IssuableDocument`]s, and hands
/// them to the `openid4vc` layer's [`AuthorizingIssuer::complete_authorization`] which generates the
/// issuer-side authorization code and writes the `AuthCodeIssued` session. Errors surface to the
/// wallet as an OAuth error redirect, since the wallet's redirect_uri is known by then.
type DigidCallbackAuthorizingIssuer<K, L, S, N, PAS, B = HttpBrpClient, O = HttpDigidClient> =
    Arc<AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow<B, O>>>;

async fn digid_callback<K, L, S, N, PAS, B, O>(
    State(authorizing_issuer): State<DigidCallbackAuthorizingIssuer<K, L, S, N, PAS, B, O>>,
    Query(DigidCallbackQuery { code, state }): Query<DigidCallbackQuery>,
) -> Response
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
    B: BrpClient + Send + Sync + 'static,
    O: DigidClient + Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    let entry: StateBridgeEntry = match flow.state_bridge_store.consume(state.as_str()).await {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            warn!("digid callback: unknown or expired issuer_state");
            return (StatusCode::BAD_REQUEST, "unknown or expired state").into_response();
        }
        Err(error) => {
            warn!("digid callback: state bridge consume failed: {error}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "state bridge error").into_response();
        }
    };

    let StateBridgeEntry {
        context,
        upstream_code_verifier,
    } = entry;
    let WalletAuthorizationContext {
        redirect_uri,
        state,
        code_challenge,
    } = context;

    let result = complete_digid_callback(&authorizing_issuer, code_challenge, upstream_code_verifier, code)
        .await
        .inspect_err(|error| warn!("digid callback: completion failed: {error}"));

    let url = WalletRedirect::new(redirect_uri, state).into_redirect_url(result.as_ref(), "server_error");

    (StatusCode::FOUND, [(header::LOCATION, String::from(url))]).into_response()
}

/// Exchange the upstream `code` for the issuable documents, then hand them to the `openid4vc` layer's
/// [`AuthorizingIssuer::complete_authorization`], which generates the issuer-side authorization code,
/// writes the `AuthCodeIssued` session, and returns the code.
async fn complete_digid_callback<K, L, S, N, PAS, B, O>(
    authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow<B, O>>,
    code_challenge: String,
    upstream_code_verifier: String,
    digid_code: AuthorizationCode,
) -> Result<AuthorizationCode, Error>
where
    S: SessionStore<IssuanceData>,
    B: BrpClient,
    O: DigidClient,
{
    let issuable_documents = authorizing_issuer
        .flow()
        .fetch_issuable_documents(digid_code, upstream_code_verifier)
        .await?;

    authorizing_issuer
        .complete_authorization(issuable_documents, code_challenge)
        .await
        .map_err(Error::CompleteAuthorization)
}

/// Add the BRP-derived BSN's recovery code (an HMAC over the BSN) as an attribute
async fn insert_recovery_code(mut attributes: Attributes, secret_key: &SecretKeyVariant) -> Result<Attributes, Error> {
    let bsn = match attributes
        .get(&vec_nonempty![ClaimPath::SelectByKey(PID_BSN.to_string())])
        .map_err(Error::RetrievingBsn)?
        .ok_or(Error::NoBsnFound)?
    {
        AttributeValue::Text(str) => str,
        _ => return Err(Error::BsnUnexpectedType),
    };

    let recovery_code = AttributeValue::Text(hex::encode(
        secret_key.sign_hmac(bsn.as_bytes()).await.map_err(Error::Hmac)?,
    ));

    attributes
        .insert(
            &vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())],
            Attribute::Single(recovery_code),
        )
        .map_err(Error::InsertingRecoveryCode)?;

    Ok(attributes)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use indexmap::IndexMap;
    use issuer_common::state_bridge_store::IssuerStateBridgeStore;
    use openid4vc::authorization::VciAuthorizationRequest;
    use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
    use openid4vc::authorization_code_flow::AuthorizeOutcome;
    use openid4vc::authorization_code_flow::WalletAuthorizationContext;
    use openid4vc::authorizing_issuer::AuthorizingIssuer;
    use openid4vc::issuer::Grant;
    use openid4vc::issuer::IssuanceData;
    use openid4vc::issuer_identifier::IssuerIdentifier;
    use openid4vc::nonce::memory_store::MemoryNonceStore;
    use openid4vc::par::PAR_TTL;
    use openid4vc::pkce::PkcePair;
    use openid4vc::pkce::S256PkcePair;
    use openid4vc::server_state::MemorySessionStore;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionToken;
    use openid4vc::store::MemoryStore;
    use openid4vc::store::Store;
    use openid4vc::test::setup_mock_issuer;
    use openid4vc::token::AuthorizationCode;
    use p256::ecdsa::SigningKey;
    use ring::hmac;
    use ring::hmac::HMAC_SHA256;
    use server_utils::keys::SecretKeyVariant;
    use server_utils::settings::SecretKey;
    use server_utils::store::StoreConnection;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use utils::vec_nonempty;

    use super::DIGID_CALLBACK_PATH;
    use super::Error;
    use super::StateBridgeEntry;
    use super::UpstreamOidcAuthorizationCodeFlow;
    use super::complete_digid_callback;
    use super::insert_recovery_code;
    use crate::pid::constants::PID_ATTESTATION_TYPE;
    use crate::pid::mock::MockBrpClient;
    use crate::pid::mock::MockDigidClient;

    /// The in-memory [`AuthorizingIssuer`] flavour used by the callback tests: the mock inner issuer
    /// from `openid4vc::test` wrapped around a mock-backed flow.
    type TestAuthorizingIssuer = AuthorizingIssuer<
        SigningKey,
        MockStatusListService,
        MemorySessionStore<IssuanceData>,
        MemoryNonceStore,
        MemoryStore<String, VciAuthorizationRequest>,
        UpstreamOidcAuthorizationCodeFlow<MockBrpClient, MockDigidClient>,
    >;

    const CLIENT_ID: &str = "issuer-client-id";
    const CALLBACK_BASE_URL: &str = "https://issuer.example.com/";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_STATE: &str = "wallet-state";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";

    fn recovery_code_secret_key() -> SecretKeyVariant {
        SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: (0..32).collect::<Vec<_>>().try_into().unwrap(),
            },
            None,
        )
        .unwrap()
    }

    fn memory_bridge_store() -> Arc<IssuerStateBridgeStore<StateBridgeEntry>> {
        Arc::new(IssuerStateBridgeStore::new(StoreConnection::Memory))
    }

    fn flow_with_clients(
        brp_client: MockBrpClient,
        digid_client: MockDigidClient,
        state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
    ) -> UpstreamOidcAuthorizationCodeFlow<MockBrpClient, MockDigidClient> {
        UpstreamOidcAuthorizationCodeFlow::new_with_store(
            brp_client,
            digid_client,
            recovery_code_secret_key(),
            state_bridge_store,
            &CALLBACK_BASE_URL.parse().unwrap(),
            String::from(CLIENT_ID),
        )
    }

    /// Wrap a flow in an [`AuthorizingIssuer`] backed by an in-memory issuer + session store, so the
    /// callback path (which writes a session via `complete_authorization`) can be exercised. Returns
    /// the session store so tests can read the written session back.
    fn authorizing_issuer_with_flow(
        flow: UpstreamOidcAuthorizationCodeFlow<MockBrpClient, MockDigidClient>,
    ) -> (TestAuthorizingIssuer, Arc<MemorySessionStore<IssuanceData>>) {
        let issuer_identifier = IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap();
        let sessions = Arc::new(MemorySessionStore::default());
        let (issuer, _, _) = setup_mock_issuer(issuer_identifier, NonZeroUsize::MIN, Arc::clone(&sessions));
        let par_store = MemoryStore::new(PAR_TTL);
        let authorizing_issuer = AuthorizingIssuer::new(
            Arc::new(issuer),
            par_store,
            flow,
            vec_nonempty![WALLET_REDIRECT_URI.parse().unwrap()],
        );
        (authorizing_issuer, sessions)
    }

    fn wallet_request() -> VciAuthorizationRequest {
        VciAuthorizationRequest::for_auth_code(
            CLIENT_ID.to_string(),
            WALLET_REDIRECT_URI.parse().unwrap(),
            WALLET_STATE.to_string(),
            None,
            &S256PkcePair::generate(),
        )
    }

    fn state_bridge_entry() -> StateBridgeEntry {
        StateBridgeEntry {
            context: WalletAuthorizationContext {
                redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
                state: Some(WALLET_STATE.to_string()),
                code_challenge: WALLET_CODE_CHALLENGE.to_string(),
            },
            upstream_code_verifier: "upstream-verifier".to_string(),
        }
    }

    #[tokio::test]
    async fn test_recovery_code() {
        let bsn = "123";
        let key: Vec<_> = (0..32).collect();

        let attrs: Attributes = IndexMap::from_iter([(
            "bsn".to_string(),
            Attribute::Single(AttributeValue::Text(bsn.to_string())),
        )])
        .into();

        let secret_key = SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: key.clone().try_into().unwrap(),
            },
            None,
        )
        .unwrap();

        let attrs = insert_recovery_code(attrs, &secret_key).await.unwrap();

        let hmac_key = &hmac::Key::new(HMAC_SHA256, &key);
        let expected_hmac = hex::encode(hmac::sign(hmac_key, bsn.as_bytes()));

        let expected_attrs = Attributes::from(IndexMap::from_iter([
            (
                "bsn".to_string(),
                Attribute::Single(AttributeValue::Text(bsn.to_string())),
            ),
            (
                "recovery_code".to_string(),
                Attribute::Single(AttributeValue::Text(expected_hmac)),
            ),
        ]));

        assert_eq!(
            attrs, expected_attrs,
            "The result should be the attributes we started with, with a recovery_code attribute added to it."
        );
    }

    #[tokio::test]
    async fn authorize_builds_upstream_redirect_and_bridge_entry() {
        let bridge = memory_bridge_store();
        let flow = flow_with_clients(
            MockBrpClient::from_fixture("frouke"),
            MockDigidClient::default(),
            Arc::clone(&bridge),
        );

        let context = WalletAuthorizationContext::try_from_request(wallet_request()).unwrap();
        let wallet_code_challenge = context.code_challenge.clone();

        let outcome = flow.authorize(context).await.unwrap();
        let AuthorizeOutcome::RedirectTo(redirect_url) = outcome else {
            panic!("authorize should redirect the user-agent to the upstream provider");
        };

        // The flow asked the digid client for the upstream redirect, handing it the issuer's
        // callback URL and a generated `issuer_state` as the upstream `state`; the mock client
        // echoes both back in the redirect URL, and that same `issuer_state` keys the bridge entry.
        assert_eq!(redirect_url.path(), DIGID_CALLBACK_PATH);
        let query_params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        let issuer_state = query_params
            .get("state")
            .expect("upstream redirect should carry the issuer_state as state");

        let entry: StateBridgeEntry = bridge
            .consume(issuer_state.as_str())
            .await
            .unwrap()
            .expect("a bridge entry should be stored under the issuer_state");

        assert_eq!(entry.context.redirect_uri.as_str(), WALLET_REDIRECT_URI);
        assert_eq!(entry.context.state.as_deref(), Some(WALLET_STATE));
        assert_eq!(entry.context.code_challenge, wallet_code_challenge);
        assert!(!entry.upstream_code_verifier.is_empty());
    }

    #[tokio::test]
    async fn complete_callback_happy_path() {
        let flow = flow_with_clients(
            MockBrpClient::from_fixture("frouke"),
            MockDigidClient::default(),
            memory_bridge_store(),
        );
        let (authorizing_issuer, sessions) = authorizing_issuer_with_flow(flow);
        let StateBridgeEntry {
            context,
            upstream_code_verifier,
        } = state_bridge_entry();

        let code = complete_digid_callback(
            &authorizing_issuer,
            context.code_challenge,
            upstream_code_verifier,
            AuthorizationCode::from("upstream-code".to_string()),
        )
        .await
        .unwrap();

        // An AuthCodeIssued session was written, keyed by the returned code, carrying both PID
        // documents (SD-JWT + mdoc) and the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(code))
            .await
            .unwrap()
            .expect("a session should have been written under the returned code");
        let IssuanceData::AuthCodeIssued(auth_code_issued) = session.data else {
            panic!("expected an AuthCodeIssued session");
        };
        assert_eq!(
            auth_code_issued.grant,
            Grant::AuthorizationCode {
                wallet_code_challenge: WALLET_CODE_CHALLENGE.to_string()
            }
        );
        assert_eq!(auth_code_issued.issuable_documents.len().get(), 2);
        assert!(
            auth_code_issued
                .issuable_documents
                .as_ref()
                .iter()
                .all(|doc| doc.attestation_type == PID_ATTESTATION_TYPE)
        );
    }

    #[tokio::test]
    async fn complete_callback_rejects_when_no_attributes_found() {
        let flow = flow_with_clients(
            MockBrpClient::from_fixture("empty"),
            MockDigidClient::default(),
            memory_bridge_store(),
        );
        let (authorizing_issuer, _sessions) = authorizing_issuer_with_flow(flow);
        let StateBridgeEntry {
            context,
            upstream_code_verifier,
        } = state_bridge_entry();

        let error = complete_digid_callback(
            &authorizing_issuer,
            context.code_challenge,
            upstream_code_verifier,
            AuthorizationCode::from("upstream-code".to_string()),
        )
        .await
        .unwrap_err();

        assert_matches!(error, Error::NoAttributesFound);
    }
}
