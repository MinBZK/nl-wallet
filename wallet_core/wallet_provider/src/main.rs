use std::net::SocketAddr;

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{ecdsa::SigningKey, pkcs8::DecodePrivateKey};

use wallet_common::utils::random_bytes;

use crate::account_server::AccountServer;

mod account_server;
mod app;
mod settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let settings = settings::Settings::new().unwrap();

    let account_server_privkey = settings.signing_private_key;

    let account_server = AccountServer::new(
        account_server_privkey.0.clone(),
        random_bytes(32),
        "account_server".into(),
    )
    .unwrap();

    dbg!(STANDARD.encode(
        SigningKey::from_pkcs8_der(&account_server_privkey.0)
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
    ));

    let app = app::router(account_server);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
