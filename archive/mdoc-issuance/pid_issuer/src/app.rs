use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, TypedHeader,
};
use base64::prelude::*;
use futures::TryFutureExt;
use http::StatusCode;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, warn};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    issuer::{IssuanceData, Issuer},
    server_keys::{KeyPair, SingleKeyRing},
    server_state::MemorySessionStore,
    ServiceEngagement,
};

use crate::{digid, settings::Settings};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC client error: {0}")]
    Digid(#[from] digid::Error),
    #[error("starting mdoc session failed: {0}")]
    StartMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("could not find attributes for BSN")]
    NoAttributesFound,
    #[error("mdoc session error: {0}")]
    Mdoc(#[source] nl_wallet_mdoc::Error),
}

// TODO: Implement proper error handling.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);

        let status_code = match self {
            Error::NoAttributesFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, format!("{}", self)).into_response()
    }
}

/// Given a BSN, determine the attributes to be issued. Contract for the BRP query.
pub trait AttributesLookup {
    fn attributes(&self, bsn: &str) -> Option<Vec<UnsignedMdoc>>;
}

/// Given an access token, lookup a BSN: a trait modeling the OIDC [`Client`](crate::openid::Client).
/// Contract for the DigiD bridge.
#[trait_variant::make(BsnLookup: Send)]
pub trait LocalBsnLookup {
    async fn bsn(&self, access_token: &str) -> Result<String, digid::Error>;
}

struct ApplicationState<A, B> {
    attributes_lookup: A,
    openid_client: B,
    issuer: Issuer<SingleKeyRing, MemorySessionStore<IssuanceData>>,
}

pub async fn create_router<A, B>(settings: Settings, attributes_lookup: A, openid_client: B) -> anyhow::Result<Router>
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    debug!("DigiD issuer discovered, starting HTTP server");

    let key = SingleKeyRing(KeyPair::from_der(
        &BASE64_STANDARD.decode(&settings.issuer_key.private_key)?,
        &BASE64_STANDARD.decode(&settings.issuer_key.certificate)?,
    )?);

    let mut public_url = settings.public_url;
    if !public_url.as_str().ends_with('/') {
        // If the url does not have a trailing slash then .join() will remove its last path segment
        // before appending its argument (which is also why we can't use .join() for appending this slash).
        // We can use .unwrap() because this errors only happens "if the scheme and `:` delimiter
        // are not followed by a `/` slash".
        public_url.path_segments_mut().unwrap().push("/");
    }
    let public_url = public_url.join("mdoc/")?;

    let application_state = Arc::new(ApplicationState {
        attributes_lookup,
        openid_client,
        issuer: Issuer::new(public_url, key, MemorySessionStore::new()),
    });

    let app = Router::new()
        .route("/health", get(|| async {}))
        .route("/mdoc/:session_token", post(mdoc_route))
        .route("/start", post(start_route))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

    Ok(app)
}

async fn mdoc_route<A, B>(
    State(state): State<Arc<ApplicationState<A, B>>>,
    Path(session_token): Path<String>,
    msg: Bytes,
) -> Result<Vec<u8>, Error> {
    let response = state
        .issuer
        .process_message(session_token.into(), &msg)
        .await
        .map_err(Error::Mdoc)?;
    Ok(response)
}

async fn start_route<A, B>(
    State(state): State<Arc<ApplicationState<A, B>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ServiceEngagement>, Error>
where
    A: AttributesLookup,
    B: LocalBsnLookup,
{
    // Using the access_token that the user specified, lookup the user's BSN at the OIDC IdP (DigiD bridge)
    let access_token = authorization_header.token();
    let bsn: String = state
        .openid_client
        .bsn(access_token)
        .inspect_err(|error| error!("error while looking up BSN: {}", error))
        .await?;

    // Start the session, and return the initial mdoc protocol message (containing the URL at which the wallet can
    // find us) to the wallet
    let attributes = state
        .attributes_lookup
        .attributes(&bsn)
        .ok_or(Error::NoAttributesFound)?;
    let service_engagement = state.issuer.new_session(attributes).map_err(Error::StartMdoc).await?;

    Ok(Json(service_engagement))
}
