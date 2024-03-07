use openid4vc::{
    issuance_session::{HttpIssuanceSession, HttpOpenidMessageClient, IssuanceSession},
    oidc::{HttpOidcClient, OidcClient},
    pkce::S256PkcePair,
};

use nl_wallet_mdoc::{holder::TrustAnchor, software_key_factory::SoftwareKeyFactory};
use wallet::{
    mock::{default_configuration, default_reqwest_client_builder},
    wallet_common::WalletConfiguration,
};
use wallet_common::config::wallet_config::DEFAULT_UNIVERSAL_LINK_BASE;
use wallet_server::pid::attributes::MockPidAttributeService;

use crate::common::*;

pub mod common;

/// Test the full PID issuance flow, i.e. including OIDC with nl-rdo-max.
/// This test depends on part of the internal API of the DigiD bridge, so it may break when nl-rdo-max is updated.
///
/// Before running this, ensure that you have nl-rdo-max properly configured and running locally:
/// - Run `setup-devenv.sh` if not recently done,
/// - Run `start-devenv.sh digid`, or else `docker compose up` in your nl-rdo-max checkout.
///
/// Run the test itself with `cargo test --package tests_integration --features=digid_test`.
///
/// See also
/// - `test_pid_ok()`, which uses the WP but mocks the OIDC part,
/// - `accept_issuance()` in the `openid4vc` integration tests, which also mocks the HTTP server and client.
#[tokio::test]
#[cfg_attr(not(feature = "digid_test"), ignore)]
async fn test_pid_issuance_digid_bridge() {
    let settings = common::wallet_server_settings();
    let attr_service = MockPidAttributeService::new(
        settings.issuer.digid.issuer_url.clone(),
        settings.issuer.digid.bsn_privkey.clone(),
        settings.issuer.mock_data.clone(),
        settings.issuer.certificates(),
    )
    .unwrap();
    start_wallet_server(settings.clone(), attr_service).await;

    let wallet_config = default_configuration();

    // Prepare DigiD flow
    let (digid_session, authorization_url) = HttpOidcClient::<S256PkcePair>::start(
        default_reqwest_client_builder().build().unwrap(),
        settings.issuer.digid.issuer_url.clone(),
        wallet_config.pid_issuance.digid_client_id.clone(),
        WalletConfiguration::issuance_redirect_uri(DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap()),
    )
    .await
    .unwrap();

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(&authorization_url, &wallet_config.pid_issuance.digid_url).await;
    let token_request = digid_session.into_token_request(&redirect_url).unwrap();

    let server_url = local_pid_base_url(&settings.public_url.port().unwrap());

    // Start issuance by exchanging the authorization code for the attestation previews
    let (pid_issuer_client, _) = HttpIssuanceSession::start_issuance(
        HttpOpenidMessageClient::from(reqwest::Client::new()),
        server_url.clone(),
        token_request,
        &wallet_config.mdoc_trust_anchors(),
    )
    .await
    .unwrap();

    let mdocs = pid_issuer_client
        .accept_issuance(
            &trust_anchors(&default_configuration()),
            SoftwareKeyFactory::default(),
            server_url,
        )
        .await
        .unwrap();

    assert_eq!(2, mdocs.len());
    assert_eq!(2, mdocs[0].cred_copies.len())
}

fn trust_anchors(wallet_conf: &WalletConfiguration) -> Vec<TrustAnchor<'_>> {
    wallet_conf
        .mdoc_trust_anchors
        .iter()
        .map(|a| (&a.owned_trust_anchor).into())
        .collect()
}
