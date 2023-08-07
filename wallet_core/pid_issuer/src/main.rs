use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::Result;

use pid_issuer::application::create_router;
use url::Url;

// TODO: Read from configuration and use separate client ID for mock PID issuer.
const DIGID_ISSUER_URL: &str = "https://example.com/digid-connector";
const WALLET_CLIENT_ID: &str = "SSSS";

// TODO: Read private key path and file name from config.
const SECRETS_DIR: &str = "secrets";
const JWK_PRIVATE_KEY_FILE: &str = "private_key.jwk";

// TODO: Make ip and port configurable.
const HOST_IP: [u8; 4] = [127, 0, 0, 1];
const HOST_PORT: u16 = 3003;

async fn serve() -> Result<()> {
    let issuer_url = Url::parse(DIGID_ISSUER_URL).expect("Could not parse DIGID_ISSUER_URL");
    let secrets_path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(SECRETS_DIR);
    let private_key_path = secrets_path.join(JWK_PRIVATE_KEY_FILE);

    let app = create_router(issuer_url, WALLET_CLIENT_ID, private_key_path).await?;

    let addr = SocketAddr::from((HOST_IP, HOST_PORT));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    // This will block unil the server shuts down.
    serve().await
}
