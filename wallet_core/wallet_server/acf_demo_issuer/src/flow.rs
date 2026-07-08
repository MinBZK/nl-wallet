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
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use chrono::NaiveDate;
use crypto::utils::random_string;
use http_utils::urls::BaseUrl;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use itertools::Itertools;
use openid4vc::AuthorizationErrorCode;
use openid4vc::BodyOrRedirectErrorResponse;
use openid4vc::ErrorWithCode;
use openid4vc::RedirectError;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
use openid4vc::authorizing_issuer::RedirectQuery;
use openid4vc::issuable_document::CredentialKind;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::AuthRequestValues;
use openid4vc::issuer::IssuanceData;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Consumed;
use openid4vc::store::Store;
use openid4vc::token::AuthorizationCode;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use server_utils::store::StoreConnection;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::info;
use tracing::warn;
use url::Url;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;
use web_utils::headers::set_static_cache_control;
use web_utils::language::LANGUAGE_JS_SHA256;
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

    #[error("none of the usecase's documents match the requested credential kinds: {}", .0.iter().join(", "))]
    NoRequestedCredentialKinds(HashSet<CredentialKind>),

    #[error("authorization flow state expired before consent was submitted")]
    ExpiredState,

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

    #[error("error completing authorization: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

impl ErrorWithCode for Error {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::MissingIssuerState
            | Self::UnknownUsecase(_)
            | Self::NoRequestedCredentialKinds(_)
            | Self::ExpiredState => AuthorizationErrorCode::InvalidRequest,

            Self::StateBridge(_) => AuthorizationErrorCode::ServerError,

            Self::CompleteAuthorization(error) => error.error_code(),
        }
    }
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

/// Build the [`IssuableDocument`]s configured for a usecase, keeping only those whose
/// [`CredentialKind`] the wallet actually requested (as captured in the authorization context), and
/// substituting demo placeholders in the attributes so each issued document gets fresh values.
fn issuable_documents(
    usecase: &Usecase,
    requested_credential_kinds: &HashSet<CredentialKind>,
) -> Result<VecNonEmpty<IssuableDocument>, Error> {
    usecase
        .documents
        .iter()
        .filter(|template| requested_credential_kinds.contains(&template.credential_kind))
        .map(|template| {
            let IssuableDocumentTemplate {
                credential_kind,
                attributes,
            } = template.clone();
            let attributes = substitute_placeholders(attributes, &mut OsRng);
            IssuableDocument::try_new_with_random_id(credential_kind, attributes).expect("attributes cannot be empty")
        })
        .collect_vec()
        .try_into()
        .map_err(|_| Error::NoRequestedCredentialKinds(requested_credential_kinds.clone()))
}

/// Traverses all attributes (including those nested inside [`AttributeValue::Array`] or
/// [`Attribute::Nested`]) and replaces every `{{INSERT_RANDOM_VALUE}}` text value with a random string
/// of 10 digits, so each immediately-issued document gets fresh values.
///
/// Intentionally duplicated from `demo_issuer` rather than shared: the placeholder convention is
/// demo-specific and not worth lifting into a common crate. Unlike `demo_issuer`, this variant only
/// needs `{{INSERT_RANDOM_VALUE}}` (no acf usecase uses `{{INSERT_CURRENT_YEAR}}`).
fn substitute_placeholders(attributes: Attributes, rng: &mut impl RngCore) -> Attributes {
    let mut inner = attributes.into_inner();
    for attribute in inner.values_mut() {
        substitute_placeholders_in_attribute(attribute, rng);
    }
    inner.into()
}

fn substitute_placeholders_in_value(value: &mut AttributeValue, rng: &mut impl RngCore) {
    match value {
        AttributeValue::Text(text) if text.as_str() == "{{INSERT_RANDOM_VALUE}}" => {
            let n: u64 = rng.next_u64() % 10_000_000_000;
            *value = AttributeValue::Text(format!("{n:010}"));
        }
        AttributeValue::Array(elements) => {
            for element in elements {
                substitute_placeholders_in_value(element, rng);
            }
        }
        _ => {}
    }
}

fn substitute_placeholders_in_attribute(attribute: &mut Attribute, rng: &mut impl RngCore) {
    match attribute {
        Attribute::Single(value) => substitute_placeholders_in_value(value, rng),
        Attribute::Nested(map) => {
            for attr in map.values_mut() {
                substitute_placeholders_in_attribute(attr, rng);
            }
        }
    }
}

impl AuthorizationCodeFlow for DemoAuthorizationCodeFlow {
    type Error = Error;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        // The usecase is identified by the `issuer_state` the wallet echoes back from the offer.
        let usecase_id = context.issuer_state.as_deref().ok_or(Error::MissingIssuerState)?;
        let usecase = self
            .usecases
            .get(usecase_id)
            .ok_or_else(|| Error::UnknownUsecase(usecase_id.to_string()))?;

