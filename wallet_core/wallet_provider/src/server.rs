use std::net::TcpListener;
use std::{error::Error, sync::Arc};

use crate::{app, app_dependencies::AppDependencies, settings::Settings};

pub async fn serve(listener: TcpListener, settings: Settings) -> Result<(), Box<dyn Error>> {
    let app = app::router(Arc::new(AppDependencies::new_from_settings(settings).await?));
    axum::Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}
