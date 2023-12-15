use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::{delete, post},
    Form, Json, Router, TypedHeader,
};
use http::StatusCode;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    server_keys::{KeyRing, PrivateKey},
    server_state::{MemorySessionStore, SessionState, SessionStore},
};
use openid4vc::{
    credential::{CredentialRequests, CredentialResponses},
    token::{TokenRequest, TokenResponseWithPreviews},
};
use tracing::warn;

use crate::{issuance_state, log_requests::log_request_response, settings::Settings};

use crate::issuance_state::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ProcessSession(#[from] issuance_state::Error),
}

// TODO proper error handling
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("{}", self);
        match self {
            Error::ProcessSession(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

struct ApplicationState<A, K, S> {
    issuer: Issuer<A, K, S>,
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
        issuer: Issuer::new(
            MemorySessionStore::new(),
            attr_service,
            IssuerKeyRing(
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
            settings.issuer.credential_issuer_identifier.clone(),
            settings.issuer.wallet_client_ids.clone().into_iter().collect(),
        ),
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

async fn token<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    Form(token_request): Form<TokenRequest>,
) -> Result<Json<TokenResponseWithPreviews>, Error>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let response = state.issuer.process_token_request(token_request).await?;
    Ok(Json(response))
}

async fn batch_credential<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, Error>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    let token = authorization_header.token();
    let response = state
        .issuer
        .process_batch_credential(token, credential_requests)
        .await?;
    Ok(Json(response))
}

async fn refuse_issuance<A, K, S>(
    State(state): State<Arc<ApplicationState<A, K, S>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, Error>
where
    A: AttributeService,
    K: KeyRing,
    S: SessionStore<Data = SessionState<IssuanceData>> + Send + Sync + 'static,
{
    state
        .issuer
        .process_refuse_issuance(authorization_header.token())
        .await
        .unwrap();
    Ok(StatusCode::NO_CONTENT)
}