        match usecase.kind {
            UsecaseKind::Immediate => {
                let documents = issuable_documents(usecase, &context.credential_kinds)?;
                Ok(AuthorizeOutcome::Authorized(documents, Box::new(context)))
            }
            UsecaseKind::Consent => {
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
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.state_bridge_store.cleanup().await.map_err(Error::StateBridge)
    }
}

// Bundled CSS — placeholder in dev, full bundle in release. Dev builds are served via ServeDir.
#[cfg(not(debug_assertions))]
const CONSENT_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/consent.css"));

struct DocumentPreview {
    attestation_type: String,
    attributes: Vec<(String, String)>,
}

/// Render an attribute value for display on the consent page. The `start_date` attribute is stored
/// as ISO `YYYY-MM-DD` text and shown in the selected language's locale (localized month name); all
/// other values are shown verbatim. Unparseable dates fall back to the raw value.
fn format_attribute_value(key: &str, value: &AttributeValue, language: Language) -> String {
    if key == "start_date"
        && let AttributeValue::Text(text) = value
        && let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d")
    {
        // Render with the localized month name in day-first order, conventional for both en-GB
        // and nl-NL (e.g. "1 January 2025", "1 januari 2025").
        return date.format_localized("%-d %B %Y", language.chrono_locale()).to_string();
    }

    value.to_string()
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "consent.askama", escape = "html", ext = "html")]
struct ConsentTemplate<'a> {
    state: String,
    usecase: String,
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
                attributes: attributes
                    .flattened()
                    .into_iter()
                    .map(|(path, value)| {
                        let key = path.as_ref().join(".");
                        let display_value = format_attribute_value(&key, value, language);
                        // Show a hardcoded, translated label; fall back to the raw path if unlabelled.
                        let label = TRANSLATIONS[language]
                            .attribute_label(&key)
                            .map(str::to_string)
                            .unwrap_or(key);
                        (label, display_value)
                    })
                    .collect(),
            }
        })
        .collect();

    let available_languages = Language::iter().collect_vec();

    ConsentTemplate {
        state,
        usecase,
        doc_previews,
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &available_languages,
            language_js_sha256: &LANGUAGE_JS_SHA256,
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
) -> Result<Redirect, BodyOrRedirectErrorResponse<AuthorizationErrorCode>>
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    // The incoming request carries only the opaque `state` (the bridge key); the wallet's `redirect_uri` lives solely
    // inside the bridge entry. So once the entry is gone there is nothing left to send the user-agent back to. An
    // `Absent` entry (never existed, or deleted after the cleanup leeway) therefore dead-ends as a plain-text body. An
    // `Expired` entry, however, still carries the `redirect_uri`, so we send the user back to the wallet with an OAuth
    // error instead of a dead-end; this is exactly what the cleanup leeway buys us.
    let context: WalletAuthorizationContext = match flow.state_bridge_store.consume(state.as_str()).await {
        Ok(Consumed::Live(context)) => context,
        Ok(Consumed::Expired(context)) => {
            info!("consent submit: flow state expired; redirecting to wallet with error");

            return Err(
                RedirectError::new(Error::ExpiredState, context.request_values.redirect_uri, context.state).into(),
            );
        }
        Ok(Consumed::Absent) => {
            warn!("consent submit: unknown flow state");

            return Err(BodyOrRedirectErrorResponse::new_body(
                StatusCode::BAD_REQUEST,
                "unknown or expired state".to_string(),
            ));
        }
        Err(error) => {
            warn!("consent submit: state bridge consume failed: {error}");

            return Err(BodyOrRedirectErrorResponse::new_body(
                StatusCode::INTERNAL_SERVER_ERROR,
                "state bridge error".to_string(),
            ));
        }
    };

    let redirect_uri = context.request_values.redirect_uri.clone();
    let state = context.state;

    let code = match complete_consent(
        authorizing_issuer.as_ref(),
        context.issuer_state,
        &context.credential_kinds,
        context.request_values,
    )
    .await
    {
        Ok(code) => code,
        Err(error) => {
            warn!("consent callback: completion failed: {error}");

            // Return any error from completing consent as a 303 redirect to the `redirect_uri`, including the `state`
            // if it was present in the Authorization Request.
            return Err(RedirectError::new(error, redirect_uri, state).into());
        }
    };

    let url = RedirectQuery::encode(redirect_uri, &code, state.as_deref());

    Ok(Redirect::to(url.as_str()))
}

