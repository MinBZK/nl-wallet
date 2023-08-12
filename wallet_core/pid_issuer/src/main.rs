use anyhow::Result;

use pid_issuer::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    // This will block unil the server shuts down.
    server::serve(settings).await?;

    Ok(())
}
