use std::collections::HashSet;
use std::sync::Arc;

use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use axum::Form;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crypto::utils::random_string;
use hsm::service::HsmError;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use itertools::Either;
use itertools::Itertools;
use oauth::errors::AuthorizationErrorCode;
use oauth::errors::BodyOrRedirectErrorResponse;
use oauth::errors::ErrorWithCode;
use oauth::errors::RedirectError;
use oauth::token::AuthorizationCode;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
use openid4vc::authorizing_issuer::RedirectQuery;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::AuthRequestValues;
use openid4vc::issuer::IssuanceData;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Consumed;
use openid4vc::store::Store;
use serde::Deserialize;
use serde::Serialize;
use server_utils::keys::SecretKeyVariant;
use server_utils::store::StoreConnection;
use tracing::warn;
use url::Url;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use crate::pid::attributes::Persons;
use crate::pid::rulebook::constants::pid_attestation_type;
use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;

const LOGIN_STATE_LENGTH: usize = 32;

/// Errors raised by [`LoginAuthorizationCodeFlow`] on either half of the flow.
#[derive(Debug, thiserror::Error)]
pub enum LoginAuthorizationCodeFlowError {
    #[error("unsupported credential type(s) requested: {}", .0.iter().join(", "))]
    UnsupportedCredentialType(Vec<CredentialKind>),

    #[error("authorization flow state expired before the callback was received")]
    ExpiredState,

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

    #[error("could not find attributes for BSN")]
    NoAttributesFound,

    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,

    #[error("failed to compute BSN HMAC: {0}")]
    Hmac(#[source] HsmError),

    #[error("error completing authorization: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

impl ErrorWithCode for LoginAuthorizationCodeFlowError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::UnsupportedCredentialType(_) => AuthorizationErrorCode::InvalidScope,

            Self::ExpiredState => AuthorizationErrorCode::InvalidRequest,

            Self::StateBridge(_) | Self::NoAttributesFound | Self::InvalidIssuableDocuments | Self::Hmac(_) => {
                AuthorizationErrorCode::ServerError
            }

            Self::CompleteAuthorization(error) => error.error_code(),
        }
    }
}

/// One state-bridge entry, written at `/authorize` and consumed when the user selects a card.
#[derive(Serialize, Deserialize)]
struct StateBridgeEntry {
    context: WalletAuthorizationContext,
    formats: VecNonEmpty<Format>,
}

/// Query parameters sent by login page
#[derive(Deserialize)]
struct StateQuery {
    state: String,
}

/// Form parameters sent by login page when user selects a card.
#[derive(Deserialize)]
struct LoginForm {
    bsn: String,
}

/// Concrete [`AuthorizationCodeFlow`] for the pid_issuer.
///
/// Owns:
/// - the state-bridge store linking the issuer-generated `bridge_key` (sent to the upstream as `state`)
/// - the recovery-code HMAC key;
/// - the issuer's own login URL.
pub struct LoginAuthorizationCodeFlow {
    recovery_code_secret_key: SecretKeyVariant,
    issuer_metadata: PidIssuerMetadata,
    state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
    login_url: Url,
    persons: Persons,
}

impl LoginAuthorizationCodeFlow {
    /// Construct the flow, building its [`IssuerStateBridgeStore`] from `store_connection`. The
    /// store's entry type is an internal detail of this flow, so callers only supply the connection.
    pub fn new(
        recovery_code_secret_key: SecretKeyVariant,
        issuer_metadata: PidIssuerMetadata,
        store_connection: StoreConnection,
        login_url: Url,
        persons: Persons,
    ) -> Self {
        Self::new_with_store(
            recovery_code_secret_key,
            issuer_metadata,
            Arc::new(IssuerStateBridgeStore::new(store_connection)),
            login_url,
            persons,
        )
    }

    fn new_with_store(
        recovery_code_secret_key: SecretKeyVariant,
        issuer_metadata: PidIssuerMetadata,
        state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
        login_url: Url,
        persons: Persons,
    ) -> Self {
        Self {
            recovery_code_secret_key,
            issuer_metadata,
            state_bridge_store,
            login_url,
            persons,
        }
    }

