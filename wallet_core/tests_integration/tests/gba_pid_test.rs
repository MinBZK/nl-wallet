use indexmap::IndexMap;
use rstest::rstest;
use uuid::Uuid;

use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use openid4vc::issuance_session::IssuanceSessionError;
use openid4vc::ErrorResponse;
use openid4vc::TokenErrorCode;
use tests_integration::fake_digid::fake_digid_auth;
use wallet::errors::PidIssuanceError;
use wallet::mock::default_configuration;
use wallet::mock::LocalConfigurationRepository;
use wallet::mock::MockStorage;
use wallet::wallet_deps::ConfigurationRepository;
use wallet::wallet_deps::HttpAccountProviderClient;
use wallet::wallet_deps::HttpDigidSession;
use wallet::Wallet;
use wallet_common::keys::software::SoftwareEcdsaKey;

#[derive(Debug, Eq, PartialEq)]
enum TestError {
    Conversion,
    UnknownBsn,
    Unknown,
}

#[tokio::test]
#[rstest]
async fn test_gba_pid_conversion_error(
    #[values("999993318", "999993896", "999990585", "999991127", "999992326", "999997658")] bsn: &str,
) {
    assert_eq!(
        TestError::Conversion,
        gba_pid(bsn).await.expect_err("should return error")
    );
}

#[tokio::test]
#[rstest]
async fn test_gba_pid_unknown_bsn(#[values("999992306", "999995657")] bsn: &str) {
    assert_eq!(
        TestError::UnknownBsn,
        gba_pid(bsn).await.expect_err("should return error")
    );
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
    assert!(gba_pid(bsn).await.is_ok());
}

async fn gba_pid(bsn: &str) -> Result<(), TestError> {
    let config_repository = LocalConfigurationRepository::new(default_configuration());
    let pid_issuance_config = &config_repository.config().pid_issuance;

    let mut wallet: Wallet<
        LocalConfigurationRepository,
        MockStorage,
        SoftwareEcdsaKey,
        HttpAccountProviderClient,
        HttpDigidSession,
        HttpIssuanceSession,
        DisclosureSession<HttpVpMessageClient, Uuid>,
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

    let redirect_url = fake_digid_auth(&authorization_url, &pid_issuance_config.digid_http_config, bsn).await;

    let unsigned_mdocs_result = wallet.continue_pid_issuance(redirect_url).await;
    let unsigned_mdocs = match unsigned_mdocs_result {
        Ok(mdocs) => mdocs,
        Err(PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(ErrorResponse {
            error: TokenErrorCode::ServerError,
            error_description: Some(description),
            ..
        }))) if description.contains("Error converting GBA-V XML to Haal-Centraal JSON: GBA-V error") => {
            return Err(TestError::Conversion)
        }
        Err(PidIssuanceError::PidIssuer(IssuanceSessionError::TokenRequest(ErrorResponse {
            error: TokenErrorCode::ServerError,
            error_description: Some(description),
            ..
        }))) if description.contains("could not find attributes for BSN") => return Err(TestError::UnknownBsn),
        Err(e) => {
            dbg!("{:?}", e);
            return Err(TestError::Unknown);
        }
    };

    let attributes = unsigned_mdocs.into_iter().fold(IndexMap::new(), |mut attrs, mdoc| {
        mdoc.attributes.into_iter().for_each(|(key, attr)| {
            attrs.insert(format!("{}__{}", mdoc.doc_type, key), attr.value);
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

    Ok(())
}
