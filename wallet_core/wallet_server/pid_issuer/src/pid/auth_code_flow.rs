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
use openid4vc::authorization::PkceCodeChallenge;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
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

    #[error("only S256 code_challenge_method is supported")]
    UnsupportedCodeChallenge,

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

    #[error("wallet did not supply a redirect_uri at /authorize, cannot finish the callback")]
    MissingWalletRedirectUri,

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
    wallet_redirect_uri: Url,
    wallet_state: Option<String>,
    wallet_code_challenge: String,
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
    /// pid_issuer binary merges this with the framework's authorization and issuance routers.
    /// The handler reads its flow state via [`AuthorizingIssuer::flow`].
    pub fn callback_router<K, L, S, N, PAS>(authorizing_issuer: Arc<AuthorizingIssuer<K, L, S, N, PAS, Self>>) -> Router
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
            .route("/digid/callback", get(digid_callback::<K, L, S, N, PAS, B, O>))
            .with_state(authorizing_issuer)
    }
}

impl<B, O> AuthorizationCodeFlow for UpstreamOidcAuthorizationCodeFlow<B, O>
where
    B: BrpClient + Send + Sync,
    O: DigidClient + Send + Sync,
{
    type Error = Error;

    async fn authorize(&self, request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
        // Capture the wallet-side parameters we'll need at callback time to redirect the
        // user-agent back to the wallet, and the wallet's PKCE challenge that the framework's
        // /token handler will verify against.
        let wallet_code_challenge = match request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge,
            PkceCodeChallenge::Plain { .. } => return Err(Error::UnsupportedCodeChallenge),
        };

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

        let entry = StateBridgeEntry {
            wallet_redirect_uri: request.redirect_uri.into_inner(),
            wallet_state: request.oauth_request.state.clone(),
            wallet_code_challenge,
            upstream_code_verifier: upstream_pkce.into_code_verifier(),
        };
        self.state_bridge_store
            .store(issuer_state, entry)
            .await
            .map_err(Error::StateBridge)?;

        let oidc_request = OidcAuthorizationRequest {
            vci_request: upstream_request,
            nonce: Some(Nonce::new_random()),
        };

        let query_string = serde_urlencoded::to_string(&oidc_request).map_err(Error::Encode)?;
        let mut redirect_url = self.digid_client.authorization_endpoint().await.map_err(Error::Digid)?;
        redirect_url.set_query(Some(&query_string));

        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }
}

/// `GET /digid/callback`: termination point for the upstream OIDC redirect. Exchanges the upstream
/// `code` for a BSN, looks up attributes in the BRP, builds the [`IssuableDocument`]s, and hands
/// them to the framework's [`AuthorizingIssuer::complete_authorization`] which mints the
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

    match complete_digid_callback(flow, authorizing_issuer.as_ref(), &entry, code).await {
        Ok(wallet_redirect_url) => {
            (StatusCode::FOUND, [(header::LOCATION, wallet_redirect_url.to_string())]).into_response()
        }
        Err(error) => {
            warn!("digid callback: completion failed: {error}");
            redirect_to_wallet_error(&entry, &error)
        }
    }
}

/// Exchange the upstream `code` for the user's BSN, look up attributes, build the issuable documents, and hand them to
/// the framework's [`AuthorizingIssuer::complete_authorization`], which mints the issuer-side authorization code,
/// writes the `AuthCodeIssued` session, and returns the wallet-facing redirect URL.
async fn complete_digid_callback<K, L, S, N, PAS, B, O>(
    flow: &UpstreamOidcAuthorizationCodeFlow<B, O>,
    authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow<B, O>>,
    entry: &StateBridgeEntry,
    upstream_code: AuthorizationCode,
) -> Result<Url, Error>
where
    S: SessionStore<IssuanceData>,
    B: BrpClient,
    O: DigidClient,
{
    let bsn = flow
        .digid_client
        .bsn(
            upstream_code,
            entry.upstream_code_verifier.clone(),
            Some(flow.callback_uri.clone()),
        )
        .await
        .map_err(Error::Digid)?;

    let mut persons = flow.brp_client.get_person_by_bsn(&bsn).await.map_err(Error::Brp)?;
    if persons.persons.len() != 1 {
        return Err(Error::NoAttributesFound);
    }
    let person = persons.persons.remove(0);
    let attributes = insert_recovery_code(person.into_attributes(), &flow.recovery_code_secret_key).await?;

    let issuable_documents = vec_nonempty![
        IssuableDocument::try_new_with_random_id(Format::SdJwt, PID_ATTESTATION_TYPE.to_string(), attributes.clone(),)
            .map_err(|_| Error::InvalidIssuableDocuments)?,
        IssuableDocument::try_new_with_random_id(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string(), attributes)
            .map_err(|_| Error::InvalidIssuableDocuments)?,
    ];

    let (_code, wallet_redirect_url) = authorizing_issuer
        .complete_authorization(
            issuable_documents,
            entry.wallet_code_challenge.clone(),
            entry.wallet_redirect_uri.clone(),
            entry.wallet_state.clone(),
        )
        .await
        .map_err(Error::CompleteAuthorization)?;

    Ok(wallet_redirect_url)
}

