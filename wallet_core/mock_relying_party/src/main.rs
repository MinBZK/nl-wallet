use anyhow::Result;

use mock_relying_party::{server, settings::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    // This will block until the server shuts down.
    server::serve(settings).await?;

    Ok(())
}
