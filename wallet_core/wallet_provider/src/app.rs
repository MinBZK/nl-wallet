use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use tower_http::trace::TraceLayer;

use wallet_common::account::{
    auth::{Certificate, Challenge, Registration},
    signed::SignedDouble,
};

use crate::account_server::AccountServer;

struct AppState {
    account_server: AccountServer,
}

pub fn router(account_server: AccountServer) -> Router {
    let shared_state = Arc::new(AppState { account_server });

    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/enroll", post(enroll))
            .route("/createwallet", post(create_wallet))
            .layer(TraceLayer::new_for_http())
            .with_state(shared_state),
    )
}

async fn enroll(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Challenge>) {
    let challenge = state.account_server.registration_challenge().unwrap(); // todo: error handling
    (
        StatusCode::OK,
        Json(Challenge {
            challenge: challenge.into(),
        }),
    )
}

async fn create_wallet(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignedDouble<Registration>>,
) -> (StatusCode, Json<Certificate>) {
    let cert = state.account_server.register(payload).unwrap(); // todo: error handling
    (StatusCode::CREATED, Json(Certificate { certificate: cert }))
}

#[cfg(test)]
mod tests {
    use axum_test_helper::TestClient;
    use p256::ecdsa::SigningKey;
    use rand::rngs::OsRng;

    use crate::account_server::stub::account_server;

    use super::*;

    #[tokio::test]
    async fn test_http_registration() {
        // Create an account server stub and copy some details
        // that are needed for testing later.
        let account_server = account_server();
        let pubkey = account_server.pubkey.clone();
        let server_name = account_server.name.clone();

        // Create a new test client for the HTTP server.
        let app = router(account_server);
        let client = TestClient::new(app);

        // Get a challenge from the server.
        let response = client.post("/api/v1/enroll").send().await;

        assert!(response.status().is_success());

        let challenge = response.json::<Challenge>().await;
        let challenge_bytes = challenge.challenge.0;

        assert!(!challenge_bytes.is_empty());

        // Use the challenge to create a new registration message,
        // but first create some private keys for this.
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);
        let registration =
            Registration::new_signed(&hw_privkey, &pin_privkey, &challenge_bytes).expect("Could not sign registration");

        let response = client.post("/api/v1/createwallet").json(&registration).send().await;

        assert!(response.status().is_success());

        // Validate the contents of the received certificate.
        let certificate = response.json::<Certificate>().await;
        let certificate_data = certificate
            .certificate
            .parse_and_verify(&pubkey)
            .expect("Could not parse and verify wallet certificate");

        assert!(!certificate_data.wallet_id.is_empty());
        assert_eq!(certificate_data.hw_pubkey.0, *hw_privkey.verifying_key());
        assert!(!certificate_data.pin_pubkey_hash.0.is_empty());
        assert_eq!(certificate_data.iss, server_name);
    }

    // TODO: add tests for non-200 responses, once implemented
}
