//! The authorization-code-flow implementation for a demo issuer.
//!
//! Unlike the `pid_issuer`, which bounces the user to an upstream OIDC provider (DigiD), the demo
//! issuer authenticates the holder with a local consent page. The [`AuthorizationCodeFlow::authorize`]
//! implementation redirects the user-agent to that consent page (carrying a random flow-state token),
//! and the `POST /consent` callback loads the stashed wallet request, builds the configured
//! [`IssuableDocument`]s and hands them to [`AuthorizingIssuer::complete_authorization`].
//!
//! The requested usecase is identified by the OpenID4VCI `issuer_state` value, which the credential
//! offer carries and the wallet echoes back in its authorization request.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use crypto::utils::random_string;
use http_utils::urls::BaseUrl;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
use openid4vc::authorizing_issuer::WalletRedirect;
use openid4vc::issuable_document::CredentialKind;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::IssuanceData;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Store;
use openid4vc::token::AuthorizationCode;
use serde::Deserialize;
use server_utils::store::StoreConnection;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::warn;
use url::Url;
use utils::path::prefix_local_path;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use web_utils::headers::set_static_cache_control;
use web_utils::language::Language;

use crate::settings::IssuableDocumentTemplate;
use crate::settings::Usecase;
use crate::settings::UsecaseKind;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;

/// Length of the random flow-state token used as key in the state-bridge store.
const FLOW_STATE_LENGTH: usize = 32;

/// Path (relative to the issuer's public URL) of the consent page served by this flow.
const CONSENT_PATH: &str = "consent";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("authorization request did not carry an issuer_state, cannot determine the usecase")]
    MissingIssuerState,

    #[error("unknown usecase: {0}")]
    UnknownUsecase(String),

    #[error("usecase {0} is not configured for the consent flow")]
    NotConsentUsecase(String),

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

    #[error("error completing authorization: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

/// The [`AuthorizingIssuer`] flavour parameterised over this flow, kept as an alias to keep handler
/// and router signatures readable.
type DemoAuthorizingIssuer<K, L, S, N, PAS> = AuthorizingIssuer<K, L, S, N, PAS, DemoAuthorizationCodeFlow>;

/// The shared [`axum`] state of the consent routes.
type ConsentCallbackIssuer<K, L, S, N, PAS> = Arc<DemoAuthorizingIssuer<K, L, S, N, PAS>>;

/// Concrete [`AuthorizationCodeFlow`] for the demo issuer's local-consent flow.
///
/// Owns:
/// - the state-bridge store linking the flow-state token to the wallet's original `redirect_uri`, `state` and PKCE
///   challenge plus the requested usecase;
/// - the URL of the consent page the user-agent is redirected to;
/// - the configured usecases, keyed by `issuer_state`.
pub struct DemoAuthorizationCodeFlow {
    state_bridge_store: Arc<IssuerStateBridgeStore<WalletAuthorizationContext>>,
    consent_uri: Url,
    usecases: HashMap<String, Usecase>,
}

impl DemoAuthorizationCodeFlow {
    pub fn new(
        store_connection: StoreConnection,
        consent_base_url: &BaseUrl,
        usecases: HashMap<String, Usecase>,
    ) -> Self {
        Self {
            state_bridge_store: Arc::new(IssuerStateBridgeStore::new(store_connection)),
            consent_uri: consent_base_url.join(CONSENT_PATH),
            usecases,
        }
    }

    /// Mount the `/consent` routes owned by this flow on a fresh [`Router`]. The demo issuer binary
    /// merges this with the framework's authorization and issuance routers. The handlers read their
    /// flow state via [`AuthorizingIssuer::flow`].
    pub fn callback_router<K, L, S, N, PAS>(authorizing_issuer: ConsentCallbackIssuer<K, L, S, N, PAS>) -> Router
    where
        K: Send + Sync + 'static,
        L: Send + Sync + 'static,
        S: SessionStore<IssuanceData> + Send + Sync + 'static,
        N: Send + Sync + 'static,
        PAS: Send + Sync + 'static,
    {
        Router::new()
            .route(&format!("/{CONSENT_PATH}"), get(consent_page).post(consent_submit))
            .with_state(authorizing_issuer)
    }
}

