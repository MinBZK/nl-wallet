use indexmap::IndexMap;
use rstest::rstest;
use tokio::fs;
use uuid::Uuid;

use nl_wallet_mdoc::holder::{CborHttpClient, DisclosureSession};
use openid4vc::{
    issuance_session::{HttpIssuanceSession, IssuanceSessionError},
    ErrorResponse,
};
use platform_support::utils::{software::SoftwareUtilities, PlatformUtilities};
use tests_integration::fake_digid::fake_digid_auth;
use wallet::{
    errors::PidIssuanceError,
    mock::{default_configuration, MockStorage},
    wallet_deps::{
        ConfigServerConfiguration, ConfigurationRepository, HttpAccountProviderClient, HttpConfigurationRepository,
        HttpDigidSession, UpdateableConfigurationRepository,
    },
    Wallet,
};
use wallet_common::keys::software::SoftwareEcdsaKey;

#[tokio::test]
#[rstest]
#[should_panic(expected = "conversion error")]
async fn test_gba_pid_conversion_error(
    #[values("999993318", "999993896", "999990585", "999991127", "999992326", "999997658")] bsn: &str,
) {
    gba_pid(bsn).await
}

#[tokio::test]
#[rstest]
#[should_panic(expected = "unknown bsn")]
async fn test_gba_pid_unknown_bsn(#[values("999992306", "999995657")] bsn: &str) {
    gba_pid(bsn).await
}

#[tokio::test]
#[rstest]
async fn test_gba_pid_success(
    #[values(
        "999991772",
        "999991000",
        "999992958",
        "999991838",
        "999991802",
        "999991644",
        "999994761",
        "999990159",
        "999993598",
        "000009842",
        "999991747",
        "999994785",
        "999992636",
        "999993811",
        "999990640",
        "999992107",
        "999992120",
        "999991577",
        "999992533",
        "999994931",
        "999993215",
        "999992983",
        "999990044",
        "999990196",
        "999993446",
        "999992880",
        "000009878",
        "999991516",
        "999991292",
        "999991401",
        "999992569",
        "999991814",
        "999994359",
        "999994542",
        "999990160",
        "999992065",
        "999991565",
        "999991243",
        "999990871",
        "999990500",
        "999993665",
        "999990627",
        "999993409",
        "999997634",
        "999997646",
        "999997671",
        "999997683",
        "999997695",
        "999997701",
        "999997713",
        "999997725",
        "999997737",
        "999997749",
        "999997750",
        "010245741",
        "010755561",
        "999998341",
        "999998353"
    )]
    bsn: &str,
) {
    gba_pid(bsn).await
}

async fn gba_pid(bsn: &str) {
    let storage_path = SoftwareUtilities::storage_path().await.unwrap();
    let etag_file = storage_path.join("latest-configuration-etag.txt");
    // make sure there are no storage files from previous test runs
    let _ = fs::remove_file(etag_file.as_path()).await;

    let config_server_config = ConfigServerConfiguration::default();
    let wallet_config = default_configuration();

    let config_repository = HttpConfigurationRepository::new(
        config_server_config.base_url,
        config_server_config.trust_anchors,
        (&config_server_config.signing_public_key).into(),
        storage_path,
        wallet_config,
    )
    .await
    .unwrap();
    config_repository.fetch().await.unwrap();
    let pid_issuance_config = &config_repository.config().pid_issuance;

    let mut wallet: Wallet<
        HttpConfigurationRepository,
        MockStorage,
        SoftwareEcdsaKey,
        HttpAccountProviderClient,
        HttpDigidSession,
        HttpIssuanceSession,
        DisclosureSession<CborHttpClient, Uuid>,
    > = Wallet::init_registration(
        config_repository,
        MockStorage::default(),
        HttpAccountProviderClient::default(),
    )
    .await
    .expect("Could not create test wallet");

    let pin = String::from("123344");

    wallet.register(pin.clone()).await.expect("Could not register wallet");

    let authorization_url = wallet
        .create_pid_issuance_auth_url()
        .await
        .expect("Could not create pid issuance auth url");

    let redirect_url = fake_digid_auth(
        &authorization_url,
        &pid_issuance_config.digid_url,
        pid_issuance_config.digid_trust_anchors(),
        bsn,
    )
    .await;

    let unsigned_mdocs_result = wallet.continue_pid_issuance(redirect_url).await;
    let unsigned_mdocs = match unsigned_mdocs_result {
        Ok(mdocs) => mdocs,
        Err(PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(ErrorResponse {
            error_description: Some(description),
            ..
        }))) if description.contains("Error converting GBA-V XML to Haal-Centraal JSON: GBA-V error") => {
            panic!("conversion error")
        }
        Err(PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(ErrorResponse {
            error_description: Some(description),
            ..
        }))) if description.contains("could not find attributes for BSN") => panic!("unknown bsn"),
        Err(e) => {
            dbg!("{:?}", e);
            panic!("could not continue pid issuance")
        }
    };

    let attributes = unsigned_mdocs.into_iter().fold(IndexMap::new(), |mut attrs, mdoc| {
        mdoc.attributes.into_iter().for_each(|(key, attr)| {
            attrs.insert(key, attr.value);
        });
        attrs
    });

    insta::with_settings!({
        description => format!("BSN: {}", bsn),
        snapshot_suffix => bsn,
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_yaml_snapshot!(attributes);
    });
}
