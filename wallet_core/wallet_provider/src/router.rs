use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use futures::try_join;
use futures::TryFutureExt;
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::warn;

use wallet_common::account::messages::auth::Certificate;
use wallet_common::account::messages::auth::Challenge;
use wallet_common::account::messages::auth::Registration;
use wallet_common::account::messages::auth::WalletCertificate;
use wallet_common::account::messages::instructions::ChangePinCommit;
use wallet_common::account::messages::instructions::ChangePinRollback;
use wallet_common::account::messages::instructions::ChangePinStart;
use wallet_common::account::messages::instructions::CheckPin;
use wallet_common::account::messages::instructions::ConstructPoa;
use wallet_common::account::messages::instructions::ConstructPoaResult;
use wallet_common::account::messages::instructions::GenerateKey;
use wallet_common::account::messages::instructions::GenerateKeyResult;
use wallet_common::account::messages::instructions::Instruction;
use wallet_common::account::messages::instructions::InstructionAndResult;
use wallet_common::account::messages::instructions::InstructionChallengeRequest;
use wallet_common::account::messages::instructions::InstructionResultMessage;
use wallet_common::account::messages::instructions::IssueWte;
use wallet_common::account::messages::instructions::IssueWteResult;
use wallet_common::account::messages::instructions::Sign;
use wallet_common::account::messages::instructions::SignResult;
use wallet_common::account::serialization::DerVerifyingKey;
use wallet_common::account::signed::ChallengeResponse;
use wallet_common::keys::EcdsaKey;
use wallet_provider_service::account_server::GoogleCrlProvider;
use wallet_provider_service::account_server::IntegrityTokenDecoder;
use wallet_provider_service::wte_issuer::WteIssuer;

use crate::errors::WalletProviderError;
use crate::router_state::RouterState;

/// All handlers should return this result. The [`WalletProviderError`] wraps
/// a [`StatusCode`] and JSON body, all top-level errors should be convertible
/// to this type.
///
/// For any errors there are generated by `axum` before we get to the handlers
/// this custom error will not be used however. In this case `axum` will return
/// the appropriate HTTP response codes within the 4xx range and the body will
/// contain a plain text string instead. Since this amounts to programmer error
/// and this is not a public API, having error responses in that do not contain
/// the custom JSON body in those cases is acceptable. The client should still
/// be able to handle these errors appropriately.
type Result<T> = std::result::Result<T, WalletProviderError>;

pub fn router<GRC, PIC>(router_state: RouterState<GRC, PIC>) -> Router
where
    GRC: GoogleCrlProvider + Send + Sync + 'static,
    PIC: IntegrityTokenDecoder + Send + Sync + 'static,
{
    let state = Arc::new(router_state);
    Router::new()
        .merge(health_router())
        .nest(
            "/api/v1",
            Router::new()
                .route("/enroll", post(enroll))
                .route("/createwallet", post(create_wallet))
                .route("/instructions/challenge", post(instruction_challenge))
                .route(&format!("/instructions/{}", CheckPin::NAME), post(check_pin))
                .route(
                    &format!("/instructions/{}", ChangePinStart::NAME),
                    post(change_pin_start),
                )
                .route(
                    &format!("/instructions/{}", ChangePinCommit::NAME),
                    post(change_pin_commit),
                )
                .route(
                    &format!("/instructions/{}", ChangePinRollback::NAME),
                    post(change_pin_rollback),
                )
                .route(&format!("/instructions/{}", GenerateKey::NAME), post(generate_key))
                .route(&format!("/instructions/{}", Sign::NAME), post(sign))
                .route(&format!("/instructions/{}", IssueWte::NAME), post(issue_wte))
                .route(&format!("/instructions/{}", ConstructPoa::NAME), post(construct_poa))
                .layer(TraceLayer::new_for_http())
                .with_state(Arc::clone(&state)),
        )
        .nest(
            "/config",
            Router::new()
                .route("/public-keys", get(public_keys))
                .layer(TraceLayer::new_for_http())
                .with_state(Arc::clone(&state)),
        )
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async {}))
}

async fn enroll<GRC, PIC>(State(state): State<Arc<RouterState<GRC, PIC>>>) -> Result<(StatusCode, Json<Challenge>)> {
    info!("Received enroll request, creating registration challenge");

    let challenge = state
        .account_server
        .registration_challenge(&state.certificate_signing_key)
        .await
        .inspect_err(|error| warn!("generating wallet registration challenge failed: {}", error))?;

    let body = Challenge { challenge };

    info!("Replying with registration challenge");

    Ok((StatusCode::OK, body.into()))
}

