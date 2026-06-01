use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use pacf_issuance_server::server;
use pacf_issuance_server::settings::PacfIssuanceServerSettings;
use server_utils::server::wallet_server_main;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pacf_issuance_server.toml", "pacf_issuance_server", main_impl).await
}

async fn main_impl(settings: PacfIssuanceServerSettings) -> Result<()> {
    let serve_status_lists = settings.issuer_settings.status_lists.serve;

    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let (issuer, database_checkers, _, server_settings) = settings.issuer_settings.into_issuer(hsm, None).await?;

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    server::serve(Arc::new(issuer), server_settings, serve_status_lists, health_checkers).await
}
