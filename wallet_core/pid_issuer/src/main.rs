use std::net::SocketAddr;

use anyhow::Result;

use pid_issuer::{application::create_router, settings::Settings};

async fn serve() -> Result<()> {
    let settings = Settings::new()?;
    let addr = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let app = create_router(settings.digid).await?;

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
