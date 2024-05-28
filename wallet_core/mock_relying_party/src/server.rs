use std::net::SocketAddr;

use anyhow::Result;
use tracing::debug;

use crate::{app::create_router, settings::Settings};

pub async fn serve(settings: Settings) -> Result<()> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);

    let app = create_router(settings);
    debug!("listening on {}", socket);

    axum::Server::bind(&socket).serve(app.into_make_service()).await?;

    Ok(())
}
