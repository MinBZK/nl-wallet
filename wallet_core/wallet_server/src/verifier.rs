use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::prelude::*;
use lazy_static::lazy_static;
use nutype::nutype;
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::{
    base64::{Base64, UrlSafe},
    formats::Unpadded,
    serde_as,
};
use strfmt::strfmt;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::log::{error, warn};
use url::Url;

use crate::{cbor::Cbor, settings::Settings};
use nl_wallet_mdoc::{
    holder::TrustAnchor,
    server_keys::{KeyRing, PrivateKey},
    server_state::{SessionState, SessionStore, SessionStoreError, SessionToken},
    utils::{reader_auth::ReturnUrlPrefix, serialization::cbor_serialize, x509::Certificate},
    verifier::{
        DisclosedAttributes, DisclosureData, ItemsRequests, SessionType, StatusResponse, VerificationError, Verifier,
    },
    SessionData,
};
use wallet_common::trust_anchor::OwnedTrustAnchor;

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
    SessionStatus(#[source] nl_wallet_mdoc::Error),
    #[error("retrieving disclosed attributes error: {0}")]
    DisclosedAttributes(#[source] nl_wallet_mdoc::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("{}", self);
        match self {
            Error::StartSession(nl_wallet_mdoc::Error::Verification(_)) => StatusCode::BAD_REQUEST,
            Error::StartSession(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ProcessMdoc(nl_wallet_mdoc::Error::Verification(verification_error))
            | Error::SessionStatus(nl_wallet_mdoc::Error::Verification(verification_error))
            | Error::DisclosedAttributes(nl_wallet_mdoc::Error::Verification(verification_error)) => {
                match verification_error {
                    VerificationError::UnknownSessionId(_)
                    | VerificationError::SessionStore(SessionStoreError::NotFound) => StatusCode::NOT_FOUND,
                    VerificationError::SessionStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
                    _ => StatusCode::BAD_REQUEST,
                }
            }
            Error::ProcessMdoc(_) => StatusCode::BAD_REQUEST,
            Error::SessionStatus(_) => StatusCode::BAD_REQUEST,
            Error::DisclosedAttributes(_) => StatusCode::BAD_REQUEST,
        }
        .into_response()
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
    public_url: Url,
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
                                SigningKey::from_pkcs8_der(&keypair.private_key.0)?,
                                Certificate::from(&keypair.certificate.0),
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
        internal_url: settings.internal_url,
        public_url: settings.public_url,
    });

    let wallet_router = Router::new()
        .route("/:session_id", post(session::<S>))
        .route(
            "/:session_id/status",
            get(status::<S>)
                // to be able to request the status from a browser, the cors headers should be set
                // but only on this endpoint
                .layer(CorsLayer::new().allow_methods([Method::GET]).allow_origin(Any)),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(application_state.clone());

    let requester_router = Router::new()
        .route("/", post(start::<S>))
        .route("/:session_id/disclosed_attributes", get(disclosed_attributes::<S>))
        .layer(TraceLayer::new_for_http())
        .with_state(application_state);

    Ok((wallet_router, requester_router))
}

async fn session<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_id): Path<SessionToken>,
    msg: Bytes,
) -> Result<Cbor<SessionData>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>>,
{
    let response = state
        .verifier
        .process_message(&msg, session_id)
        .await
        .map_err(Error::ProcessMdoc)?;

    Ok(Cbor(response))
}

async fn status<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_id): Path<SessionToken>,
) -> Result<Json<StatusResponse>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>> + Send + Sync + 'static,
{
    let status = state.verifier.status(&session_id).await.map_err(Error::SessionStatus)?;
    Ok(Json(status))
}

fn is_valid_return_url_template(s: &str) -> bool {
    // it should be a valid ReturnUrlPrefix when removing the template parameter
    let s = s.replace("{session_id}", "");
    let url = s.parse::<Url>(); // this makes sure no Url-invalid characters are present
    url.is_ok_and(|mut u| {
        u.set_query(None); // query is allowed in a template but not in a prefix
        u.set_fragment(None); // fragment is allowed in a template but not in a prefix
        u = u
            .join("path_segment_that_ends_with_a_slash/")
            .expect("should always result in a valid URL"); // path not ending with a '/' is allowed in a template but not in prefix
        TryInto::<ReturnUrlPrefix>::try_into(u).is_ok()
    })
}

#[nutype(
    derive(Debug, Deserialize, Serialize, FromStr),
    validate(predicate = is_valid_return_url_template),
)]
pub struct ReturnUrlTemplate(String);

