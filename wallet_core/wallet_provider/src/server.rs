use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use tracing::debug;

use super::{app, app_dependencies::AppDependencies, settings::Settings};

pub async fn serve(settings: Settings) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::new(settings.webserver.ip, settings.webserver.port);
    let listener = TcpListener::bind(socket)?;
    debug!("listening on {}", socket);

    let dependencies = Arc::new(AppDependencies::new_from_settings(settings).await?);

    let app = app::router(dependencies);
    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}