/// Build the configured documents for the entry's usecase and hand them to the `openid4vc` layer's
/// [`AuthorizingIssuer::complete_authorization`], ensuring a session is created keyed by a new authorization_code
/// containing the documents.
async fn complete_consent<K, L, S, N, PAS>(
    authorizing_issuer: &DemoAuthorizingIssuer<K, L, S, N, PAS>,
    issuer_state: Option<String>,
    credential_kinds: &HashSet<CredentialKind>,
    request_values: AuthRequestValues,
) -> Result<AuthorizationCode, Error>
where
    S: SessionStore<IssuanceData>,
{
    // The usecase is identified by the `issuer_state` the wallet echoed back from the offer; `authorize`
    // already validated it is present, but re-resolve it here as the single source of truth for the session.
    let usecase_id = issuer_state.ok_or(Error::MissingIssuerState)?;
    let usecase = authorizing_issuer
        .flow()
        .usecases
        .get(&usecase_id)
        .ok_or_else(|| Error::UnknownUsecase(usecase_id))?;

    let code = authorizing_issuer
        .complete_authorization(issuable_documents(usecase, credential_kinds)?, request_values)
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
    use itertools::Itertools;
    use openid4vc::authorization::VciAuthorizationRequest;
    use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
    use openid4vc::authorization_code_flow::AuthorizeOutcome;
    use openid4vc::authorization_code_flow::WalletAuthorizationContext;
    use openid4vc::authorizing_issuer::AuthorizingIssuer;
    use openid4vc::issuable_document::CredentialKind;
    use openid4vc::issuer::AuthRequestValues;
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
    use openid4vc::store::Store;
    use openid4vc::test::MOCK_ATTRS;
    use openid4vc::test::mock_type_metadata;
    use openid4vc::test::setup_mock_issuer_from_sd_jwt_metadata;
    use p256::ecdsa::SigningKey;
    use server_utils::store::StoreConnection;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use utils::vec_nonempty;

    use super::CONSENT_PATH;
    use super::DemoAuthorizationCodeFlow;
    use super::Error;
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
    const WALLET_CLIENT_ID: &str = "wallet-client-id";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";
    const WALLET_SCOPE: &str = "wallet-scope";
    const USECASE_ID: &str = "diploma";
    const ATTESTATION_TYPE: &str = "com.example.diploma";

    fn mock_attributes() -> Attributes {
        IndexMap::from_iter(MOCK_ATTRS.map(|(key, val)| {
            (
                key.to_string(),
                Attribute::Single(AttributeValue::Text(val.to_string())),
            )
        }))
        .into()
    }

    /// A usecase keyed by [`USECASE_ID`] with the given `kind` and one document per credential kind.
    fn usecase_with_credential_kinds(
        kind: UsecaseKind,
        credential_kinds: HashSet<CredentialKind>,
    ) -> HashMap<String, Usecase> {
        let documents = credential_kinds
            .into_iter()
            .map(|credential_kind| IssuableDocumentTemplate {
                credential_kind,
                attributes: mock_attributes(),
            })
            .collect_vec()
            .try_into()
            .unwrap();

        HashMap::from([(USECASE_ID.to_string(), Usecase { kind, documents })])
    }

    /// A single usecase keyed by [`USECASE_ID`] with the given `kind` and one SD-JWT document.
    fn usecases_with_kind(kind: UsecaseKind) -> HashMap<String, Usecase> {
        usecase_with_credential_kinds(kind, credential_kinds())
    }

    fn test_usecases() -> HashMap<String, Usecase> {
        usecases_with_kind(UsecaseKind::Consent)
    }

    fn flow_with_usecases(usecases: HashMap<String, Usecase>) -> DemoAuthorizationCodeFlow {
        DemoAuthorizationCodeFlow::new(StoreConnection::Memory, &CONSENT_BASE_URL.parse().unwrap(), usecases)
    }

    fn flow() -> DemoAuthorizationCodeFlow {
        flow_with_usecases(test_usecases())
    }

    fn test_context(issuer_state: Option<String>) -> WalletAuthorizationContext {
        WalletAuthorizationContext {
            state: None,
            issuer_state,
            credential_kinds: credential_kinds(),
            request_values: AuthRequestValues {
                client_id: WALLET_CLIENT_ID.to_string(),
                redirect_uri: WALLET_REDIRECT_URI.parse().unwrap(),
                scope: HashSet::from([WALLET_SCOPE.parse().unwrap()]),
                code_challenge: WALLET_CODE_CHALLENGE.to_string(),
            },
        }
    }

    fn credential_kinds() -> HashSet<CredentialKind> {
        HashSet::from([CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string())])
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

        let context = test_context(Some(USECASE_ID.to_string()));

        let code = complete_consent(
            &authorizing_issuer,
            context.issuer_state,
            &context.credential_kinds,
            context.request_values,
        )
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
            Grant::AuthorizationCode(request)
                if request.scope == HashSet::from([WALLET_SCOPE.parse::<Scope>().unwrap()])
                    && request.code_challenge == WALLET_CODE_CHALLENGE
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

    #[tokio::test]
    async fn authorize_happy_path() {
        let flow = flow();
        let context = test_context(Some(USECASE_ID.to_string()));

        let outcome = flow.authorize(context).await.unwrap();

        // The user-agent is redirected to the local consent page, carrying the flow-state token and the usecase.
        let AuthorizeOutcome::RedirectTo(url) = outcome else {
            panic!("expected a RedirectTo outcome");
        };
        assert_eq!(url.path(), format!("/{CONSENT_PATH}"));
        let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("usecase").map(String::as_str), Some(USECASE_ID));
        let flow_state = params.get("state").expect("redirect should carry a state token");

        // The wallet's request was stashed under that flow-state token.
        let stored = flow
            .state_bridge_store
            .consume(flow_state.as_str())
            .await
            .unwrap()
            .live()
            .expect("the wallet context should have been stored under the flow-state token");
        assert_eq!(stored.issuer_state.as_deref(), Some(USECASE_ID));
        assert_eq!(stored.request_values.client_id, WALLET_CLIENT_ID);
        assert_eq!(stored.request_values.redirect_uri, WALLET_REDIRECT_URI.parse().unwrap());
        assert_eq!(stored.request_values.code_challenge, WALLET_CODE_CHALLENGE);
        assert_eq!(
            stored.request_values.scope,
            HashSet::from([WALLET_SCOPE.parse().unwrap()])
        );
    }

    #[tokio::test]
    async fn authorize_missing_issuer_state() {
        let flow = flow();
        let context = test_context(None);

        let error = flow.authorize(context).await.unwrap_err();

        assert_matches!(error, Error::MissingIssuerState);
    }

    #[tokio::test]
    async fn authorize_unknown_usecase() {
        let flow = flow();
        let context = test_context(Some("nonexistent".to_string()));

        let error = flow.authorize(context).await.unwrap_err();

        assert_matches!(error, Error::UnknownUsecase(usecase) if usecase == "nonexistent");
    }

    #[tokio::test]
    async fn authorize_immediate_usecase() {
        let flow = flow_with_usecases(usecases_with_kind(UsecaseKind::Immediate));
        let context = test_context(Some(USECASE_ID.to_string()));

        let outcome = flow.authorize(context).await.unwrap();

        let AuthorizeOutcome::Authorized(documents, _) = outcome else {
            panic!("expected an Authorized outcome");
        };
        assert_eq!(documents.len().get(), 1);
        assert!(
            documents
                .iter()
                .all(|doc| doc.credential_kind.attestation_type == ATTESTATION_TYPE)
        );
    }

    #[tokio::test]
    async fn authorize_immediate_filters_documents_to_requested_credential_kinds() {
        // The usecase offers both an SD-JWT and an mso_mdoc document, but the wallet only requested
        // the SD-JWT credential kind, so only that document is issued.
        let flow = flow_with_usecases(usecase_with_credential_kinds(
            UsecaseKind::Immediate,
            HashSet::from([
                CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string()),
                CredentialKind::new(Format::MsoMdoc, ATTESTATION_TYPE.to_string()),
            ]),
        ));
        let mut context = test_context(Some(USECASE_ID.to_string()));
        context.credential_kinds = HashSet::from([CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string())]);

        let outcome = flow.authorize(context).await.unwrap();

        let AuthorizeOutcome::Authorized(documents, _) = outcome else {
            panic!("expected an Authorized outcome");
        };
        assert_eq!(
            documents.iter().map(|doc| &doc.credential_kind).collect_vec(),
            vec![&CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string())]
        );
    }

    #[tokio::test]
    async fn authorize_immediate_without_matching_credential_kind_errors() {
        // The usecase only offers an SD-JWT document, but the wallet requested an mso_mdoc one, so
        // there is nothing to issue.
        let flow = flow_with_usecases(usecase_with_credential_kinds(
            UsecaseKind::Immediate,
            HashSet::from([CredentialKind::new(Format::SdJwt, ATTESTATION_TYPE.to_string())]),
        ));
        let mut context = test_context(Some(USECASE_ID.to_string()));
        context.credential_kinds = HashSet::from([CredentialKind::new(Format::MsoMdoc, ATTESTATION_TYPE.to_string())]);

        let error = flow.authorize(context).await.unwrap_err();

        assert_matches!(error, Error::NoRequestedCredentialKinds(_));
    }
}
