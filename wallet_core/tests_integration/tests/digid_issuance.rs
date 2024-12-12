use openid4vc::credential::MdocCopies;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuance_session::HttpVcMessageClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::oidc::HttpOidcClient;

use nl_wallet_mdoc::holder::TrustAnchor;
use tests_integration::common::*;
use tests_integration::fake_digid::fake_digid_auth;
use wallet::mock::default_configuration;
use wallet::wallet_common::WalletConfiguration;
use wallet::wallet_deps::DigidSession;
use wallet::wallet_deps::HttpDigidSession;
use wallet_common::keys::mock_remote::MockRemoteKeyFactory;
use wallet_common::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use wallet_common::urls::{self};
use wallet_server::pid::attributes::BrpPidAttributeService;
use wallet_server::pid::brp::client::HttpBrpClient;

/// Test the full PID issuance flow, i.e. including OIDC with nl-rdo-max and retrieving the PID from BRP
/// (Haal-Centraal). This test depends on part of the internal API of the DigiD bridge, so it may break when nl-rdo-max
/// is updated.
///
/// Before running this, ensure that you have nl-rdo-max and brpproxy properly configured and running locally:
/// - Run `setup-devenv.sh` if not recently done,
/// - Run `start-devenv.sh digid brpproxy`, or else `docker compose up` in your nl-rdo-max checkout, and `docker compose
///   up brpproxy` in /scripts.
///
/// Run the test itself with `cargo test --package tests_integration --features=digid_test`
///
/// See also
/// - `test_pid_ok()`, which uses the WP but mocks the OIDC part,
/// - `accept_issuance()` in the `openid4vc` integration tests, which also mocks the HTTP server and client.
#[tokio::test]
async fn test_pid_issuance_digid_bridge() {
    let settings = wallet_server_settings();
    let attr_service = BrpPidAttributeService::new(
        HttpBrpClient::new(settings.issuer.brp_server.clone()),
        &settings.issuer.digid.bsn_privkey,
        settings.issuer.digid.http_config.clone(),
        settings.issuer.certificates(),
    )
    .unwrap();
    start_wallet_server(settings.clone(), attr_service).await;

    start_gba_hc_converter(gba_hc_converter_settings()).await;

    let wallet_config = default_configuration();

    // Prepare DigiD flow
    let (digid_session, authorization_url) = HttpDigidSession::<HttpOidcClient>::start(
        wallet_config.pid_issuance.digid.clone(),
        &wallet_config.pid_issuance.digid_http_config,
        urls::issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner(),
    )
    .await
    .unwrap();

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(
        &authorization_url,
        &wallet_config.pid_issuance.digid_http_config,
        "999991772",
    )
    .await;
    let token_request = digid_session.into_token_request(redirect_url).await.unwrap();

    let server_url = local_pid_base_url(&settings.urls.public_url.as_ref().port().unwrap());

    // Start issuance by exchanging the authorization code for the attestation previews
    let (pid_issuer_client, _) = HttpIssuanceSession::start_issuance(
        HttpVcMessageClient::from(reqwest::Client::new()),
        server_url.clone(),
        token_request,
        &wallet_config.mdoc_trust_anchors(),
    )
    .await
    .unwrap();

    let mdocs = pid_issuer_client
        .accept_issuance(
            &trust_anchors(&default_configuration()),
            MockRemoteKeyFactory::default(),
            None,
            server_url,
        )
        .await
        .unwrap();

    assert_eq!(2, mdocs.len());
    assert_eq!(2, <&MdocCopies>::try_from(&mdocs[0]).unwrap().len());
}

fn trust_anchors(wallet_conf: &WalletConfiguration) -> Vec<TrustAnchor<'_>> {
    wallet_conf.mdoc_trust_anchors.iter().map(|a| a.into()).collect()
}