/// Build the [`IssuableDocument`]s configured for a usecase.
fn issuable_documents(usecase: &Usecase) -> VecNonEmpty<IssuableDocument> {
    usecase
        .documents
        .nonempty_iter()
        .map(|template| {
            let IssuableDocumentTemplate {
                credential_kind,
                attributes,
            } = template.clone();
            IssuableDocument::try_new_with_random_id(credential_kind, attributes).expect("attributes cannot be empty")
        })
        .collect()
}

impl AuthorizationCodeFlow for DemoAuthorizationCodeFlow {
    type Error = Error;

    async fn authorize(
        &self,
        context: WalletAuthorizationContext,
        _credential_kinds: VecNonEmpty<CredentialKind>,
    ) -> Result<AuthorizeOutcome, Self::Error> {
        // The usecase is identified by the `issuer_state` the wallet echoes back from the offer.
        let usecase_id = context.issuer_state.as_deref().ok_or(Error::MissingIssuerState)?;
        let usecase = self
            .usecases
            .get(usecase_id)
            .ok_or_else(|| Error::UnknownUsecase(usecase_id.to_string()))?;
        if !matches!(usecase.kind, UsecaseKind::Consent) {
            return Err(Error::NotConsentUsecase(usecase_id.to_string()));
        }

        // Stash the wallet request under a random flow-state token, then redirect to the consent page.
        let flow_state = random_string(FLOW_STATE_LENGTH);

        let mut consent_url = self.consent_uri.clone();
        consent_url
            .query_pairs_mut()
            .append_pair("state", &flow_state)
            .append_pair("usecase", usecase_id);

        self.state_bridge_store
            .store(flow_state, context)
            .await
            .map_err(Error::StateBridge)?;

        Ok(AuthorizeOutcome::RedirectTo(consent_url))
    }
}

// Bundled CSS — placeholder in dev, full bundle in release. Dev builds are served via ServeDir.
#[cfg(not(debug_assertions))]
const CONSENT_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/consent.css"));

struct DocumentPreview {
    attestation_type: String,
    attributes: Vec<(String, String)>,
}

fn collect_attributes(prefix: &str, attr: &Attribute, out: &mut Vec<(String, String)>) {
    match attr {
        Attribute::Single(value) => out.push((prefix.to_string(), value.to_string())),
        Attribute::Nested(map) => {
            for (key, child) in map {
                collect_attributes(&format!("{prefix}.{key}"), child, out);
            }
        }
    }
}

fn flatten_attributes(attrs: &Attributes) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (key, attr) in attrs.as_ref() {
        collect_attributes(key, attr, &mut out);
    }
    out
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
}

#[derive(Template, WebTemplate)]
#[template(path = "consent.askama", escape = "html", ext = "html")]
struct ConsentTemplate<'a> {
    state: String,
    doc_previews: Vec<DocumentPreview>,
    base: BaseTemplate<'a>,
}

/// Query parameters of the consent page, set by [`DemoAuthorizationCodeFlow::authorize`].
#[derive(Deserialize)]
struct ConsentQuery {
    state: String,
    usecase: String,
}

/// `GET /consent`: render the consent page. The `state` token is round-tripped to `POST /consent`.
async fn consent_page<K, L, S, N, PAS>(
    State(authorizing_issuer): State<ConsentCallbackIssuer<K, L, S, N, PAS>>,
    Query(ConsentQuery { state, usecase }): Query<ConsentQuery>,
    language: Language,
) -> Response
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let doc_previews = authorizing_issuer
        .flow()
        .usecases
        .get(&usecase)
        .into_iter()
        .flat_map(|uc| uc.documents.as_ref())
        .map(|doc| {
            let IssuableDocumentTemplate {
                credential_kind,
                attributes,
            } = doc.clone();
            DocumentPreview {
                attestation_type: credential_kind.attestation_type,
                attributes: flatten_attributes(&attributes),
            }
        })
        .collect();

    ConsentTemplate {
        state,
        doc_previews,
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
        },
    }
    .into_response()
}

