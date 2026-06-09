use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::Mutex;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
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
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorization_code_flow::WalletAuthorizationContext;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::WalletRedirect;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::IssuanceData;
use openid4vc::server_state::SessionStore;
use serde::Deserialize;
use tracing::warn;
use url::Url;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use uuid::Uuid;

use crate::pid::constants::PID_ADDRESS_GROUP;
use crate::pid::constants::PID_AGE_OVER_18;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BIRTH_DATE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_FAMILY_NAME;
use crate::pid::constants::PID_GIVEN_NAME;
use crate::pid::constants::PID_RECOVERY_CODE;
use crate::pid::constants::PID_RESIDENT_CITY;
use crate::pid::constants::PID_RESIDENT_COUNTRY;
use crate::pid::constants::PID_RESIDENT_HOUSE_NUMBER;
use crate::pid::constants::PID_RESIDENT_POSTAL_CODE;
use crate::pid::constants::PID_RESIDENT_STREET;

/// Mock [`AuthorizationCodeFlow`] for pid, providing a deterministic end-to-end stand-in for the
/// real DigiD-backed flow:
/// - `authorize` captures the wallet's [`WalletAuthorizationContext`] into an in-memory bridge keyed by a generated
///   `issuer_state`, then 302s to the issuer's own mock callback URL
///   `<issuer_url>/mock/digid/callback?code=<rnd>&state=<issuer_state>`.
/// - `callback_router` mounts that callback URL: it consumes the bridge entry, plants an `AuthCodeIssued` session via
///   [`AuthorizingIssuer::complete_authorization`] with copies of the preconfigured documents (fresh ids), and 302s the
///   user-agent to the wallet's universal link with the issuer-generated code + the wallet's original state.
pub struct MockPidAuthorizationCodeFlow {
    callback_uri: Url,
    documents: VecNonEmpty<IssuableDocument>,
    bridge: Mutex<HashMap<String, WalletAuthorizationContext>>,
}

impl MockPidAuthorizationCodeFlow {
    pub fn new(callback_uri: Url, documents: VecNonEmpty<IssuableDocument>) -> Self {
        Self {
            callback_uri,
            documents,
            bridge: Mutex::new(HashMap::new()),
        }
    }

    /// Mount the mock's `/mock/digid/callback` route on a fresh [`Router`]. The integration
    /// scaffolding merges this with the `openid4vc` layer's authorization and issuance routers.
    pub fn callback_router<K, L, S, N, PAS>(authorizing_issuer: Arc<AuthorizingIssuer<K, L, S, N, PAS, Self>>) -> Router
    where
        K: Send + Sync + 'static,
        L: Send + Sync + 'static,
        S: SessionStore<IssuanceData> + Send + Sync + 'static,
        N: Send + Sync + 'static,
        PAS: Send + Sync + 'static,
    {
        Router::new()
            .route("/mock/digid/callback", get(mock_digid_callback::<K, L, S, N, PAS>))
            .with_state(authorizing_issuer)
    }
}

#[derive(Deserialize)]
struct MockDigidCallbackQuery {
    state: String,
}

type MockDigidCallbackAuthorizingIssuer<K, L, S, N, PAS> =
    Arc<AuthorizingIssuer<K, L, S, N, PAS, MockPidAuthorizationCodeFlow>>;

async fn mock_digid_callback<K, L, S, N, PAS>(
    State(authorizing_issuer): State<MockDigidCallbackAuthorizingIssuer<K, L, S, N, PAS>>,
    Query(MockDigidCallbackQuery { state }): Query<MockDigidCallbackQuery>,
) -> Response
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    let entry = match flow.bridge.lock().unwrap().remove(&state) {
        Some(entry) => entry,
        None => {
            warn!("mock digid callback: unknown issuer_state");
            return (StatusCode::BAD_REQUEST, "unknown state").into_response();
        }
    };

    let documents = flow
        .documents
        .nonempty_iter()
        .cloned()
        .map(|mut document| {
            document.id = Uuid::new_v4();
            document
        })
        .collect();
    // The wallet's redirect_uri is known, so a completion failure bounces the user-agent back to the
    // wallet as an OAuth error redirect. Capture the redirect target before `entry` is consumed.
    let wallet_redirect = WalletRedirect::new(entry.redirect_uri.clone(), entry.state.clone());

    let result = authorizing_issuer.complete_authorization(documents, entry).await;
    if let Err(error) = &result {
        warn!("mock digid callback: complete_authorization failed: {error}");
    }

    let url = wallet_redirect.into_redirect_url(result, "server_error");
    (StatusCode::FOUND, [(header::LOCATION, String::from(url))]).into_response()
}

pub fn mock_issuable_documents_pid() -> VecNonEmpty<IssuableDocument> {
    vec_nonempty![
        IssuableDocument::try_new_with_random_id(
            Format::SdJwt,
            PID_ATTESTATION_TYPE.to_string(),
            eudi_nl_pid_example()
        )
        .unwrap(),
        IssuableDocument::try_new_with_random_id(
            Format::MsoMdoc,
            PID_ATTESTATION_TYPE.to_string(),
            eudi_nl_pid_example()
        )
        .unwrap(),
    ]
}

impl Default for MockPidAuthorizationCodeFlow {
    fn default() -> Self {
        Self::new(
            Url::parse("https://issuer.example.com/mock/digid/callback").unwrap(),
            mock_issuable_documents_pid(),
        )
    }
}

impl AuthorizationCodeFlow for MockPidAuthorizationCodeFlow {
    type Error = Infallible;

    async fn authorize(&self, context: WalletAuthorizationContext) -> Result<AuthorizeOutcome, Self::Error> {
        // Capture the wallet-side context into the in-memory bridge so the mock callback can
        // build the wallet-facing redirect later.
        let issuer_state = random_string(32);
        self.bridge
            .lock()
            .expect("mutex shouldn't be poisoned in test")
            .insert(issuer_state.clone(), context);

        // Redirect to our own mock callback URL with a fake upstream code + the issuer_state. The
        // callback handler will resolve the bridge entry and complete the flow.
        let mut redirect_url = self.callback_uri.clone();
        let query =
            serde_urlencoded::to_string([("code", random_string(32).as_str()), ("state", issuer_state.as_str())])
                .expect("encoding (code, state) query string should never fail");
        redirect_url.set_query(Some(&query));
        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }
}

/// Represents a single card with both PID and address claims
pub fn eudi_nl_pid_example() -> Attributes {
    Attributes::example([
        (
            vec![PID_GIVEN_NAME],
            AttributeValue::Text("Willeke Liselotte".to_string()),
        ),
        (vec![PID_FAMILY_NAME], AttributeValue::Text("De Bruijn".to_string())),
        (vec![PID_BIRTH_DATE], AttributeValue::Text("1997-05-10".to_string())),
        (vec![PID_AGE_OVER_18], AttributeValue::Bool(true)),
        (vec![PID_BSN], AttributeValue::Text("999991772".to_string())),
        (vec![PID_RECOVERY_CODE], AttributeValue::Text("1234567".to_string())),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            AttributeValue::Text("Turfmarkt".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            AttributeValue::Text("147".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
            AttributeValue::Text("2511 DP".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
            AttributeValue::Text("Den Haag".to_string()),
        ),
        (
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
            AttributeValue::Text("Nederland".to_string()),
        ),
    ])
}
