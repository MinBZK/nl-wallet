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
use crypto::x509::CertificateError;
use hsm::service::HsmError;
use indexmap::IndexSet;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use jwk_simple::Key;
use jwt::nonce::Nonce;
use openid4vc::authorization::OidcAuthorizationRequest;
use openid4vc::authorization::VciAuthorizationRequest;
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

/// Errors raised by [`UpstreamOidcAuthorizationCodeFlow`] on either half of the flow.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[source] digid::Error),

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

    #[error("encoding upstream authorization request as query string failed: {0}")]
    Encode(#[source] serde_urlencoded::ser::Error),

    #[error("could not find attributes for BSN")]
    NoAttributesFound,

    #[error("error retrieving from BRP: {0}")]
    Brp(#[source] BrpError),

    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,

    #[error("certificate error: {0}")]
    Certificate(#[source] CertificateError),

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
    state_bridge_store: Arc<IssuerStateBridgeStore>,
    callback_uri: Url,
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
        state_bridge_store: Arc<IssuerStateBridgeStore>,
        callback_uri: Url,
    ) -> Result<Self, Error> {
        let client_id: String = client_id.into();
        let digid_client =
            HttpDigidClient::try_new(bsn_privkey, client_id.clone(), digid_metadata_cache).map_err(Error::Digid)?;

        Ok(Self::new_with_clients(
            brp_client,
            digid_client,
            client_id,
            recovery_code_secret_key,
            state_bridge_store,
            callback_uri,
        ))
    }
}

impl<B, O> UpstreamOidcAuthorizationCodeFlow<B, O> {
    fn new_with_clients(
        brp_client: B,
        digid_client: O,
        client_id: impl Into<String>,
        recovery_code_secret_key: SecretKeyVariant,
        state_bridge_store: Arc<IssuerStateBridgeStore>,
        callback_uri: Url,
    ) -> Self {
        Self {
            brp_client,
            digid_client,
            recovery_code_secret_key,
            state_bridge_store,
            callback_uri,
            client_id: client_id.into(),
        }
    }

    /// Mount the `/digid/callback` route owned by this flow on a fresh [`Router`]. The
    /// pid_issuer binary merges this with the `openid4vc` layer's authorization and issuance routers.
    /// The handler reads its flow state via [`AuthorizingIssuer::flow`].
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
            .route("/digid/callback", get(digid_callback))
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
            .bsn(upstream_code, upstream_code_verifier, Some(self.callback_uri.clone()))
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

        // Create a new upstream authorization request
        let mut upstream_request = VciAuthorizationRequest::for_auth_code(
            self.client_id.clone(),
            self.callback_uri.clone(),
            issuer_state.clone(),
            None,
            &upstream_pkce,
        );
        upstream_request.scope = Some(IndexSet::from_iter([String::from("openid")]));

        let oidc_request = OidcAuthorizationRequest {
            vci_request: upstream_request,
            nonce: Some(Nonce::new_random()),
        };

        let query_string = serde_urlencoded::to_string(&oidc_request).map_err(Error::Encode)?;
        let mut redirect_url = self.digid_client.authorization_endpoint().await.map_err(Error::Digid)?;
        redirect_url.set_query(Some(&query_string));

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
/// issuer-side authorization code, writes the `AuthCodeIssued` session, and produces the wallet-facing
/// redirect URL. Errors during the BSN / BRP / issuable-build steps surface to the wallet as an
/// OAuth error redirect, since the wallet's redirect_uri is known by then.
type DigidCallbackAuthorizingIssuer<K, L, S, N, PAS, B, O> =
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

    // The wallet's redirect_uri is known from here on, so for any failure below, the oauth error redirect uri is sent
    // back to the wallet.
    let wallet_redirect = WalletRedirect::new(entry.context.redirect_uri.clone(), entry.context.state.clone());

    let result = complete_digid_callback(&authorizing_issuer, entry, code).await;
    if let Err(error) = &result {
        warn!("digid callback: completion failed: {error}");
    }

    let url = wallet_redirect.into_redirect_url(result, "server_error");
    (StatusCode::FOUND, [(header::LOCATION, String::from(url))]).into_response()
}

/// Exchange the upstream `code` for the issuable documents, then hand them to the `openid4vc` layer's
/// [`AuthorizingIssuer::complete_authorization`], which generates the issuer-side authorization code,
/// writes the `AuthCodeIssued` session, and returns the wallet-facing redirect URL.
async fn complete_digid_callback<K, L, S, N, PAS, B, O>(
    authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow<B, O>>,
    entry: StateBridgeEntry,
    code: AuthorizationCode,
) -> Result<Url, Error>
where
    S: SessionStore<IssuanceData>,
    B: BrpClient,
    O: DigidClient,
{
    let issuable_documents = authorizing_issuer
        .flow()
        .fetch_issuable_documents(code, entry.upstream_code_verifier)
        .await?;

    authorizing_issuer
        .complete_authorization(issuable_documents, entry.context)
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
    use std::fs;
    use std::num::NonZeroUsize;
    use std::path::PathBuf;
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
    use openid4vc::authorizing_issuer::WalletRedirect;
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
    use url::Url;
    use utils::path::prefix_local_path;
    use utils::vec_nonempty;

    use super::Error;
    use super::StateBridgeEntry;
    use super::UpstreamOidcAuthorizationCodeFlow;
    use super::complete_digid_callback;
    use super::insert_recovery_code;
    use crate::pid::brp::client::BrpClient;
    use crate::pid::brp::client::BrpError;
    use crate::pid::brp::data::BrpPersons;
    use crate::pid::constants::PID_ATTESTATION_TYPE;
    use crate::pid::digid::DigidClient;
    use crate::pid::digid::Error as DigidError;

    /// The in-memory [`AuthorizingIssuer`] flavour used by the callback tests: the mock inner issuer
    /// from `openid4vc::test` wrapped around a fake-backed flow.
    type TestAuthorizingIssuer = AuthorizingIssuer<
        SigningKey,
        MockStatusListService,
        MemorySessionStore<IssuanceData>,
        MemoryNonceStore,
        MemoryStore<String, VciAuthorizationRequest>,
        UpstreamOidcAuthorizationCodeFlow<FakeBrpClient, FakeDigidClient>,
    >;

    const CLIENT_ID: &str = "issuer-client-id";
    const CALLBACK_URI: &str = "https://issuer.example.com/digid/callback";
    const UPSTREAM_AUTHORIZATION_ENDPOINT: &str = "https://digid.example.com/oauth2/authorize";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_STATE: &str = "wallet-state";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";

    /// [`BrpClient`] returning a `BrpPersons` deserialized from a haal-centraal test
    /// fixture, so the callback path can be exercised without a live BRP proxy.
    struct FakeBrpClient {
        persons_json: String,
    }

    impl FakeBrpClient {
        fn from_fixture(name: &str) -> Self {
            let persons_json = fs::read_to_string(prefix_local_path(PathBuf::from(format!(
                "resources/test/haal-centraal-examples/{name}.json"
            ))))
            .unwrap();
            Self { persons_json }
        }
    }

    impl BrpClient for FakeBrpClient {
        async fn get_person_by_bsn(&self, _bsn: &str) -> Result<BrpPersons, BrpError> {
            Ok(serde_json::from_str(&self.persons_json)?)
        }
    }

    /// [`DigidClient`] returning fixed values, so the flow can be tested without a live
    /// upstream provider.
    struct FakeDigidClient {
        authorization_endpoint: Url,
        bsn: String,
    }

    impl Default for FakeDigidClient {
        fn default() -> Self {
            Self {
                authorization_endpoint: UPSTREAM_AUTHORIZATION_ENDPOINT.parse().unwrap(),
                bsn: "999991772".to_string(),
            }
        }
    }

    impl DigidClient for FakeDigidClient {
        async fn authorization_endpoint(&self) -> Result<Url, DigidError> {
            Ok(self.authorization_endpoint.clone())
        }

        async fn bsn(
            &self,
            _code: AuthorizationCode,
            _code_verifier: String,
            _redirect_uri: Option<Url>,
        ) -> Result<String, DigidError> {
            Ok(self.bsn.clone())
        }
    }

    fn recovery_code_secret_key() -> SecretKeyVariant {
        SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: (0..32).collect::<Vec<_>>().try_into().unwrap(),
            },
            None,
        )
        .unwrap()
    }

    fn memory_bridge_store() -> Arc<IssuerStateBridgeStore> {
        Arc::new(IssuerStateBridgeStore::new(StoreConnection::Memory))
    }

    fn flow_with_clients(
        brp_client: FakeBrpClient,
        digid_client: FakeDigidClient,
        state_bridge_store: Arc<IssuerStateBridgeStore>,
    ) -> UpstreamOidcAuthorizationCodeFlow<FakeBrpClient, FakeDigidClient> {
        UpstreamOidcAuthorizationCodeFlow::new_with_clients(
            brp_client,
            digid_client,
            CLIENT_ID,
            recovery_code_secret_key(),
            state_bridge_store,
            CALLBACK_URI.parse().unwrap(),
        )
    }

    /// Wrap a flow in an [`AuthorizingIssuer`] backed by an in-memory issuer + session store, so the
    /// callback path (which writes a session via `complete_authorization`) can be exercised. Returns
    /// the session store so tests can read the written session back.
    fn authorizing_issuer_with_flow(
        flow: UpstreamOidcAuthorizationCodeFlow<FakeBrpClient, FakeDigidClient>,
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
            FakeBrpClient::from_fixture("frouke"),
            FakeDigidClient::default(),
            Arc::clone(&bridge),
        );

        let context = WalletAuthorizationContext::try_from_request(wallet_request()).unwrap();
        let wallet_code_challenge = context.code_challenge.clone();

        let outcome = flow.authorize(context).await.unwrap();
        let AuthorizeOutcome::RedirectTo(redirect_url) = outcome else {
            panic!("expected a RedirectTo outcome");
        };

        // The redirect targets the upstream authorization endpoint.
        assert!(redirect_url.as_str().starts_with(UPSTREAM_AUTHORIZATION_ENDPOINT));

        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        // The upstream request carries the issuer's own client_id and callback as redirect_uri.
        assert_eq!(params.get("client_id").map(String::as_str), Some(CLIENT_ID));
        assert_eq!(params.get("redirect_uri").map(String::as_str), Some(CALLBACK_URI));
        assert_eq!(params.get("scope").map(String::as_str), Some("openid"));
        assert!(params.contains_key("nonce"));
        // The upstream PKCE challenge is freshly generated, not the wallet's.
        assert_eq!(params.get("code_challenge_method").map(String::as_str), Some("S256"));
        assert_ne!(
            params.get("code_challenge").map(String::as_str),
            Some(wallet_code_challenge.as_str())
        );

        // The `state` is the issuer_state, which keys the bridge entry.
        let issuer_state = params
            .get("state")
            .expect("redirect should carry a state param")
            .clone();
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
            FakeBrpClient::from_fixture("frouke"),
            FakeDigidClient::default(),
            memory_bridge_store(),
        );
        let (authorizing_issuer, sessions) = authorizing_issuer_with_flow(flow);
        let entry = state_bridge_entry();

        let redirect_url = complete_digid_callback(
            &authorizing_issuer,
            entry,
            AuthorizationCode::from("upstream-code".to_string()),
        )
        .await
        .unwrap();

        // The wallet is redirected back to its redirect_uri with a code and the echoed state.
        assert!(redirect_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = redirect_url.query_pairs().into_owned().collect();
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
        let code = params.get("code").expect("redirect should carry a code").clone();

        // An AuthCodeIssued session was written, keyed by the generated code, carrying both PID
        // documents (SD-JWT + mdoc) and the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(AuthorizationCode::from(code)))
            .await
            .unwrap()
            .expect("a session should have been written under the migeneratednted code");
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
            FakeBrpClient::from_fixture("empty"),
            FakeDigidClient::default(),
            memory_bridge_store(),
        );
        let (authorizing_issuer, _sessions) = authorizing_issuer_with_flow(flow);
        let entry = state_bridge_entry();

        let error = complete_digid_callback(
            &authorizing_issuer,
            entry,
            AuthorizationCode::from("upstream-code".to_string()),
        )
        .await
        .unwrap_err();

        assert_matches!(error, Error::NoAttributesFound);
    }

    #[test]
    fn error_redirect_carries_oauth_error() {
        let entry = state_bridge_entry();
        let wallet_redirect = WalletRedirect::new(entry.context.redirect_uri.clone(), entry.context.state.clone());

        // A completion failure resolves to a wallet error redirect carrying the `server_error` OAuth
        // error code, the error's description, and the wallet's original state.
        let url = wallet_redirect.into_redirect_url(Err(Error::NoAttributesFound), "server_error");

        assert!(url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("error").map(String::as_str), Some("server_error"));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
        assert!(params.contains_key("error_description"));
    }
}