/// Creates a router that serves the static assets for the consent page (CSS, fonts, images).
///
/// This must be merged into the top-level router **outside** of `add_cache_control_no_store_layer`
/// so that fonts and images can be cached by the browser.
pub fn create_static_router() -> Router {
    let app = Router::new();

    // In release, serve the bundled CSS from a route handler (no disk access needed).
    // In debug, the ServeDir fallback below serves it directly from the symlinked source tree.
    #[cfg(not(debug_assertions))]
    let app = {
        use axum::http::HeaderMap;
        use web_utils::css::serve_bundled_css;
        app.route(
            "/static/css/consent.css",
            get(|h: HeaderMap| async move { serve_bundled_css(&h, CONSENT_CSS) }),
        )
    };

    app.fallback_service(
        ServiceBuilder::new()
            .layer(middleware::from_fn(set_static_cache_control))
            .service(
                ServeDir::new(prefix_local_path(Path::new("assets")))
                    .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
            ),
    )
}

/// Query parameters of the consent submission.
#[derive(Deserialize)]
struct ConsentSubmitQuery {
    state: String,
}

/// `POST /consent`: the holder consented. Load the stashed wallet request, build the configured
/// documents and hand them to [`AuthorizingIssuer::complete_authorization`], which mints the
/// issuer-side authorization code, writes the `AuthCodeIssued` session and produces the wallet-facing
/// redirect URL. Errors after the wallet's redirect_uri is known surface as an OAuth error redirect.
async fn consent_submit<K, L, S, N, PAS>(
    State(authorizing_issuer): State<ConsentCallbackIssuer<K, L, S, N, PAS>>,
    Query(ConsentSubmitQuery { state }): Query<ConsentSubmitQuery>,
) -> Response
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    let context: WalletAuthorizationContext = match flow.state_bridge_store.consume(state.as_str()).await {
        Ok(Some(context)) => context,
        Ok(None) => {
            warn!("consent submit: unknown or expired flow state");
            return (StatusCode::BAD_REQUEST, "unknown or expired state").into_response();
        }
        Err(error) => {
            warn!("consent submit: state bridge consume failed: {error}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "state bridge error").into_response();
        }
    };

    let result = complete_consent(flow, authorizing_issuer.as_ref(), &context)
        .await
        .inspect_err(|error| warn!("consent callback: completion failed: {error}"));

    let url =
        WalletRedirect::new(context.redirect_uri, context.state).into_redirect_url(result.as_ref(), "server_error");

    (StatusCode::FOUND, [(header::LOCATION, String::from(url))]).into_response()
}

