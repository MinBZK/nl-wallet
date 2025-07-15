use serial_test::serial;

use crypto::mock_remote::MockRemoteKeyFactory;
use hsm::service::Pkcs11Hsm;
use http_utils::urls;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuance_session::HttpVcMessageClient;
use openid4vc::issuance_session::IssuanceSession;
use openid4vc::oidc::HttpOidcClient;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use server_utils::keys::SecretKeyVariant;
use server_utils::settings::SecretKey;
use server_utils::settings::NL_WALLET_CLIENT_ID;
use tests_integration::common::*;
use tests_integration::fake_digid::fake_digid_auth;
use wallet::wallet_deps::default_wallet_config;
use wallet::wallet_deps::DigidSession;
use wallet::wallet_deps::HttpDigidSession;

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
#[serial(hsm)]
async fn test_pid_issuance_digid_bridge() {
    let (settings, _) = pid_issuer_settings();
    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let attr_service = BrpPidAttributeService::try_new(
        HttpBrpClient::new(settings.brp_server.clone()),
        &settings.digid.bsn_privkey,
        settings.digid.http_config.clone(),
        SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: (0..32).collect::<Vec<_>>().try_into().unwrap(),
            },
            None,
        )
        .unwrap(),
    )
    .unwrap();

    let port = start_pid_issuer_server(settings.clone(), hsm, attr_service).await;

    start_gba_hc_converter(gba_hc_converter_settings()).await;

    let wallet_config = default_wallet_config();

    // Prepare DigiD flow
    let (digid_session, authorization_url) = HttpDigidSession::<HttpOidcClient>::start(
        wallet_config.pid_issuance.digid.clone(),
        wallet_config.pid_issuance.digid_http_config.clone(),
        urls::issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()).into_inner(),
    )
    .await
    .unwrap();

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(
        authorization_url,
        wallet_config.pid_issuance.digid_http_config.clone(),
        "999991772",
    )
    .await;
    let token_request = digid_session.into_token_request(redirect_url).await.unwrap();

    let server_url = local_pid_base_url(port);

    // Start issuance by exchanging the authorization code for the attestation previews
    let issuance_session = HttpIssuanceSession::start_issuance(
        HttpVcMessageClient::new(NL_WALLET_CLIENT_ID.to_string(), reqwest::Client::new()),
        server_url.clone(),
        token_request,
        &wallet_config.mdoc_trust_anchors(),
    )
    .await
    .unwrap();

    let credential_with_metadata = issuance_session
        .accept_issuance(
            &wallet_config.mdoc_trust_anchors(),
            &MockRemoteKeyFactory::default(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(2, credential_with_metadata.len());
    assert_eq!(2, credential_with_metadata[0].copies.as_ref().as_slice().len());
}
