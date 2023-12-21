use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
};

use tracing::debug;
use wallet_common::config::wallet_config::WalletConfiguration;

use super::{router, router_state::RouterState, settings::Settings};

pub async fn serve(settings: Settings, wallet_config: WalletConfiguration) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let (router_state, wallet_config) = RouterState::new_from_settings(settings, wallet_config).await?;

    let app = router::router(router_state, wallet_config);
    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}
