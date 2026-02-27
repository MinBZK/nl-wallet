use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::time::Duration;

use base64::prelude::*;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use rstest::rstest;

use android_attest::attestation_extension::key_description::KeyDescription;
use attestation_types::status_claim::StatusClaim;
use crypto::server_keys::generate::Ca;
use db_test::DbSetup;
use hsm::model::mock::MockPkcs11Client;
use hsm::service::HsmError;
use platform_support::attested_key::mock::MockAppleAttestedKey;
use status_lists::config::StatusListConfig;
use status_lists::postgres::PostgresStatusListService;
use wallet_account::messages::instructions::CheckPin;
use wallet_account::messages::instructions::PerformIssuance;
use wallet_account::messages::instructions::PerformIssuanceWithWua;
use wallet_account::messages::registration::Registration;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_account::signed::ChallengeResponse;
use wallet_provider_domain::EpochGenerator;
use wallet_provider_domain::generator::mock::MockGenerators;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::model::TimeoutPinPolicy;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::db_from_setup;
use wallet_provider_persistence::wallet_user;
use wallet_provider_persistence::wallet_user_wua;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::account_server::mock;
use wallet_provider_service::account_server::mock::AttestationCa;
use wallet_provider_service::account_server::mock::AttestationType;
use wallet_provider_service::account_server::mock::MOCK_APPLE_CA;
use wallet_provider_service::account_server::mock::MOCK_GOOGLE_CA_CHAIN;
use wallet_provider_service::account_server::mock::MockAccountServer;
use wallet_provider_service::account_server::mock::MockHardwareKey;
use wallet_provider_service::keys::WalletCertificateSigningKey;
use wallet_provider_service::wallet_certificate;
use wallet_provider_service::wua_issuer::WUA_ATTESTATION_TYPE_IDENTIFIER;
use wallet_provider_service::wua_issuer::mock::MockWuaIssuer;

async fn do_registration(
    account_server: &MockAccountServer,
    certificate_signing_key: &impl WalletCertificateSigningKey,
    pin_privkey: &SigningKey,
    db: Db,
    attestation_ca: AttestationCa<'_>,
    wrapping_key_identifier: &str,
) -> (
    WalletCertificate,
    MockHardwareKey,
    WalletCertificateClaims,
    UserState<Repositories, MockPkcs11Client<HsmError>, MockWuaIssuer, PostgresStatusListService<SigningKey>>,
) {
    let challenge = account_server
        .registration_challenge(certificate_signing_key)
        .await
        .expect("Could not get registration challenge");

    let challenge_hash = crypto::utils::sha256(&challenge);
    let (registration_message, hw_privkey) = match attestation_ca {
        AttestationCa::Apple(apple_mock_ca) => {
            let (attested_key, attestation_data) = MockAppleAttestedKey::new_with_attestation(
                apple_mock_ca,
                &challenge_hash,
                account_server.apple_config.environment,
                account_server.apple_config.app_identifier.clone(),
            );
            let registration_message =
                ChallengeResponse::<Registration>::new_apple(&attested_key, attestation_data, pin_privkey, challenge)
                    .await
                    .expect("Could not sign new Apple attested registration");

            (registration_message, MockHardwareKey::Apple(attested_key))
        }
        AttestationCa::Google(android_mock_ca_chain) => {
            let integrity_token = BASE64_STANDARD.encode(&challenge_hash);
            let key_description = KeyDescription::new_valid_mock(challenge_hash);
            let (attested_certificate_chain, attested_private_key) =
                android_mock_ca_chain.generate_attested_leaf_certificate(&key_description);
            let registration_message = ChallengeResponse::new_google(
                &attested_private_key,
                attested_certificate_chain.try_into().unwrap(),
                integrity_token,
                pin_privkey,
                challenge,
            )
            .await
            .expect("Could not sign new Google attested registration");

            (registration_message, MockHardwareKey::Google(attested_private_key))
        }
    };

    let wua_issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let key_pair = wua_issuer_ca.generate_status_list_mock().unwrap();

    let db_connection = db.to_connection();
    let wua_status_list_config = StatusListConfig {
        list_size: 100.try_into().unwrap(),
        create_threshold: 10.try_into().unwrap(),
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,

        base_url: "http://example.com".parse().unwrap(), // unused
        publish_dir: std::env::temp_dir().to_path_buf().try_into().unwrap(),
        key_pair, // unused
    };

    let status_list_service =
        PostgresStatusListService::try_new(db_connection, WUA_ATTESTATION_TYPE_IDENTIFIER, wua_status_list_config)
            .await
            .unwrap();

    let user_state = mock::user_state(
        Repositories::from(db),
        wallet_certificate::mock::setup_hsm().await,
        wrapping_key_identifier.to_string(),
        vec![],
        status_list_service,
    );

    let (certificate, _recovery_code) = account_server
        .register(certificate_signing_key, registration_message, &user_state)
        .await
        .expect("Could not process registration message at account server");

    let (_, cert_data) = certificate
        .parse_and_verify_with_sub(&(&certificate_signing_key.verifying_key().await.unwrap()).into())
        .expect("Could not parse and verify wallet certificate");

    (certificate, hw_privkey, cert_data, user_state)
}

