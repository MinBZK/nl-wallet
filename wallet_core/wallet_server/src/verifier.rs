use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::prelude::*;
use lazy_static::lazy_static;
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::log::{error, warn};
use url::Url;

use crate::settings::Settings;
use nl_wallet_mdoc::{
    holder::TrustAnchor,
    server_keys::{KeyRing, PrivateKey},
    server_state::{SessionState, SessionStore, SessionToken},
    utils::{
        serialization::cbor_serialize,
        x509::{Certificate, OwnedTrustAnchor},
    },
    verifier::{DisclosureData, StatusResponse, VerificationError, Verifier},
    ItemsRequest, SessionData,
};

lazy_static! {
    static ref UL_ENGAGEMENT: Url =
        Url::parse("walletdebuginteraction://wallet.edi.rijksoverheid.nl/disclosure/").unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("starting mdoc session failed: {0}")]
    StartSession(#[source] nl_wallet_mdoc::Error),
    #[error("process mdoc message error: {0}")]
    ProcessMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving status error: {0}")]
    Status(#[source] nl_wallet_mdoc::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("{}", self);
        match self {
            Error::StartSession(nl_wallet_mdoc::Error::Verification(_)) => StatusCode::BAD_REQUEST.into_response(),
            Error::StartSession(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::ProcessMdoc(nl_wallet_mdoc::Error::Verification(VerificationError::UnknownSessionId(_))) => {
                StatusCode::NOT_FOUND.into_response()
            }
            Error::ProcessMdoc(_) => StatusCode::BAD_REQUEST.into_response(),
            Error::Status(nl_wallet_mdoc::Error::Verification(VerificationError::UnknownSessionId(_))) => {
                StatusCode::NOT_FOUND.into_response()
            }
            Error::Status(_) => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

struct RelyingPartyKeyRing(HashMap<String, PrivateKey>);

impl KeyRing for RelyingPartyKeyRing {
    fn private_key(&self, usecase: &str) -> Option<&PrivateKey> {
        self.0.get(usecase)
    }
}

struct ApplicationState<S> {
    verifier: Verifier<RelyingPartyKeyRing, S>,
    internal_url: Url,
}

pub fn create_routers<S>(settings: Settings, sessions: S) -> anyhow::Result<(Router, Router)>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState {
        verifier: Verifier::new(
            settings.public_url.clone(),
            RelyingPartyKeyRing(
                settings
                    .usecases
                    .into_iter()
                    .map(|(usecase, keypair)| {
                        Ok((
                            usecase,
                            PrivateKey::new(
                                SigningKey::from_pkcs8_der(&BASE64_STANDARD.decode(&keypair.private_key)?)?,
                                Certificate::from(BASE64_STANDARD.decode(&keypair.certificate)?),
                            ),
                        ))
                    })
                    .collect::<anyhow::Result<HashMap<_, _>>>()?,
            ),
            sessions,
            settings
                .trust_anchors
                .into_iter()
                .map(|certificate| {
                    Ok(Into::<OwnedTrustAnchor>::into(&TryInto::<TrustAnchor>::try_into(
                        &Certificate::from(BASE64_STANDARD.decode(certificate)?),
                    )?))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        ),
        internal_url: settings.internal_url.unwrap_or(settings.public_url),
    });

    let wallet_router = Router::new()
        .route("/:session_id", post(session::<S>))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state.clone());

    let requester_router = Router::new()
        .route("/", post(start::<S>))
        .route("/:session_id/status", get(status::<S>))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

    Ok((wallet_router, requester_router))
}

async fn session<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_id): Path<SessionToken>,
    msg: Bytes,
) -> Result<Json<SessionData>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let disclosure_data = state
        .verifier
        .process_message(&msg, session_id)
        .await
        .map_err(Error::ProcessMdoc)?;

    Ok(Json(disclosure_data))
}

#[derive(Deserialize, Serialize)]
pub struct StartDisclosureRequest {
    pub usecase: String,
    pub items_requests: Vec<ItemsRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartDisclosureResponse {
    pub session_url: Url,
    pub engagement_url: Url,
}

async fn start<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let (session_id, engagement) = state
        .verifier
        .new_session(start_request.items_requests, start_request.usecase)
        .map_err(Error::StartSession)?;

    let session_url = state
        .internal_url
        .join(&format!("/sessions/{session_id}/status"))
        .unwrap();
    // base64 produces an alphanumberic value, cbor_serialize takes a Cbor_IntMap here
    let engagement_url = UL_ENGAGEMENT
        .join(&BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&engagement).unwrap()))
        .unwrap();
    // Note: return URL can be added by the RP

    Ok(Json(StartDisclosureResponse {
        session_url,
        engagement_url,
    }))
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_id): Path<SessionToken>,
) -> Result<Json<StatusResponse>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let status = state.verifier.status(&session_id).map_err(Error::Status)?;
    Ok(Json(status))
}