fn redirect_to_wallet_error(entry: &StateBridgeEntry, error: &Error) -> Response {
    #[derive(Serialize)]
    struct RedirectErrorQuery<'a> {
        error: &'a str,
        error_description: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<&'a str>,
    }

    let mut url = entry.wallet_redirect_uri.clone();

    let query = serde_urlencoded::to_string(RedirectErrorQuery {
        error: "server_error",
        error_description: &error.to_string(),
        state: entry.wallet_state.as_deref(),
    })
    .expect("encoding error query string should never fail");

    url.set_query(Some(&query));
    (StatusCode::FOUND, [(header::LOCATION, url.to_string())]).into_response()
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
    use axum::http::StatusCode;
    use axum::http::header;
    use indexmap::IndexMap;
    use issuer_common::state_bridge_store::IssuerStateBridgeStore;
    use openid4vc::authorization::PkceCodeChallenge;
    use openid4vc::authorization::VciAuthorizationRequest;
    use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
    use openid4vc::authorization_code_flow::AuthorizeOutcome;
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
    use url::Url;
    use utils::path::prefix_local_path;
    use utils::vec_nonempty;

    use super::Error;
    use super::StateBridgeEntry;
    use super::UpstreamOidcAuthorizationCodeFlow;
    use super::complete_digid_callback;
    use super::insert_recovery_code;
    use super::redirect_to_wallet_error;
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
            wallet_redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
            wallet_state: Some(WALLET_STATE.to_string()),
            wallet_code_challenge: WALLET_CODE_CHALLENGE.to_string(),
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
    async fn authorize_rejects_plain_code_challenge() {
        let flow = flow_with_clients(
            FakeBrpClient::from_fixture("frouke"),
            FakeDigidClient::default(),
            memory_bridge_store(),
        );

        let mut request = wallet_request();
        request.code_challenge = PkceCodeChallenge::Plain {
            code_challenge: "plain-challenge".to_string(),
        };

        let error = flow.authorize(request).await.unwrap_err();
        assert_matches!(error, Error::UnsupportedCodeChallenge);
    }

    #[tokio::test]
    async fn authorize_builds_upstream_redirect_and_bridge_entry() {
        let bridge = memory_bridge_store();
        let flow = flow_with_clients(
            FakeBrpClient::from_fixture("frouke"),
            FakeDigidClient::default(),
            Arc::clone(&bridge),
        );

        let request = wallet_request();
        let wallet_code_challenge = match &request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge.clone(),
            PkceCodeChallenge::Plain { .. } => unreachable!(),
        };

        let outcome = flow.authorize(request).await.unwrap();
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
        assert_eq!(entry.wallet_redirect_uri.as_str(), WALLET_REDIRECT_URI);
        assert_eq!(entry.wallet_state.as_deref(), Some(WALLET_STATE));
        assert_eq!(entry.wallet_code_challenge, wallet_code_challenge);
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
            authorizing_issuer.flow(),
            &authorizing_issuer,
            &entry,
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
            authorizing_issuer.flow(),
            &authorizing_issuer,
            &entry,
            AuthorizationCode::from("upstream-code".to_string()),
        )
        .await
        .unwrap_err();

        assert_matches!(error, Error::NoAttributesFound);
    }

    #[test]
    fn error_redirect_carries_oauth_error() {
        let entry = state_bridge_entry();
        let response = redirect_to_wallet_error(&entry, &Error::NoAttributesFound);

        assert_eq!(response.status(), StatusCode::FOUND);
        let location = response.headers().get(header::LOCATION).unwrap().to_str().unwrap();
        let location_url: Url = location.parse().unwrap();
        assert!(location_url.as_str().starts_with(WALLET_REDIRECT_URI));
        let params: HashMap<_, _> = location_url.query_pairs().into_owned().collect();
        assert_eq!(params.get("error").map(String::as_str), Some("server_error"));
        assert_eq!(params.get("state").map(String::as_str), Some(WALLET_STATE));
        assert!(params.contains_key("error_description"));
    }
}
