use std::net::SocketAddr;

use anyhow::Result;
use tracing::debug;

use crate::app::{create_router, AttributesLookup, BsnLookup};

use super::settings::Settings;

pub async fn serve<A, B>(settings: Settings, attributes_lookup: A, openid_client: B) -> Result<()>
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);

    let app = create_router(settings, attributes_lookup, openid_client).await?;
    debug!("listening on {}", socket);

    axum::Server::bind(&socket).serve(app.into_make_service()).await?;

    Ok(())
}
