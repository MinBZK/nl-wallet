use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use tower_http::trace::TraceLayer;

use wallet_common::account::{
    messages::{
        auth::{Certificate, Challenge, Registration},
        instructions::{CheckPin, Instruction, InstructionChallengeRequest, InstructionResultMessage},
    },
    signed::SignedDouble,
};

use crate::{app_dependencies::AppDependencies, errors::WalletProviderError};

/// All handlers should return this result. The [`WalletProviderError`] wraps
/// a [`StatusCode`] and JSON body, all top-level errors should be convertable
/// to this type.
///
/// For any errors there are generated by `axum` before we get to the handlers
/// this custom error will not be used however. In this case `axum` will return
/// the appropriate HTTP response codes within the 4xx range and the body will
/// contain a plain text string instead. Since this ammounts to programmer error
/// and this is not a public API, having error responses in that do not contain
/// the custom JSON body in those cases is acceptable. The client should still
/// be able to handle these errors appropriately.
type Result<T> = std::result::Result<T, WalletProviderError>;

pub fn router(dependencies: Arc<AppDependencies>) -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/enroll", post(enroll))
            .route("/createwallet", post(create_wallet))
            .route("/instructions/challenge", post(instruction_challenge))
            .route("/instructions/check_pin", post(check_pin))
            .layer(TraceLayer::new_for_http())
            .with_state(dependencies),
    )
}

async fn enroll(State(state): State<Arc<AppDependencies>>) -> Result<(StatusCode, Json<Challenge>)> {
    let challenge = state.account_server.registration_challenge()?;
    let body = Challenge {
        challenge: challenge.into(),
    };

    Ok((StatusCode::OK, body.into()))
}

async fn create_wallet(
    State(state): State<Arc<AppDependencies>>,
    Json(payload): Json<SignedDouble<Registration>>,
) -> Result<(StatusCode, Json<Certificate>)> {
    let cert = state
        .account_server
        .register(state.as_ref(), &state.repositories, payload)
        .await?;

    let body = Certificate { certificate: cert };

    Ok((StatusCode::CREATED, body.into()))
}

async fn instruction_challenge(
    State(state): State<Arc<AppDependencies>>,
    Json(payload): Json<InstructionChallengeRequest>,
) -> Result<(StatusCode, Json<Challenge>)> {
    let challenge = state
        .account_server
        .instruction_challenge(payload, &state.repositories)
        .await?;

    let body = Challenge {
        challenge: challenge.into(),
    };

    Ok((StatusCode::OK, body.into()))
}

async fn check_pin(
    State(state): State<Arc<AppDependencies>>,
    Json(payload): Json<Instruction<CheckPin>>,
) -> Result<(StatusCode, Json<InstructionResultMessage<()>>)> {
    let result = state
        .account_server
        .handle_instruction(payload, &state.repositories, &state.pin_policy, state.as_ref())
        .await?;

    let body = InstructionResultMessage { result };

    Ok((StatusCode::OK, body.into()))
}
