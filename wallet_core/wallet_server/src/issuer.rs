use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::{delete, post},
    Form, Json, Router, TypedHeader,
};
use base64::prelude::*;
use http::StatusCode;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::{KeyRing, PrivateKey},
    server_state::{MemorySessionStore, SessionState, SessionStore},
};
use openid4vc::{
    credential::{CredentialRequests, CredentialResponses},
    token::{TokenRequest, TokenRequestGrantType, TokenResponseWithPreviews},
};
use url::Url;
use wallet_common::utils::sha256;

use crate::{log_requests::log_request_response, settings::Settings, verifier::Error};

use crate::issuance_state::*;

struct Issuer<K> {
    sessions: MemorySessionStore<IssuanceData>,
    private_keys: K,
    credential_issuer_identifier: Url,
    wallet_client_ids: Vec<String>,
}

struct ApplicationState<A, K> {
    issuer: Issuer<K>,
    attr_service: A,
}

#[async_trait]
pub trait AttributeService: Send + Sync + 'static {
    type Error: std::fmt::Debug;
    type Settings;

    async fn new(settings: &Self::Settings) -> Result<Self, Self::Error>
    where
        Self: Sized;

    async fn attributes(
        &self,
        session: &SessionState<Created>,
        token_request: TokenRequest,
    ) -> Result<Vec<UnsignedMdoc>, Self::Error>;
}

/// A deterministic function to convert an authorization code to an access token.
/// This allows us to store sessions in the session store by their access code while being able to look it up
/// using either the code or the access token.
pub fn code_to_access_token(code_hash_key: &[u8], code: &str) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(sha256([code_hash_key, code.as_bytes()].concat().as_slice()))
}

pub struct IssuerKeyRing(pub HashMap<String, PrivateKey>);

impl KeyRing for IssuerKeyRing {
    fn private_key(&self, usecase: &str) -> Option<&PrivateKey> {
        self.0.get(usecase)
    }
}

pub async fn create_issuance_router<A: AttributeService>(
    settings: Settings,
    attr_service: A,
) -> anyhow::Result<Router> {
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::<IssuerKeyRing> {
            sessions: MemorySessionStore::new(),
            credential_issuer_identifier: settings.issuer.credential_issuer_identifier.clone(),
            wallet_client_ids: settings.issuer.wallet_client_ids.clone().into_iter().collect(),
            private_keys: IssuerKeyRing(
                settings
                    .issuer
                    .private_keys
                    .into_iter()
                    .map(|(doctype, keypair)| {
                        Ok((
                            doctype,
                            PrivateKey::from_der(&keypair.private_key.0, &keypair.certificate.0)?,
                        ))
                    })
                    .collect::<anyhow::Result<HashMap<_, _>>>()?,
            ),
        },
        attr_service,
    });

    let issuance_router = Router::new()
        .route("/token", post(token))
        .route("/batch_credential", post(batch_credential))
        .route("/batch_credential", delete(refuse_issuance))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(log_request_response))
        .with_state(application_state);

    Ok(issuance_router)
}

async fn token<A: AttributeService, K: KeyRing>(
    State(state): State<Arc<ApplicationState<A, K>>>,
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponseWithPreviews>, Error> {
    if !matches!(
        token_request.grant_type,
        TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code: _ }
    ) {
        panic!("token request must be of type pre-authorized_code");
    }

    let code = token_request.code();

    // Retrieve the session from the session store, if present. It need not be, depending on the implementation of the
    // attribute service.
    let session = state
        .issuer
        .sessions
        .get(&code.clone().into())
        .await
        .unwrap()
        .unwrap_or(SessionState::<IssuanceData>::new(
            code.clone().into(),
            IssuanceData::Created(Created {
                code,
                unsigned_mdocs: None,
            }),
        ));
    let session = Session::<Created>::from_enum(session).unwrap();

    // TODO remove session from store, if present, so that the code is now consumed

    let result = session.process_token_request(token_request, &state.attr_service).await;

    let (response, next) = match result {
        Ok((response, next)) => (Ok(Json(response)), next.into_enum()),
        Err((err, next)) => (Err(err), next.into_enum()),
    };

    state.issuer.sessions.write(&next).await.unwrap();

    response
}

async fn batch_credential<A: AttributeService, K: KeyRing>(
    State(state): State<Arc<ApplicationState<A, K>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, Error> {
    let token = authorization_header.token();

    // The access token should be a random string with the authorization code appended to it, so that we can
    // use the code suffix to retrieve the session from the session store. If what the user provided in the
    // authorization header is shorter than that, we can just use unwrap_or_default(), since no session will
    // ever be index by the empty string.
    let code = token.get(32..).unwrap_or_default().to_string().into();

    let session = state.issuer.sessions.get(&code).await.unwrap().unwrap(); // TODO
    let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

    let (response, next) = session
        .process_response(
            credential_requests,
            token.to_string(),
            &state.issuer.private_keys,
            &state.issuer.credential_issuer_identifier,
            &state.issuer.wallet_client_ids,
        )
        .await;

    state.issuer.sessions.write(&next.into_enum()).await.unwrap();

    response.map(Json)
}

async fn refuse_issuance<A: AttributeService, K: KeyRing>(
    State(state): State<Arc<ApplicationState<A, K>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, Error> {
    let token = authorization_header.token().to_string().into();
    let session = state.issuer.sessions.get(&token).await.unwrap().unwrap(); // TODO
    let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

    let next = session.transition(Done {
        session_result: SessionResult::Cancelled,
    });

    state.issuer.sessions.write(&next.into_enum()).await.unwrap();

    Ok(StatusCode::NO_CONTENT)
}
