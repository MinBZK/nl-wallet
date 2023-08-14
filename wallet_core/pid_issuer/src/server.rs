use std::net::SocketAddr;

use anyhow::Result;
use tracing::debug;

use crate::{
    application::{create_router, AttributesLookup},
    userinfo_client::BsnLookup,
};

use super::settings::Settings;

pub async fn serve<A, B>(settings: Settings) -> Result<()>
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let app = create_router::<A, B>(settings).await?;
    debug!("listening on {}", socket);

    axum::Server::bind(&socket).serve(app.into_make_service()).await?;

    Ok(())
}