async fn assert_instruction_data(
    repos: &Repositories,
    wallet_id: &WalletId,
    expected_sequence_number: u64,
    has_challenge: bool,
) {
    let tx = repos.begin_transaction().await.unwrap();
    let user_result = repos.find_wallet_user_by_wallet_id(&tx, wallet_id).await.unwrap();
    match user_result {
        QueryResult::Found(user_boxed) => {
            let user = *user_boxed;

            assert_eq!(expected_sequence_number, user.instruction_sequence_number);
            assert!(user.instruction_challenge.is_some() == has_challenge);
        }
        _ => panic!("User should have been found"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
async fn test_instruction_challenge(
    #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
) {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_key_identifier = "my-wrapping-key-identifier";

    let certificate_signing_key = SigningKey::random(&mut OsRng);
    let certificate_signing_pubkey = certificate_signing_key.verifying_key();

    let account_server = mock::setup_account_server(certificate_signing_pubkey, Default::default());
    let pin_privkey = SigningKey::random(&mut OsRng);

    let attestation_ca = match attestation_type {
        AttestationType::Apple => AttestationCa::Apple(&MOCK_APPLE_CA),
        AttestationType::Google => AttestationCa::Google(&MOCK_GOOGLE_CA_CHAIN),
    };

    let (certificate, hw_privkey, cert_data, user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        attestation_ca,
        wrapping_key_identifier,
    )
    .await;

    let challenge1 = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 1, certificate.clone())
                .await,
            &EpochGenerator,
            &user_state,
        )
        .await
        .unwrap();

    assert_instruction_data(&user_state.repositories, &cert_data.wallet_id.clone().into(), 1, true).await;

    let challenge2 = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 2, certificate)
                .await,
            &EpochGenerator,
            &user_state,
        )
        .await
        .unwrap();

    assert_instruction_data(&user_state.repositories, &cert_data.wallet_id.into(), 2, true).await;

    assert_ne!(challenge1, challenge2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_wua_status() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_key_identifier = "my-wrapping-key-identifier";

    let certificate_signing_key = SigningKey::random(&mut OsRng);
    let certificate_signing_pubkey = certificate_signing_key.verifying_key();

    let account_server = mock::setup_account_server(certificate_signing_pubkey, Default::default());
    let pin_privkey = SigningKey::random(&mut OsRng);

    let (certificate, hw_privkey, cert_data, user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        AttestationCa::Apple(&MOCK_APPLE_CA),
        wrapping_key_identifier,
    )
    .await;

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<PerformIssuanceWithWua>(
                    cert_data.wallet_id.clone().into(),
                    1,
                    certificate.clone(),
                )
                .await,
            &EpochGenerator,
            &user_state,
        )
        .await
        .unwrap();

    let instruction = hw_privkey
        .sign_instruction(
            PerformIssuanceWithWua {
                issuance_instruction: PerformIssuance {
                    key_count: NonZeroUsize::MIN,
                    aud: "aud".to_string(),
                    nonce: Some("nonce".to_string()),
                },
            },
            challenge,
            44,
            &pin_privkey,
            certificate.clone(),
        )
        .await;

    let result = account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &MockGenerators,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .unwrap();

    // fetch all WUA IDs for this wallet directly from the database
    let tx = user_state.repositories.begin_transaction().await.unwrap();
    let wallet_user_ids = wallet_user::find_wallet_user_id_by_wallet_ids(&tx, &HashSet::from([cert_data.wallet_id]))
        .await
        .unwrap()
        .into_values()
        .collect_vec();
    let wua_ids = wallet_user_wua::find_wua_ids_for_wallet_users(&tx, wallet_user_ids)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // assert that one WUA has been stored in the database, linked to this wallet
    assert!(wua_ids.len() == 1);

    assert!(matches!(
        result
            .dangerous_parse_unverified()
            .unwrap()
            .1
            .result
            .wua_disclosure
            .wua()
            .dangerous_parse_unverified()
            .unwrap()
            .1
            .status,
        StatusClaim::StatusList(_)
    ));
}