    async fn fetch_issuable_documents(
        &self,
        bsn: &str,
        formats: VecNonEmpty<Format>,
    ) -> Result<VecNonEmpty<IssuableDocument>, LoginAuthorizationCodeFlowError> {
        // Fetch person from the in-memory store
        let person = self
            .persons
            .as_ref()
            .get(bsn)
            .ok_or_else(|| LoginAuthorizationCodeFlowError::NoAttributesFound)?
            .clone();

        // Generate recovery code and issuer metadata
        let recovery_code = recovery_code(bsn, &self.recovery_code_secret_key).await?;
        let issuer_metadata = self.issuer_metadata.clone();

        // Create an `IssuableDocument` for each requested format.
        let format_count = formats.len();
        let issuable_documents = formats
            .into_nonempty_iter()
            .zip(utils::vec_at_least::repeat_n(
                (person, recovery_code, issuer_metadata),
                format_count,
            ))
            .map(|(format, (person, recovery_code, issuer_metadata))| {
                let attestation_type = pid_attestation_type(format);
                let attributes = match format {
                    Format::MsoMdoc => person.into_mdoc_attributes(recovery_code, issuer_metadata),
                    Format::SdJwt => person.into_sd_jwt_attributes(recovery_code, issuer_metadata),
                };

                IssuableDocument::try_new_with_random_id(
                    CredentialKind::new(format, attestation_type.to_owned()),
                    attributes,
                )
            })
            .collect::<Result<_, _>>()
            .map_err(|_| LoginAuthorizationCodeFlowError::InvalidIssuableDocuments)?;

        Ok(issuable_documents)
    }
}

impl AuthorizationCodeFlow for LoginAuthorizationCodeFlow {
    type Error = LoginAuthorizationCodeFlowError;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        // Return an error if any of the attestation types are not the PID attestation type and retain only the
        // requested formats.
        let (formats, unsupported): (HashSet<_>, HashSet<_>) = context
            .credential_kinds
            .clone()
            .into_iter()
            .partition_map(|credential_kind| {
                if credential_kind.attestation_type == pid_attestation_type(credential_kind.format) {
                    Either::Left(credential_kind.format)
                } else {
                    Either::Right(credential_kind)
                }
            });

        if !unsupported.is_empty() {
            return Err(Self::Error::UnsupportedCredentialType(
                unsupported.into_iter().collect(),
            ));
        }

        let formats = formats.into_iter().collect_vec().try_into().expect(
            "deduplicated formats from non-emtpy formats and types should never be empty when there are no \
             unsupported attestation types",
        );

        // Generate a random `login_state` to keep track of the flow
        let login_state = random_string(LOGIN_STATE_LENGTH);

        // Create the login URL to which the wallet should redirect
        let mut login_url = self.login_url.clone();
        login_url.query_pairs_mut().append_pair("state", login_state.as_str());

        // Retain the wallet-side context so the callback can redirect the user-agent back to the
        // wallet and the `openid4vc` layer's /token handler can verify the wallet's PKCE challenge.
        let entry = StateBridgeEntry { context, formats };
        self.state_bridge_store
            .store(login_state, entry)
            .await
            .map_err(Self::Error::StateBridge)?;

        Ok(AuthorizeOutcome::RedirectTo(login_url))
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        self.state_bridge_store
            .cleanup()
            .await
            .map_err(Self::Error::StateBridge)
    }
}

/// Issuer that incorporates a login submit handle that fetches the chosen BSN from the in-memory
/// store and hands them to the `openid4vc` layer's [`AuthorizingIssuer::complete_authorization`]
/// which generates the issuer-side authorization code and writes the `AuthCodeIssued` session.
/// Errors surface to the wallet as an OAuth error redirect, since the wallet's redirect_uri is
/// known by then.
type LoginAuthorizingIssuer<K, L, S, N, PAS> = Arc<AuthorizingIssuer<K, L, S, N, PAS, LoginAuthorizationCodeFlow>>;

/// Create a [`Router`] that handles the login select on the given path.
pub fn login_router<K, L, S, N, PAS>(authorizing_issuer: LoginAuthorizingIssuer<K, L, S, N, PAS>, path: &str) -> Router
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    Router::new()
        .route(path, post(login_submit))
        .with_state(authorizing_issuer)
}

