use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

use utils::built_info::version_string;

use crate::app::create_routers;
use crate::settings::Settings;

pub async fn serve(settings: Settings) -> Result<()> {
    let web_listener = TcpListener::bind((settings.webserver.ip, settings.webserver.port)).await?;
    let issuance_listener = TcpListener::bind((settings.issuance_server.ip, settings.issuance_server.port))
        .await?
        .into_std()?;

    info!("{}", version_string());

    info!("listening on {}:{}", settings.webserver.ip, settings.webserver.port);
    info!(
        "listening on {}:{}",
        settings.issuance_server.ip, settings.issuance_server.port
    );

    let (web_router, issuance_router) = create_routers(settings.clone());

    info!("listening for web on {}", web_listener.local_addr()?);
    let web_server = tokio::spawn(async move {
        axum::serve(web_listener, web_router)
            .await
            .expect("web server should be started");
    });

    info!("listening for issuance on {}", issuance_listener.local_addr()?);
    let issuance_server = tokio::spawn(async move {
        if let Some(tls_config) = &settings.issuance_server_tls_config {
            axum_server::from_tcp_rustls(
                issuance_listener,
                tls_config
                    .clone()
                    .into_rustls_config()
                    .await
                    .expect("TLS config should be valid"),
            )
            .serve(issuance_router.into_make_service())
            .await
            .expect("issuance server should be started");
        } else {
            axum_server::from_tcp(issuance_listener)
                .serve(issuance_router.into_make_service())
                .await
                .expect("issuance server should be started");
        }
    });

    tokio::try_join!(web_server, issuance_server)?;

    Ok(())
}
