use anyhow::Result;
use tokio::net::TcpListener;
use tracing::debug;

use crate::app::create_router;
use crate::settings::Settings;

pub async fn serve(settings: Settings) -> Result<()> {
    let listener = TcpListener::bind((settings.webserver.ip, settings.webserver.port)).await?;
    debug!("listening on {}:{}", settings.webserver.ip, settings.webserver.port);

    let app = create_router(settings);

    axum::serve(listener, app).await?;

    Ok(())
}
