use serial_test::serial;
use tracing::debug;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use db_test::DbSetup;
use hsm::service::Pkcs11Hsm;
use http_utils::reqwest::HttpJsonClient;
use http_utils::reqwest::default_reqwest_client_builder;
use http_utils::urls;
use http_utils::urls::DEFAULT_UNIVERSAL_LINK_BASE;
use openid4vc::wallet_issuance::AuthorizationSession;
use openid4vc::wallet_issuance::IssuanceDiscovery;
use openid4vc::wallet_issuance::IssuanceSession;
use openid4vc::wallet_issuance::discovery::HttpIssuanceDiscovery;
use pid_issuer::pid::attributes::BrpPidAttributeService;
use pid_issuer::pid::brp::client::HttpBrpClient;
use pid_issuer::pid::constants::PID_ADDRESS_GROUP;
use pid_issuer::pid::constants::PID_ATTESTATION_TYPE;
use pid_issuer::pid::constants::PID_BSN;
use pid_issuer::pid::constants::PID_FAMILY_NAME;
use pid_issuer::pid::constants::PID_GIVEN_NAME;
use pid_issuer::pid::constants::PID_RESIDENT_COUNTRY;
use server_utils::keys::SecretKeyVariant;
use server_utils::settings::SecretKey;
use tests_integration::common::*;
use tests_integration::fake_digid::fake_digid_auth;
use wallet::test::default_wallet_config;

/// Test the DigiD connector + BRP proxy integration as consumed by the pid_issuer.
///
/// The test runs an in-process pid_issuer against a real nl-rdo-max and BRP proxy, drives the OIDC
/// authorization code flow with `fake_digid_auth`, exchanges the code for attestation previews, and
/// asserts on the BSN and BRP-derived fields in those previews. It stops at `start_issuance` —
/// actually accepting issuance would drag in WSCD/WUA plumbing which is covered by `test_pid_ok`
/// and friends under the `integration_test` feature (those tests mock OIDC so they do not exercise
/// the bridge covered here).
///
/// Before running this, ensure that you have nl-rdo-max and brpproxy properly configured and
/// running locally:
/// - Run `setup-devenv.sh` if not recently done,
/// - Run `start-devenv.sh digid brpproxy`, or else `docker compose up` in your nl-rdo-max checkout, and `docker compose
///   up brpproxy` in /scripts.
///
/// Run the test itself with `cargo test --package tests_integration --features=digid_test`.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn ltc1_test_pid_issuance_digid_bridge() {
    let db_setup = DbSetup::create_clean().await;
    let (settings, _) = pid_issuer_settings(db_setup.pid_issuer_url());

    let hsm = settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()
        .unwrap();

    let issuer_url = start_pid_issuer_server(
        settings.clone(),
        hsm,
        BrpPidAttributeService::try_new(
            HttpBrpClient::new(settings.brp_server.clone()),
            &settings.digid.bsn_privkey,
            settings.digid.client_id.clone(),
            settings.digid.client_settings.clone(),
            SecretKeyVariant::from_settings(
                SecretKey::Software {
                    secret_key: (0..32).collect::<Vec<_>>().try_into().unwrap(),
                },
                None,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .await;

    start_gba_hc_converter(gba_hc_converter_settings()).await;

    let wallet_config = default_wallet_config();

    // Discover the credential issuer and start authorization code flow. `client_id` and
    // `redirect_uri` are forwarded verbatim to rdo-max, which validates them against its
    // registered client — overridable here so CI can point at a deployed rdo-max whose
    // registered client differs from the local-dev defaults.
    let client_id = option_env!("DIGID_TEST_CLIENT_ID")
        .map(str::to_owned)
        .unwrap_or_else(|| wallet_config.pid_issuance.client_id.clone());
    let redirect_uri = option_env!("DIGID_TEST_REDIRECT_URI")
        .map(|raw| raw.parse().expect("DIGID_TEST_REDIRECT_URI is not a valid URL"))
        .unwrap_or_else(|| {
            urls::issuance_base_uri(&DEFAULT_UNIVERSAL_LINK_BASE.parse().unwrap())
                .as_ref()
                .clone()
        });

    let http_client = HttpJsonClient::try_new(default_reqwest_client_builder()).unwrap();
    let credential_issuer_discovery = HttpIssuanceDiscovery::new(http_client);

    let authorization_session = credential_issuer_discovery
        .start_authorization_code_flow(&issuer_url.public, client_id, redirect_uri)
        .await
        .unwrap();

    debug!(
        "authorization_url: {}",
        authorization_session.auth_url().clone().to_string()
    );
    debug!(
        "digid base url: {}",
        settings
            .digid
            .client_settings
            .oidc_identifier
            .as_base_url()
            .clone()
            .to_string()
    );

    // Do fake DigiD authentication and parse the access token out of the redirect URL
    let redirect_url = fake_digid_auth(
        authorization_session.auth_url().clone(),
        settings.digid.client_settings.oidc_identifier.as_base_url().clone(),
        "999991772",
    )
    .await;

    // Exchange the authorization code for the attestation previews. This is where the DigiD
    // connector is queried for the BSN and the BRP proxy is queried for the attributes.
    let issuance_session = authorization_session
        .start_issuance(&redirect_url, &wallet_config.issuer_trust_anchors())
        .await
        .unwrap();

    let previews = issuance_session.normalized_credential_preview();
    assert_eq!(previews.len(), 1);

    let payload = &previews[0].content.credential_payload;
    assert_eq!(payload.attestation_type, PID_ATTESTATION_TYPE);

    let attributes = payload.attributes.as_ref();

    let bsn = attributes
        .get(PID_BSN)
        .unwrap_or_else(|| panic!("preview is missing {PID_BSN} attribute"));
    assert_eq!(bsn, &Attribute::Single(AttributeValue::Text("999991772".to_string())));

    for key in [PID_GIVEN_NAME, PID_FAMILY_NAME] {
        let attr = attributes
            .get(key)
            .unwrap_or_else(|| panic!("preview is missing {key} attribute"));
        let Attribute::Single(AttributeValue::Text(value)) = attr else {
            panic!("{key} is not a text value: {attr:?}");
        };
        assert!(!value.is_empty(), "{key} is empty");
    }

    let address = attributes
        .get(PID_ADDRESS_GROUP)
        .unwrap_or_else(|| panic!("preview is missing {PID_ADDRESS_GROUP} group"));
    let Attribute::Nested(address_fields) = address else {
        panic!("{PID_ADDRESS_GROUP} is not a nested group: {address:?}");
    };
    let country = address_fields
        .get(PID_RESIDENT_COUNTRY)
        .unwrap_or_else(|| panic!("address is missing {PID_RESIDENT_COUNTRY}"));
    assert_eq!(
        country,
        &Attribute::Single(AttributeValue::Text("Nederland".to_string()))
    );
}
