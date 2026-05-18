use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::default_reqwest_client_builder;
use issuance_server::server;
use issuance_server::settings::IssuanceServerSettings;
use server_utils::server::wallet_server_main;
use server_utils::store::SessionStoreVariant;
use token_status_list::verification::reqwest::HttpStatusListClient;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("issuance_server.toml", "issuance_server", main_impl).await
}

async fn main_impl(settings: IssuanceServerSettings) -> Result<()> {
    // Note that HTTP is explicitly allowed for the retrieval of status lists.
    let status_list_client = HttpStatusListClient::new(default_reqwest_client_builder())?;
    let revocation_verifier = settings.to_revocation_verifier(status_list_client);

    let serve_status_lists = settings.issuer_settings.status_lists.serve;

    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let (issuer, database_checkers, store_connection, server_settings) = settings
        .issuer_settings
        .into_issuer(hsm.clone(), None, (), |_| (), |_| (), None)
        .await?;

    let issuer = Arc::new(issuer);

    let disclosure_sessions = SessionStoreVariant::new(store_connection, (&server_settings.storage).into());

    let disclosure_router = settings
        .verifier_settings
        .into_disclosure_router(
            hsm,
            Arc::clone(&issuer),
            disclosure_sessions,
            revocation_verifier,
            &server_settings,
        )
        .await?;

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(
        issuer,
        server_settings,
        serve_status_lists,
        health_checkers,
        disclosure_router,
    )
    .await
}
