use std::convert::Infallible;
use std::sync::Arc;

use acf_demo_issuer::flow::DemoAuthorizationCodeFlow;
use acf_demo_issuer::server;
use acf_demo_issuer::settings::AcfDemoIssuerSettings;
use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use server_utils::server::wallet_server_main;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("acf_demo_issuer.toml", "acf_demo_issuer", main_impl).await
}

async fn main_impl(settings: AcfDemoIssuerSettings) -> Result<()> {
    let serve_status_lists = settings.authorizing_issuer_settings.issuer_settings.status_lists.serve;

    let hsm = settings
        .authorizing_issuer_settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    let usecases = settings.usecases;
    let issuer_identifier = settings.authorizing_issuer_settings.issuer_settings.public_url.clone();

    // The consent page sets its own CSP, whose `form-action` must allow the wallet redirect target.
    // Build it once here, while the wallet redirect URIs are still in scope, and leak it for the
    // 'static lifetime the response middleware requires (it lives for the process, like the server).
    let csp_header: &'static str = Box::leak(
        server::build_consent_csp(&settings.authorizing_issuer_settings.wallet_redirect_uris).into_boxed_str(),
    );

    let (issuer, database_checkers, _, server_settings) = settings
        .authorizing_issuer_settings
        .into_authorizing_issuer(hsm, |store_connection| {
            Ok::<_, Infallible>(DemoAuthorizationCodeFlow::new(
                store_connection,
                issuer_identifier.as_base_url(),
                usecases,
            ))
        })
        .await?;

    let authorizing_issuer = Arc::new(issuer);

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(
        authorizing_issuer,
        server_settings,
        serve_status_lists,
        health_checkers,
        csp_header,
    )
    .await
}
