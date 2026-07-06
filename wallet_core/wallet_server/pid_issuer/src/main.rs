use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::tls_reqwest_client_builder;
use pid_issuer::pid::auth_code_flow::UpstreamOidcAuthorizationCodeFlow;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::pid::digid::DigidMetadataClient;
use pid_issuer::pid::digid_mock::MOCK_LOGIN_PATH;
use pid_issuer::pid::digid_mock::MockLoginState;
use pid_issuer::server;
use pid_issuer::settings::PidIssuerSettings;
use reqwest::redirect::Policy;
use server_utils::keys::SecretKeyVariant;
use server_utils::server::wallet_server_main;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: PidIssuerSettings) -> Result<()> {
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

    // Capture the DigiD trust anchors before the client settings are consumed below: the mock login
    // page (if enabled) needs its own HTTP client that trusts nl-rdo-max.
    let digid_trust_anchors = settings.digid.client_settings.trust_anchors.clone();
    let mock_subjects = settings.digid.mock_subjects;

    let digid_metadata_client = DigidMetadataClient::try_new(settings.digid.client_settings)?;
    let brp_client = HttpBrpClient::new(settings.brp_server);
    let recovery_code_secret_key = SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?;
    let digid_client_id = settings.digid.client_id;
    let bsn_privkey = settings.digid.bsn_privkey;

    let callback_base_url = settings
        .authorizing_issuer_settings
        .issuer_settings
        .public_url
        .as_base_url()
        .clone();

    // Dev/demo only: when mock subjects are configured, serve our own mock DigiD login page and point
    // the authorization flow at it instead of directly at nl-rdo-max.
    let mock_login = if mock_subjects.is_empty() {
        None
    } else {
        let trust_anchors = digid_trust_anchors
            .into_iter()
            .map(|anchor| anchor.into_certificate())
            .collect::<Vec<_>>();
        // Redirects are handled by the caller (browser); the driver reads nl-rdo-max's Location itself.
        let client = tls_reqwest_client_builder(trust_anchors)
            .redirect(Policy::none())
            .build()?;

        Some(MockLoginState::new(client, mock_subjects))
    };
    let mock_login_uri = mock_login.as_ref().map(|_| callback_base_url.join(MOCK_LOGIN_PATH));

    let (issuer, database_checkers, _, server_settings) = settings
        .authorizing_issuer_settings
        .into_authorizing_issuer(hsm, |store_connection| {
            UpstreamOidcAuthorizationCodeFlow::try_new(
                brp_client,
                &bsn_privkey,
                digid_client_id,
                digid_metadata_client,
                recovery_code_secret_key,
                store_connection,
                &callback_base_url,
            )
            .map(|flow| flow.with_mock_login_uri(mock_login_uri))
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
        mock_login,
    )
    .await
}
