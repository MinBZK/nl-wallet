use std::error::Error;
use std::net::SocketAddr;

use tracing::info;

use wallet_common::built_info::version_string;

use super::router;
use super::router_state::RouterState;
use super::settings::Settings;

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    info!("{}", version_string());
    info!("listening on {}:{}", settings.webserver.ip, settings.webserver.port);

    let tls_config = settings.tls_config.clone();
    let router_state = RouterState::new_from_settings(settings).await?;
    let app = router::router(router_state);

    if let Some(tls_config) = tls_config {
        axum_server::bind_rustls(socket, tls_config.to_rustls_config().await?)
            .serve(app.into_make_service())
            .await?;
    } else {
        axum_server::bind(socket).serve(app.into_make_service()).await?;
    }

    Ok(())
}