#[derive(Deserialize, Serialize)]
pub struct StartDisclosureRequest {
    pub usecase: String,
    pub items_requests: ItemsRequests,
    pub session_type: SessionType,
    pub return_url_template: Option<ReturnUrlTemplate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartDisclosureResponse {
    pub session_url: Url,
    pub engagement_url: Url,
    pub disclosed_attributes_url: Url,
}

/// Adds the query parameters of the engagement URL by adding the session_type and the formatted return_url
fn format_engagement_url_params(
    mut engagement_url: Url,
    session_type: SessionType,
    return_url_tuple: Option<(ReturnUrlTemplate, SessionToken)>,
) -> Url {
    engagement_url
        .query_pairs_mut()
        .append_pair("session_type", &session_type.to_string());
    if let Some((template, session_id)) = return_url_tuple {
        let return_url = strfmt!(&template.into_inner(), session_id => session_id.to_string())
            .expect("return_template should always format");
        engagement_url.query_pairs_mut().append_pair("return_url", &return_url);
    }
    engagement_url
}

async fn start<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Json(start_request): Json<StartDisclosureRequest>,
) -> Result<Json<StartDisclosureResponse>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>>,
{
    let (session_id, engagement) = state
        .verifier
        .new_session(
            start_request.items_requests,
            start_request.session_type,
            start_request.usecase,
            start_request.return_url_template.is_some(),
        )
        .await
        .map_err(Error::StartSession)?;

    let session_url = state
        .public_url
        .join(&format!("/{session_id}/status"))
        .expect("should always be a valid URL");
    let disclosed_attributes_url = state
        .internal_url
        .join(&format!("/sessions/{session_id}/disclosed_attributes"))
        .expect("should always be a valid URL");

    // base64 produces an alphanumberic value, cbor_serialize takes a Cbor_IntMap here
    let engagement_url = UL_ENGAGEMENT
        .join(&BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&engagement).unwrap()))
        .expect("universal link should be hardcoded s.t. this will never fail");

    // add session_type and if available the return_url
    let engagement_url = format_engagement_url_params(
        engagement_url,
        start_request.session_type,
        start_request.return_url_template.map(|t| (t, session_id)),
    );

    Ok(Json(StartDisclosureResponse {
        session_url,
        engagement_url,
        disclosed_attributes_url,
    }))
}

#[serde_as]
#[derive(Deserialize)]
struct DisclosedAttributesParams {
    #[serde_as(as = "Option<Base64<UrlSafe, Unpadded>>")]
    transcript_hash: Option<Vec<u8>>,
}

async fn disclosed_attributes<S>(
    State(state): State<Arc<ApplicationState<S>>>,
    Path(session_id): Path<SessionToken>,
    Query(params): Query<DisclosedAttributesParams>,
) -> Result<Json<DisclosedAttributes>, Error>
where
    S: SessionStore<Data = SessionState<DisclosureData>>,
{
    let disclosed_attributes = state
        .verifier
        .disclosed_attributes(&session_id, params.transcript_hash)
        .await
        .map_err(Error::DisclosedAttributes)?;
    Ok(Json(disclosed_attributes))
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(
        "https://example.com",
        SessionType::CrossDevice,
        None,
        "https://example.com?session_type=cross_device"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/return/{session_id}".parse().unwrap()),
        "https://example.com?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef"
    )]
    #[case(
        "https://example.com",
        SessionType::CrossDevice,
        Some("https://example.com/return/{session_id}".parse().unwrap()),
        "https://example.com?session_type=cross_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/return/".parse().unwrap()),
        "https://example.com?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2F"
    )]
    #[case(
        "https://example.com/path/",
        SessionType::CrossDevice,
        Some("https://example.com/{session_id}/my_return_url/".parse().unwrap()),
        "https://example.com/path/?session_type=cross_device&return_url=https%3A%2F%2Fexample.com%2Fdeadbeef%2Fmy_return_url%2F"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/return/{session_id}?hello=world#hashtag".parse().unwrap()),
        "https://example.com?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Freturn%2Fdeadbeef%3Fhello%3Dworld%23hashtag"
    )]
    #[case(
        "https://example.com",
        SessionType::SameDevice,
        Some("https://example.com/{session_id}?id={session_id}#{session_id}".parse().unwrap()),
        "https://example.com?session_type=same_device&return_url=https%3A%2F%2Fexample.com%2Fdeadbeef%3Fid%3Ddeadbeef%23deadbeef"
    )]
    fn test_format_engagement_url_params(
        #[case] engagement_url: Url,
        #[case] session_type: SessionType,
        #[case] return_url_template: Option<ReturnUrlTemplate>,
        #[case] expected: Url,
    ) {
        let result = format_engagement_url_params(
            engagement_url,
            session_type,
            return_url_template.map(|u| (u, "deadbeef".to_owned().into())),
        );
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("https://example.com/{session_id}", true)]
    #[case("https://example.com/return/{session_id}", true)]
    #[case("https://example.com/return/{session_id}/url", true)]
    #[case("https://example.com/{session_id}/", true)]
    #[case("https://example.com/return/{session_id}/", true)]
    #[case("https://example.com/return/{session_id}/url/", true)]
    #[case("https://example.com/return/{session_id}?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_id}/{session_id}", true)]
    #[case("https://example.com/", true)]
    #[case("https://example.com/return", true)]
    #[case("https://example.com/return/url", true)]
    #[case("https://example.com/return/", true)]
    #[case("https://example.com/return/url/", true)]
    #[case("https://example.com/return/?hello=world&bye=mars#hashtag", true)]
    #[case("https://example.com/{session_id}/{not_session_id}", true)]
    #[case("file://etc/passwd", false)]
    #[case("file://etc/{session_id}", false)]
    #[case("https://{session_id}", false)]
    fn test_return_url_template(#[case] return_url_string: String, #[case] should_parse: bool) {
        assert_eq!(return_url_string.parse::<ReturnUrlTemplate>().is_ok(), should_parse);
        assert_eq!(is_valid_return_url_template(&return_url_string), should_parse)
    }
}