async fn create_wallet<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<ChallengeResponse<Registration>>,
) -> Result<(StatusCode, Json<Certificate>)>
where
    GRC: GoogleCrlProvider,
    PIC: IntegrityTokenDecoder,
{
    info!("Received create wallet request, registering with account server");

    let cert = state
        .account_server
        .register(&state.certificate_signing_key, payload, &state.user_state)
        .await
        .inspect_err(|error| warn!("wallet registration failed: {}", error))?;

    let body = Certificate { certificate: cert };

    info!("Replying with the created wallet certificate");

    Ok((StatusCode::CREATED, body.into()))
}

async fn instruction_challenge<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<InstructionChallengeRequest>,
) -> Result<(StatusCode, Json<Challenge>)> {
    info!("Received challenge request, creating challenge");

    let challenge = state
        .account_server
        .instruction_challenge(payload, state.as_ref(), &state.user_state)
        .await
        .inspect_err(|error| warn!("generating instruction challenge failed: {}", error))?;

    let body = Challenge { challenge };

    info!("Replying with the created challenge");

    Ok((StatusCode::OK, body.into()))
}

async fn check_pin<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<CheckPin>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<()>>)> {
    info!("Received check pin request, handling the CheckPin instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling CheckPin instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

async fn change_pin_start<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<ChangePinStart>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<WalletCertificate>>)> {
    info!("Received change pin start request, handling the ChangePinStart instruction");

    let result = state
        .account_server
        .handle_change_pin_start_instruction(
            payload,
            (&state.instruction_result_signing_key, &state.certificate_signing_key),
            state.as_ref(),
            &state.pin_policy,
            &state.user_state,
        )
        .await
        .inspect_err(|error| warn!("handling ChangePinStart instruction failed: {}", error))?;

    let body = InstructionResultMessage { result };

    Ok((StatusCode::OK, body.into()))
}

async fn change_pin_commit<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<ChangePinCommit>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<()>>)> {
    info!("Received change pin commit request, handling the ChangePinCommit instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling ChangePinCommit instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

async fn change_pin_rollback<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<ChangePinRollback>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<()>>)> {
    info!("Received change pin rollback request, handling the ChangePinRollback instruction");

    let result = state
        .account_server
        .handle_change_pin_rollback_instruction(
            payload,
            &state.instruction_result_signing_key,
            state.as_ref(),
            &state.pin_policy,
            &state.user_state,
        )
        .await
        .inspect_err(|error| warn!("handling ChangePinRollback instruction failed: {}", error))?;

    let body = InstructionResultMessage { result };

    info!("Replying with the instruction result");

    Ok((StatusCode::OK, body.into()))
}

async fn generate_key<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<GenerateKey>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<GenerateKeyResult>>)> {
    info!("Received generate key request, handling the GenerateKey instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling GenerateKey instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

async fn sign<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<Sign>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<SignResult>>)> {
    info!("Received sign request, handling the SignRequest instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling SignRequest instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

async fn issue_wte<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<IssueWte>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<IssueWteResult>>)> {
    info!("Received issue WTE request, handling the IssueWte instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling IssueWte instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

async fn construct_poa<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
    Json(payload): Json<Instruction<ConstructPoa>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<ConstructPoaResult>>)> {
    info!("Received new PoA request, handling the ConstructPoa instruction");
    let body = state
        .handle_instruction(payload)
        .await
        .inspect_err(|error| warn!("handling ConstructPoa instruction failed: {}", error))?;

    Ok((StatusCode::OK, body.into()))
}

#[derive(Serialize)]
struct PublicKeys {
    certificate_public_key: DerVerifyingKey,
    instruction_result_public_key: DerVerifyingKey,
    wte_signing_key: DerVerifyingKey,
}

async fn public_keys<GRC, PIC>(
    State(state): State<Arc<RouterState<GRC, PIC>>>,
) -> Result<(StatusCode, Json<PublicKeys>)> {
    let (certificate_public_key, instruction_result_public_key, wte_signing_key) = try_join!(
        state
            .certificate_signing_key
            .verifying_key()
            .map_err(WalletProviderError::Hsm),
        state
            .instruction_result_signing_key
            .verifying_key()
            .map_err(WalletProviderError::Hsm),
        state
            .user_state
            .wte_issuer
            .public_key()
            .map_err(WalletProviderError::Wte)
    )
    .inspect_err(|error| warn!("getting wallet provider public keys failed: {}", error))?;

    let body = PublicKeys {
        certificate_public_key: certificate_public_key.into(),
        instruction_result_public_key: instruction_result_public_key.into(),
        wte_signing_key: wte_signing_key.into(),
    };

    Ok((StatusCode::OK, body.into()))
}
