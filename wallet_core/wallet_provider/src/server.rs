use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use tracing::debug;
use wallet_common::config::wallet_config::WalletConfiguration;

use super::{router, router_state::RouterState, settings::Settings};

pub async fn serve(settings: Settings, wallet_config: WalletConfiguration) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let state = Arc::new(RouterState::new_from_settings(settings).await?);

    let app = router::router(state, wallet_config);
    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}
