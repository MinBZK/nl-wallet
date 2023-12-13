use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::post,
    Form, Json, Router, TypedHeader,
};
use base64::prelude::*;
use chrono::Utc;
use futures::future::try_join_all;
use tower_http::trace::TraceLayer;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    server_keys::{KeyRing, PrivateKey, SingleKeyRing},
    server_state::{MemorySessionStore, SessionState, SessionStore},
    utils::serialization::cbor_serialize,
    IssuerSigned,
};
use openid4vc::{
    credential::{CredentialRequest, CredentialRequests, CredentialResponse, CredentialResponses},
    token::{TokenRequest, TokenRequestGrantType, TokenResponse, TokenResponseWithPreviews, TokenType},
};
use wallet_common::utils::random_string;

use crate::{log_requests::log_request_response, settings::Settings, verifier::Error};

mod state {
    use nl_wallet_mdoc::basic_sa_ext::UnsignedMdoc;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct WaitingForResponse {
        pub access_token: String,
        pub c_nonce: String,
        pub unsigned_mdocs: Vec<UnsignedMdoc>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Done {
        pub session_result: SessionResult,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum IssuanceData {
        WaitingForResponse(WaitingForResponse),
        Done(Done),
    }

    pub trait IssuanceState {}
    impl IssuanceState for WaitingForResponse {}
    impl IssuanceState for Done {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "UPPERCASE", tag = "status")]
    pub enum SessionResult {
        Done,
        Failed { error: String },
        Cancelled,
    }
}

use state::*;

struct Issuer<K> {
    sessions: MemorySessionStore<IssuanceData>,
    private_keys: K,
}

struct ApplicationState<A, K> {
    issuer: Issuer<K>,
    attr_service: A,
}

#[async_trait]
pub trait AttributeService: Sized + Send + Sync + 'static {
    type Error: std::fmt::Debug;
    type Settings;

    async fn new(settings: &Self::Settings) -> Result<Self, Self::Error>;
    async fn attributes(&self, token_request: TokenRequest) -> Result<Vec<UnsignedMdoc>, Self::Error>;
}

pub async fn create_issuance_router<A: AttributeService>(
    settings: Settings,
    attr_service: A,
) -> anyhow::Result<Router> {
    let key = SingleKeyRing(PrivateKey::from_der(
        &settings.issuer_key.private_key.0,
        &settings.issuer_key.certificate.0,
    )?);
    let application_state = Arc::new(ApplicationState {
        issuer: Issuer::<SingleKeyRing> {
            sessions: MemorySessionStore::new(),
            private_keys: key,
        },
        attr_service,
    });

    let issuance_router = Router::new()
        .route("/token", post(token))
        .route("/batch_credential", post(batch_credential))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(log_request_response))
        .with_state(application_state);

    Ok(issuance_router)
}

async fn batch_credential<A: AttributeService, K: KeyRing>(
    State(state): State<Arc<ApplicationState<A, K>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
    Json(credential_requests): Json<CredentialRequests>,
) -> Result<Json<CredentialResponses>, Error> {
    let token = authorization_header.token().to_string().into();
    let session = state.issuer.sessions.get(&token).await.unwrap().unwrap(); // TODO

    let session_state = match session.session_data {
        IssuanceData::WaitingForResponse(state) => state,
        IssuanceData::Done(_) => panic!("incorrect state"),
    };

    // Sanity check
    if session_state.access_token != authorization_header.token() {
        panic!("wrong access token")
    }

    let credential_responses = try_join_all(
        credential_requests
            .credential_requests
            .iter()
            .zip(
                session_state
                    .unsigned_mdocs
                    .iter()
                    .flat_map(|unsigned| std::iter::repeat(unsigned).take(unsigned.copy_count as usize)),
            )
            .map(|(cred_req, unsigned_mdoc)| async {
                sign_attestation(state.as_ref(), session_state.c_nonce.clone(), cred_req, unsigned_mdoc).await
            }),
    )
    .await
    .unwrap();

    Ok(Json(CredentialResponses { credential_responses }))
}

async fn sign_attestation<A: AttributeService, K: KeyRing>(
    state: &ApplicationState<A, K>,
    c_nonce: String,
    cred_req: &CredentialRequest,
    unsigned_mdoc: &UnsignedMdoc,
) -> Result<CredentialResponse, Error> {
    assert!(matches!(cred_req.format, openid4vc::Format::MsoMdoc));
    let pubkey = cred_req
        .proof
        .verify(c_nonce, "wallet_name".to_string(), "audience".to_string())
        .unwrap(); // TODO
    let mdoc_public_key = (&pubkey).try_into().unwrap();

    let (issuer_signed, _) = IssuerSigned::sign(
        unsigned_mdoc.clone(),
        mdoc_public_key,
        state
            .issuer
            .private_keys
            .private_key(&unsigned_mdoc.doc_type)
            .as_ref()
            .unwrap(),
    )
    .await
    .unwrap(); // TODO

    Ok(CredentialResponse {
        format: openid4vc::Format::MsoMdoc,
        credential: serde_json::to_value(
            BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&issuer_signed).unwrap()), // TODO
        )
        .unwrap(),
    })
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

    let unsigned_mdocs = state.attr_service.attributes(token_request).await.unwrap(); // TODO

    let access_token = random_string(32);
    let c_nonce = random_string(32);

    state
        .issuer
        .sessions
        .write(&SessionState {
            session_data: IssuanceData::WaitingForResponse(WaitingForResponse {
                access_token: access_token.clone(),
                c_nonce: c_nonce.clone(),
                unsigned_mdocs: unsigned_mdocs.clone(),
            }),
            token: access_token.clone().into(),
            last_active: Utc::now(),
        })
        .await
        .unwrap(); // TODO

    let response = TokenResponseWithPreviews {
        token_response: TokenResponse {
            access_token,
            c_nonce: Some(c_nonce),
            token_type: TokenType::Bearer,
            expires_in: None,
            refresh_token: None,
            scope: None,
            c_nonce_expires_in: None,
            authorization_details: None,
        },
        attestation_previews: unsigned_mdocs,
    };

    Ok(Json(response))
}
