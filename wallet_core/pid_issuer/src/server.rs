use std::net::SocketAddr;

use anyhow::Result;
use tracing::debug;

use crate::application::create_router;

use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<()> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let app = create_router(settings).await?;
    debug!("listening on {}", socket);

    axum::Server::bind(&socket).serve(app.into_make_service()).await?;

    Ok(())
}
