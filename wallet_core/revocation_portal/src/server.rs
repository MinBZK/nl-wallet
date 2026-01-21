use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

use utils::built_info::version_string;

use crate::app::create_router;
use crate::revocation_client::HttpRevocationClient;
use crate::settings::Settings;

pub async fn serve(settings: Settings) -> Result<()> {
    let listener = TcpListener::bind((settings.webserver.ip, settings.webserver.port)).await?;
    info!("{}", version_string());

    info!("listening on {}:{}", settings.webserver.ip, settings.webserver.port);

    let revocation_client = HttpRevocationClient {};

    let app = create_router(&settings, revocation_client);

    axum::serve(listener, app).await?;

    Ok(())
}