async fn login_submit<K, L, S, N, PAS>(
    State(authorizing_issuer): State<LoginAuthorizingIssuer<K, L, S, N, PAS>>,
    Query(StateQuery { state }): Query<StateQuery>,
    Form(LoginForm { bsn }): Form<LoginForm>,
) -> Result<Redirect, BodyOrRedirectErrorResponse<AuthorizationErrorCode>>
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    let entry: StateBridgeEntry = match flow.state_bridge_store.consume(state.as_str()).await {
        Ok(Consumed::Live(entry)) => entry,
        Ok(Consumed::Expired(entry)) => {
            warn!("login callback: bridge entry expired; redirecting to wallet with error");

            let WalletAuthorizationContext {
                state, request_values, ..
            } = entry.context;

            return Err(RedirectError::new(
                LoginAuthorizationCodeFlowError::ExpiredState,
                Box::new(request_values.redirect_uri),
                state,
            )
            .into());
        }
        Ok(Consumed::Absent) => {
            warn!("login callback: unknown bridge key");

            return Err(BodyOrRedirectErrorResponse::new_body(
                StatusCode::BAD_REQUEST,
                "unknown or expired state".to_string(),
            ));
        }
        Err(error) => {
            warn!("login callback: state bridge consume failed: {error}");

            return Err(BodyOrRedirectErrorResponse::new_body(
                StatusCode::INTERNAL_SERVER_ERROR,
                "state bridge error".to_string(),
            ));
        }
    };

    let redirect_uri = entry.context.request_values.redirect_uri.clone();
    let state = entry.context.state;

    let code = match complete_login_authorization(&authorizing_issuer, entry.context.request_values, bsn, entry.formats)
        .await
    {
        Ok(code) => code,
        Err(error) => {
            warn!("login callback: completion failed: {error}");

            // Return any error from completing login as a 303 redirect to the `redirect_uri`, including the `state`
            // if it was present in the Authorization Request.
            return Err(RedirectError::new(error, Box::new(redirect_uri), state).into());
        }
    };

    let url = RedirectQuery::encode(redirect_uri, &code, state.as_deref());

    Ok(Redirect::to(url.as_str()))
}

/// Create issuable documents for the BSN and hand them to the `openid4vc` layer's
/// [`AuthorizingIssuer::complete_authorization`], which generates the issuer-side authorization code,
/// writes the `AuthCodeIssued` session, and returns the code.
async fn complete_login_authorization<K, L, S, N, PAS>(
    authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, LoginAuthorizationCodeFlow>,
    auth_request_values: AuthRequestValues,
    bsn: String,
    formats: VecNonEmpty<Format>,
) -> Result<AuthorizationCode, LoginAuthorizationCodeFlowError>
where
    S: SessionStore<IssuanceData>,
{
    let issuable_documents = authorizing_issuer
        .flow()
        .fetch_issuable_documents(&bsn, formats)
        .await?;

    authorizing_issuer
        .complete_authorization(issuable_documents, auth_request_values)
        .await
        .map_err(LoginAuthorizationCodeFlowError::CompleteAuthorization)
}