/// Build the configured documents for the entry's usecase and hand them to the `openid4vc` layer's
/// [`AuthorizingIssuer::complete_authorization`], ensuring a session is created keyed by a new authorization_code
/// containing the documents.
async fn complete_consent<K, L, S, N, PAS>(
    flow: &DemoAuthorizationCodeFlow,
    authorizing_issuer: &DemoAuthorizingIssuer<K, L, S, N, PAS>,
    context: &WalletAuthorizationContext,
) -> Result<AuthorizationCode, Error>
where
    S: SessionStore<IssuanceData>,
{
    // The usecase is identified by the `issuer_state` the wallet echoed back from the offer; `authorize`
    // already validated it is present, but re-resolve it here as the single source of truth for the session.
    let usecase_id = context.issuer_state.as_deref().ok_or(Error::MissingIssuerState)?;
    let usecase = flow
        .usecases
        .get(usecase_id)
        .ok_or_else(|| Error::UnknownUsecase(usecase_id.to_string()))?;

    let code = authorizing_issuer
        .complete_authorization(
            issuable_documents(usecase),
            context.scope.clone(),
            context.code_challenge.clone(),
        )
        .await
        .map_err(Error::CompleteAuthorization)?;

    Ok(code)
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_types::credential_format::Format;
    use indexmap::IndexMap;
    use openid4vc::authorization::VciAuthorizationRequest;
    use openid4vc::authorization_code_flow::WalletAuthorizationContext;
    use openid4vc::authorizing_issuer::AuthorizingIssuer;
    use openid4vc::issuable_document::CredentialKind;
    use openid4vc::issuer::Grant;
    use openid4vc::issuer::IssuanceData;
    use openid4vc::issuer_identifier::IssuerIdentifier;
    use openid4vc::nonce::memory_store::MemoryNonceStore;
    use openid4vc::par::PAR_TTL;
    use openid4vc::scope::Scope;
    use openid4vc::server_state::MemorySessionStore;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionToken;
    use openid4vc::store::MemoryStore;
    use openid4vc::test::MOCK_ATTRS;
    use openid4vc::test::mock_type_metadata;
    use openid4vc::test::setup_mock_issuer_from_sd_jwt_metadata;
    use p256::ecdsa::SigningKey;
    use server_utils::store::StoreConnection;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use utils::vec_nonempty;

    use super::DemoAuthorizationCodeFlow;
    use super::complete_consent;
    use crate::settings::IssuableDocumentTemplate;
    use crate::settings::Usecase;
    use crate::settings::UsecaseKind;

    type TestAuthorizingIssuer = AuthorizingIssuer<
        SigningKey,
        MockStatusListService,
        MemorySessionStore<IssuanceData>,
        MemoryNonceStore,
        MemoryStore<String, VciAuthorizationRequest>,
        DemoAuthorizationCodeFlow,
    >;

    const CONSENT_BASE_URL: &str = "https://issuer.example.com/";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";
    const WALLET_SCOPE: &str = "wallet-scope";
    const USECASE_ID: &str = "diploma";
    const ATTESTATION_TYPE: &str = "com.example.diploma";

    fn test_usecases() -> HashMap<String, Usecase> {
        let attributes: Attributes = IndexMap::from_iter(MOCK_ATTRS.map(|(key, val)| {
            (
                key.to_string(),
                Attribute::Single(AttributeValue::Text(val.to_string())),
            )
        }))
        .into();

        let template = IssuableDocumentTemplate::new(
            CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string()),
            attributes,
        );

        HashMap::from([(
            USECASE_ID.to_string(),
            Usecase {
                kind: UsecaseKind::Consent,
                documents: vec_nonempty![template],
            },
        )])
    }

    fn flow() -> DemoAuthorizationCodeFlow {
        DemoAuthorizationCodeFlow::new(
            StoreConnection::Memory,
            &CONSENT_BASE_URL.parse().unwrap(),
            test_usecases(),
        )
    }

    fn authorizing_issuer_with_flow(
        flow: DemoAuthorizationCodeFlow,
    ) -> (TestAuthorizingIssuer, Arc<MemorySessionStore<IssuanceData>>) {
        let issuer_identifier = IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap();
        let sessions = Arc::new(MemorySessionStore::default());
        let (issuer, _, _) = setup_mock_issuer_from_sd_jwt_metadata(
            issuer_identifier,
            vec![mock_type_metadata(ATTESTATION_TYPE)],
            Arc::clone(&sessions),
        );
        let par_store = MemoryStore::new(PAR_TTL);
        let authorizing_issuer = AuthorizingIssuer::new(
            Arc::new(issuer),
            par_store,
            flow,
            vec_nonempty![WALLET_REDIRECT_URI.parse().unwrap()],
        );
        (authorizing_issuer, sessions)
    }

    #[tokio::test]
    async fn complete_consent_happy_path() {
        let (authorizing_issuer, sessions) = authorizing_issuer_with_flow(flow());

        let context = WalletAuthorizationContext {
            redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
            scope: HashSet::from([WALLET_SCOPE.parse().unwrap()]),
            state: None,
            code_challenge: WALLET_CODE_CHALLENGE.to_string(),
            issuer_state: Some(USECASE_ID.to_string()),
        };

        let code = complete_consent(authorizing_issuer.flow(), &authorizing_issuer, &context)
            .await
            .unwrap();

        // An AuthCodeIssued session was written, keyed by the generated code, carrying the configured
        // document and the wallet's PKCE challenge.
        let session = sessions
            .get(&SessionToken::from(code))
            .await
            .unwrap()
            .expect("a session should have been written under the generated code");
        let IssuanceData::AuthCodeIssued(auth_code_issued) = session.data else {
            panic!("expected an AuthCodeIssued session");
        };
        assert_matches!(
            auth_code_issued.grant,
            Grant::AuthorizationCode {
                request_scope,
                wallet_code_challenge,
            } if request_scope == HashSet::from([WALLET_SCOPE.parse::<Scope>().unwrap()]) && wallet_code_challenge == WALLET_CODE_CHALLENGE
        );
        assert_eq!(auth_code_issued.credential_ids_and_documents.len().get(), 1);
        assert!(
            auth_code_issued
                .credential_ids_and_documents
                .as_ref()
                .iter()
                .all(|(_config_id, doc)| doc.credential_kind.attestation_type == ATTESTATION_TYPE)
        );
    }
}
