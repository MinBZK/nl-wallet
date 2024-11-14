use std::error::Error;

use tokio::net::TcpListener;
use tracing::debug;

use super::router;
use super::router_state::RouterState;
use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind((settings.webserver.ip, settings.webserver.port)).await?;
    debug!("listening on {}:{}", settings.webserver.ip, settings.webserver.port);

    let router_state = RouterState::new_from_settings(settings).await?;

    let app = router::router(router_state);

    axum::serve(listener, app).await?;

    Ok(())
}