/// The recovery code of a person (an HMAC over the BSN)
async fn recovery_code(bsn: &str, secret_key: &SecretKeyVariant) -> Result<String, LoginAuthorizationCodeFlowError> {
    let hmac = secret_key
        .sign_hmac(bsn.as_bytes())
        .await
        .map_err(LoginAuthorizationCodeFlowError::Hmac)?;
    Ok(hex::encode(hmac))
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use issuer_common::state_bridge_store::IssuerStateBridgeStore;
    use itertools::Itertools;
    use oauth::issuer_identifier::IssuerIdentifier;
    use oauth::par::PAR_TTL;
    use oauth::scope::Scope;
    use openid4vc::authorization::VciAuthorizationRequest;
    use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
    use openid4vc::authorization_code_flow::AuthorizeOutcome;
    use openid4vc::authorization_code_flow::WalletAuthorizationContext;
    use openid4vc::authorizing_issuer::AuthorizingIssuer;
    use openid4vc::credential_configurations::CredentialConfigurationFormat;
    use openid4vc::credential_configurations::CredentialConfigurationTypeMetadata;
    use openid4vc::issuer::AuthRequestValues;
    use openid4vc::issuer::Grant;
    use openid4vc::issuer::IssuanceData;
    use openid4vc::metadata::issuer_metadata::CredentialMetadata;
    use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
    use openid4vc::nonce::memory_store::MemoryNonceStore;
    use openid4vc::server_state::MemorySessionStore;
    use openid4vc::server_state::SessionStore;
    use openid4vc::server_state::SessionToken;
    use openid4vc::store::MemoryStore;
    use openid4vc::store::Store;
    use openid4vc::test::MockObtainingStatusListService;
    use openid4vc::test::setup_mock_issuer_with_metadata;
    use p256::ecdsa::SigningKey;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use server_utils::keys::SecretKeyVariant;
    use server_utils::settings::SecretKey;
    use server_utils::store::StoreConnection;
    use utils::path::prefix_local_path;
    use utils::vec_nonempty;

    use super::LoginAuthorizationCodeFlow;
    use super::LoginAuthorizationCodeFlowError;
    use super::StateBridgeEntry;
    use super::complete_login_authorization;
    use super::recovery_code;
    use crate::pid::mock::MOCK_PERSON;
    use crate::pid::rulebook::constants::PID_MDOC_DOCTYPE;
    use crate::pid::rulebook::constants::PID_NL_SD_JWT_VCT;
    use crate::pid::rulebook::constants::pid_attestation_type;
    use crate::pid::rulebook::country_code::IsoCountryCode;
    use crate::pid::rulebook::issuer_metadata::PidIssuerMetadata;

    /// The in-memory [`AuthorizingIssuer`] flavour used by the callback tests: the mock inner issuer
    /// from `openid4vc::test` wrapped around a mock-backed flow.
    type TestAuthorizingIssuer = AuthorizingIssuer<
        SigningKey,
        MockObtainingStatusListService,
        MemorySessionStore<IssuanceData>,
        MemoryNonceStore,
        MemoryStore<String, VciAuthorizationRequest>,
        LoginAuthorizationCodeFlow,
    >;

    const LOGIN_URL: &str = "https://issuer.example.com/login";
    const WALLET_REDIRECT_URI: &str = "https://wallet.example.com/callback";
    const WALLET_STATE: &str = "wallet-state";
    const WALLET_SCOPE: &str = "wallet-scope";
    const WALLET_CODE_CHALLENGE: &str = "wallet-code-challenge";

    static NL_PID_METADATA: LazyLock<TypeMetadataDocuments> = LazyLock::new(|| {
        TypeMetadataDocuments::new(vec_nonempty![
            fs::read(prefix_local_path(Path::new("resources/test/metadata/eudi_pid_1.json"))).unwrap(),
            fs::read(prefix_local_path(Path::new(
                "resources/test/metadata/eudi_pid_nl_1.json"
            )))
            .unwrap()
        ])
    });

    static PID_MDOC_METADATA: LazyLock<CredentialMetadata> = LazyLock::new(|| {
        let json = fs::read_to_string(prefix_local_path(Path::new(
            "resources/test/metadata/eu.europa.ec.eudi.pid.1.json",
        )))
        .unwrap();
        serde_json::from_str(&json).unwrap()
    });

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

    fn flow_with_store(
        state_bridge_store: Arc<IssuerStateBridgeStore<StateBridgeEntry>>,
    ) -> LoginAuthorizationCodeFlow {
        let issuer_metadata = PidIssuerMetadata {
            issuing_country: IsoCountryCode::Netherlands,
            issuing_authority: "Test Issuing Authority".to_owned(),
        };
        LoginAuthorizationCodeFlow::new_with_store(
            recovery_code_secret_key(),
            issuer_metadata,
            state_bridge_store,
            LOGIN_URL.parse().unwrap(),
            MOCK_PERSON.clone().into(),
        )
    }

    /// Wrap a flow in an [`AuthorizingIssuer`] backed by an in-memory issuer + session store, so the
    /// callback path (which writes a session via `complete_authorization`) can be exercised. Returns
    /// the session store so tests can read the written session back.
    ///
    /// Note that the [`AuthorizingIssuer`] needs to be configured with the real PID SD-JWT VC Type Metadata, as
    /// completing the callback will lead to the [`IssuableDocument`]s being validated against it.
    fn authorizing_issuer_with_flow(
        flow: LoginAuthorizationCodeFlow,
    ) -> (TestAuthorizingIssuer, Arc<MemorySessionStore<IssuanceData>>) {
        let sessions = Arc::new(MemorySessionStore::default());

        let (issuer, _, _, _, _) = setup_mock_issuer_with_metadata(
            IssuerIdentifier::try_new("https://issuer.example.com".to_string()).unwrap(),
            vec![
                CredentialConfigurationFormat::new_sd_jwt_type_metadata(
                    CredentialConfigurationTypeMetadata::try_new(PID_NL_SD_JWT_VCT, NL_PID_METADATA.clone()).unwrap(),
                ),
                CredentialConfigurationFormat::new_mdoc(PID_MDOC_DOCTYPE.to_owned(), PID_MDOC_METADATA.clone()),
            ],
            true,
            Arc::clone(&sessions),
        );
        let authorizing_issuer = AuthorizingIssuer::new(
            Arc::new(issuer),
            MemoryStore::new(PAR_TTL),
            flow,
            vec_nonempty![WALLET_REDIRECT_URI.parse().unwrap()],
        );

        (authorizing_issuer, sessions)
    }

    /// Builds the wallet-side context `AuthorizationCodeFlow::authorize` receives, carrying the
    /// `credential_kinds` the `openid4vc` layer derived from the request's scopes.
    fn wallet_context(credential_kinds: HashSet<CredentialKind>) -> WalletAuthorizationContext {
        WalletAuthorizationContext {
            state: Some(WALLET_STATE.to_string()),
            issuer_state: None,
            credential_kinds,
            request_values: AuthRequestValues::new(
                MOCK_WALLET_CLIENT_ID.to_string(),
                WALLET_REDIRECT_URI.parse().unwrap(),
                WALLET_CODE_CHALLENGE.to_string(),
                HashSet::from([WALLET_SCOPE.parse().unwrap()]),
            ),
        }
    }

    fn state_bridge_entry() -> StateBridgeEntry {
        StateBridgeEntry {
            context: WalletAuthorizationContext {
                state: Some(WALLET_STATE.to_string()),
                issuer_state: None,
                credential_kinds: HashSet::from([
                    CredentialKind::new(Format::MsoMdoc, PID_MDOC_DOCTYPE.to_owned()),
                    CredentialKind::new(Format::SdJwt, PID_NL_SD_JWT_VCT.to_owned()),
                ]),
                request_values: AuthRequestValues::new(
                    MOCK_WALLET_CLIENT_ID.to_string(),
                    WALLET_REDIRECT_URI.parse().unwrap(),
                    WALLET_CODE_CHALLENGE.to_string(),
                    HashSet::from([
                        Scope::try_new(format!("{PID_MDOC_DOCTYPE}_{}", Format::MsoMdoc)).unwrap(),
                        Scope::try_new(format!("{PID_NL_SD_JWT_VCT}_{}", Format::SdJwt)).unwrap(),
                    ]),
                ),
            },
            formats: vec_nonempty![Format::MsoMdoc, Format::SdJwt],
        }
    }

    #[tokio::test]
    async fn test_recovery_code() {
        // Static key, if changed, update the expected recovery code
        let key: Vec<_> = (0..32).collect();
        let secret_key = SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: key.clone().try_into().unwrap(),
            },
            None,
        )
        .unwrap();

        let recovery_code = recovery_code(&MOCK_PERSON.bsn, &secret_key).await.unwrap();
        assert_eq!(
            &recovery_code,
            "298134f31030565bada59326e608e86a99801ee9c87dfe338828a17f9eb4d5dc"
        );
    }

    #[tokio::test]
    async fn authorize_sets_bridge_entry() {
        let bridge = memory_bridge_store();
        let flow = flow_with_store(Arc::clone(&bridge));

        let context = wallet_context(HashSet::from([
            CredentialKind::new(Format::SdJwt, PID_NL_SD_JWT_VCT.to_owned()),
            CredentialKind::new(Format::MsoMdoc, PID_MDOC_DOCTYPE.to_owned()),
            // Test deduplication.
            CredentialKind::new(Format::SdJwt, PID_NL_SD_JWT_VCT.to_owned()),
        ]));
        let wallet_code_challenge = context.request_values.code_challenge.clone();

        let outcome = flow.authorize(context).await.unwrap();
        let AuthorizeOutcome::RedirectTo(redirect_url) = outcome else {
            panic!("authorize should redirect the user-agent to the login page");
        };

        // The flow redirects to the login URL with a state parameter.
        assert!(redirect_url.as_str().starts_with(LOGIN_URL));
        let mut pairs = redirect_url.query_pairs();
        let (query_param_key, login_state) = pairs.next().expect("should have one query param");
        assert_eq!(query_param_key, "state");
        assert!(pairs.next().is_none());

        let entry: StateBridgeEntry = bridge
            .consume(login_state.as_ref())
            .await
            .unwrap()
            .live()
            .expect("a bridge entry should be stored under the bridge key");

        assert_eq!(entry.context.state.as_deref(), Some(WALLET_STATE));
        assert_eq!(entry.context.request_values.client_id, MOCK_WALLET_CLIENT_ID);
        assert_eq!(entry.context.request_values.redirect_uri.as_str(), WALLET_REDIRECT_URI);
        assert_eq!(entry.context.request_values.code_challenge, wallet_code_challenge);
        assert_eq!(
            entry.context.request_values.scope,
            HashSet::from([WALLET_SCOPE.parse().unwrap()])
        );
        assert!(
            entry
                .formats
                .iter()
                .map(ToString::to_string)
                .sorted()
                .eq(["dc+sd-jwt", "mso_mdoc"])
        );
    }

    #[tokio::test]
    async fn test_authorize_error_unsupported_attestation_type() {
        let bridge = memory_bridge_store();
        let flow = flow_with_store(Arc::clone(&bridge));

        let context = wallet_context(HashSet::from([
            CredentialKind::new(Format::SdJwt, "foo".to_string()),
            CredentialKind::new(Format::MsoMdoc, "bar".to_string()),
            CredentialKind::new(Format::SdJwt, "not_supported".to_string()),
        ]));

        let error = flow
            .authorize(context)
            .await
            .expect_err("starting authorization flow should fail");

        assert_matches!(
            error,
            LoginAuthorizationCodeFlowError::UnsupportedCredentialType(unsupported)
                if unsupported
                    .iter()
                    .map(|credential_kind| &credential_kind.attestation_type)
                    .sorted()
                    .eq(["bar", "foo", "not_supported"])
        )
    }

    #[tokio::test]
    async fn complete_login_happy_path() {
        let flow = flow_with_store(memory_bridge_store());
        let (authorizing_issuer, sessions) = authorizing_issuer_with_flow(flow);
        let StateBridgeEntry { context, formats } = state_bridge_entry();

        let expected_auth_request_values = context.request_values.clone();

        let code = complete_login_authorization(
            &authorizing_issuer,
            context.request_values,
            MOCK_PERSON.bsn.clone(),
            formats,
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
        assert_matches!(
            auth_code_issued.grant,
            Grant::AuthorizationCode(auth_request_values) if auth_request_values == expected_auth_request_values
        );
        assert_eq!(auth_code_issued.credential_ids_and_documents.len().get(), 2);
        assert!(
            auth_code_issued
                .credential_ids_and_documents
                .as_ref()
                .iter()
                .all(|(_config_id, document)| {
                    document.credential_kind.attestation_type == pid_attestation_type(document.credential_kind.format)
                })
        );
    }

    #[tokio::test]
    async fn complete_login_rejects_when_no_attributes_found() {
        let flow = flow_with_store(memory_bridge_store());
        let (authorizing_issuer, _sessions) = authorizing_issuer_with_flow(flow);
        let StateBridgeEntry { context, formats } = state_bridge_entry();

        let error =
            complete_login_authorization(&authorizing_issuer, context.request_values, "unknown".into(), formats)
                .await
                .unwrap_err();

        assert_matches!(error, LoginAuthorizationCodeFlowError::NoAttributesFound);
    }
}
