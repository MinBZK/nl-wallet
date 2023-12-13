use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    routing::post,
    Form, Json, Router, TypedHeader,
};
use base64::prelude::*;
use josekit::util::random_bytes;
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
    token::{TokenRequest, TokenRequestGrantType, TokenResponseWithPreviews},
};
use wallet_common::utils::sha256;

use crate::{log_requests::log_request_response, settings::Settings, verifier::Error};

mod state {
    use chrono::Utc;
    use futures::future::try_join_all;
    use nl_wallet_mdoc::{basic_sa_ext::UnsignedMdoc, server_keys::KeyRing, server_state::SessionState};
    use openid4vc::{
        credential::{CredentialRequests, CredentialResponses},
        token::{TokenRequest, TokenResponse, TokenResponseWithPreviews, TokenType},
    };
    use serde::{Deserialize, Serialize};
    use wallet_common::utils::random_string;

    use crate::{issuer::sign_attestation, verifier::Error};

    use super::{code_to_access_token, AttributeService};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Created {
        pub code: String,
        pub unsigned_mdocs: Option<Vec<UnsignedMdoc>>,
    }

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
        Created(Created),
        WaitingForResponse(WaitingForResponse),
        Done(Done),
    }

    pub trait IssuanceState {}
    impl IssuanceState for Created {}
    impl IssuanceState for WaitingForResponse {}
    impl IssuanceState for Done {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "UPPERCASE", tag = "status")]
    pub enum SessionResult {
        Done,
        Failed { error: String },
        Cancelled,
    }

    #[derive(Debug)]
    pub struct Session<S: IssuanceState> {
        pub state: SessionState<S>,
    }

    impl Session<Created> {
        pub fn into_enum(self) -> SessionState<IssuanceData> {
            SessionState {
                session_data: IssuanceData::Created(self.state.session_data),
                token: self.state.token,
                last_active: self.state.last_active,
            }
        }

        pub fn from_enum(session: SessionState<IssuanceData>) -> Result<Self, Error> {
            let session_data = match session.session_data {
                IssuanceData::Created(state) => state,
                _ => panic!("incorrect state"),
            };
            Ok(Session::<Created> {
                state: SessionState {
                    session_data,
                    token: session.token,
                    last_active: session.last_active,
                },
            })
        }

        pub async fn process_token_request(
            self,
            token_request: TokenRequest,
            attr_service: &impl AttributeService,
            code_hash_key: &[u8],
        ) -> Result<(TokenResponseWithPreviews, Session<WaitingForResponse>), (Error, Session<Done>)> {
            let code = token_request.code();
            let unsigned_mdocs = attr_service.attributes(&self.state, token_request).await.unwrap(); // TODO

            let access_token = code_to_access_token(code_hash_key, &code);
            let c_nonce = random_string(32);

            let response = TokenResponseWithPreviews {
                token_response: TokenResponse {
                    access_token: access_token.clone(),
                    c_nonce: Some(c_nonce.clone()),
                    token_type: TokenType::Bearer,
                    expires_in: None,
                    refresh_token: None,
                    scope: None,
                    c_nonce_expires_in: None,
                    authorization_details: None,
                },
                attestation_previews: unsigned_mdocs.clone(),
            };

            let next = self.transition(WaitingForResponse {
                access_token,
                c_nonce,
                unsigned_mdocs,
            });

            Ok((response, next))
        }
    }

    impl Session<WaitingForResponse> {
        pub fn into_enum(self) -> SessionState<IssuanceData> {
            SessionState {
                session_data: IssuanceData::WaitingForResponse(self.state.session_data),
                token: self.state.token,
                last_active: self.state.last_active,
            }
        }

        pub fn from_enum(session: SessionState<IssuanceData>) -> Result<Self, Error> {
            let session_data = match session.session_data {
                IssuanceData::WaitingForResponse(state) => state,
                _ => panic!("incorrect state"),
            };
            Ok(Session::<WaitingForResponse> {
                state: SessionState {
                    session_data,
                    token: session.token,
                    last_active: session.last_active,
                },
            })
        }

        pub async fn process_response(
            self,
            credential_requests: CredentialRequests,
            authorization_header: String,
            private_keys: &impl KeyRing,
        ) -> (Result<CredentialResponses, Error>, Session<Done>) {
            let session_data = self.session_data();

            // Sanity check
            if session_data.access_token != authorization_header {
                panic!("wrong access token")
            }

            let credential_responses = try_join_all(
                credential_requests
                    .credential_requests
                    .iter()
                    .zip(
                        session_data
                            .unsigned_mdocs
                            .iter()
                            .flat_map(|unsigned| std::iter::repeat(unsigned).take(unsigned.copy_count as usize)),
                    )
                    .map(|(cred_req, unsigned_mdoc)| async {
                        sign_attestation(private_keys, session_data.c_nonce.clone(), cred_req, unsigned_mdoc).await
                    }),
            )
            .await
            .unwrap();

            let next = self.transition(Done {
                session_result: SessionResult::Done,
            });

            (Ok(CredentialResponses { credential_responses }), next)
        }
    }

    impl Session<Done> {
        pub fn into_enum(self) -> SessionState<IssuanceData> {
            SessionState {
                session_data: IssuanceData::Done(self.state.session_data),
                token: self.state.token,
                last_active: self.state.last_active,
            }
        }
    }

    // Transitioning functions and helpers valid for any state
    impl<T: IssuanceState> Session<T> {
        /// Transition `self` to a new state, consuming the old state, also updating the `last_active` timestamp.
        pub fn transition<NewT: IssuanceState>(self, new_state: NewT) -> Session<NewT> {
            Session {
                state: SessionState::<NewT> {
                    session_data: new_state,
                    token: self.state.token,
                    last_active: Utc::now(),
                },
            }
        }

        pub fn session_data(&self) -> &T {
            &self.state.session_data
        }
    }
}

pub use state::*;

struct Issuer<K> {
    sessions: MemorySessionStore<IssuanceData>,
    code_hash_key: Vec<u8>,
    private_keys: K,
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
fn code_to_access_token(code_hash_key: &[u8], code: &str) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(sha256([code_hash_key, code.as_bytes()].concat().as_slice()))
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
            code_hash_key: random_bytes(32), // TODO make configurable
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
    // attribute service. NB: the access token has not been handed out yet to the wallet at this point, but we need it
    // since we store sessions by their access token.
    let access_token = code_to_access_token(&state.issuer.code_hash_key, &code);
    let session = state
        .issuer
        .sessions
        .get(&access_token.clone().into())
        .await
        .unwrap()
        .unwrap_or(SessionState::<IssuanceData>::new(
            access_token.into(),
            IssuanceData::Created(Created {
                code,
                unsigned_mdocs: None,
            }),
        ));
    let session = Session::<Created>::from_enum(session).unwrap();

    let result = session
        .process_token_request(token_request, &state.attr_service, &state.issuer.code_hash_key)
        .await;

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
    let token = authorization_header.token().to_string().into();
    let session = state.issuer.sessions.get(&token).await.unwrap().unwrap(); // TODO
    let session = Session::<WaitingForResponse>::from_enum(session).unwrap(); // TODO

    let (response, next) = session
        .process_response(
            credential_requests,
            authorization_header.token().to_string(),
            &state.issuer.private_keys,
        )
        .await;

    state.issuer.sessions.write(&next.into_enum()).await.unwrap();

    response.map(Json)
}

async fn sign_attestation(
    private_keys: &impl KeyRing,
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
        private_keys.private_key(&unsigned_mdoc.doc_type).as_ref().unwrap(),
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
