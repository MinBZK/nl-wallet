use std::error::Error;
use std::net::SocketAddr;
use std::net::TcpListener;

use tracing::info;

use hsm::service::Pkcs11Hsm;
use utils::built_info::version_string;
use wallet_provider_service::account_server::GoogleCrlProvider;
use wallet_provider_service::account_server::IntegrityTokenDecoder;

use super::router;
use super::router_state::RouterState;
use super::settings::Settings;

pub async fn serve<GRC, PIC>(
    settings: Settings,
    hsm: Pkcs11Hsm,
    google_crl_client: GRC,
    play_integrity_client: PIC,
) -> Result<(), Box<dyn Error>>
where
    GRC: GoogleCrlProvider + Send + Sync + 'static,
    PIC: IntegrityTokenDecoder + Send + Sync + 'static,
{
    let listener = TcpListener::bind(SocketAddr::new(settings.webserver.ip, settings.webserver.port))?;
    serve_with_listener(listener, settings, hsm, google_crl_client, play_integrity_client).await
}

pub async fn serve_with_listener<GRC, PIC>(
    listener: TcpListener,
    settings: Settings,
    hsm: Pkcs11Hsm,
    google_crl_client: GRC,
    play_integrity_client: PIC,
) -> Result<(), Box<dyn Error>>
where
    GRC: GoogleCrlProvider + Send + Sync + 'static,
    PIC: IntegrityTokenDecoder + Send + Sync + 'static,
{
    info!("{}", version_string());
    let addr = listener.local_addr()?;
    info!("listening on {}:{}", addr.ip(), addr.port());
    listener.set_nonblocking(true)?;

    let tls_config = settings.tls_config.clone();
    let router_state = RouterState::new_from_settings(settings, hsm, google_crl_client, play_integrity_client).await?;
    let app = router::router(router_state);

    if let Some(tls_config) = tls_config {
        axum_server::from_tcp_rustls(listener, tls_config.into_rustls_config().await?)
            .serve(app.into_make_service())
            .await?;
    } else {
        axum_server::from_tcp(listener).serve(app.into_make_service()).await?;
    }

    Ok(())
}
