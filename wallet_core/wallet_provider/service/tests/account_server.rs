use base64::prelude::*;
use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use rstest::rstest;

use android_attest::attestation_extension::key_description::KeyDescription;
use wallet_common::account::messages::auth::Registration;
use wallet_common::account::messages::auth::WalletCertificate;
use wallet_common::account::messages::auth::WalletCertificateClaims;
use wallet_common::account::messages::instructions::CheckPin;
use wallet_common::account::signed::ChallengeResponse;
use wallet_common::apple::MockAppleAttestedKey;
use wallet_common::utils;
use wallet_provider_database_settings::Settings;
use wallet_provider_domain::model::hsm::mock::MockPkcs11Client;
use wallet_provider_domain::model::wallet_user::WalletUserQueryResult;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;
use wallet_provider_domain::EpochGenerator;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_service::account_server::mock;
use wallet_provider_service::account_server::mock::AttestationCa;
use wallet_provider_service::account_server::mock::AttestationType;
use wallet_provider_service::account_server::mock::MockAccountServer;
use wallet_provider_service::account_server::mock::MockHardwareKey;
use wallet_provider_service::account_server::mock::MOCK_APPLE_CA;
use wallet_provider_service::account_server::mock::MOCK_GOOGLE_CA_CHAIN;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::hsm::HsmError;
use wallet_provider_service::keys::WalletCertificateSigningKey;
use wallet_provider_service::wallet_certificate;
use wallet_provider_service::wte_issuer::mock::MockWteIssuer;

async fn db_from_env() -> Result<Db, PersistenceError> {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );

    let settings = Settings::new().unwrap();
    Db::new(settings.database.connection_string(), Default::default()).await
}

async fn do_registration(
    account_server: &MockAccountServer,
    certificate_signing_key: &impl WalletCertificateSigningKey,
    pin_privkey: &SigningKey,
    db: Db,
    attestation_ca: AttestationCa<'_>,
) -> (
    WalletCertificate,
    MockHardwareKey,
    WalletCertificateClaims,
    UserState<Repositories, MockPkcs11Client<HsmError>, MockWteIssuer>,
) {
    let challenge = account_server
        .registration_challenge(certificate_signing_key)
        .await
        .expect("Could not get registration challenge");

    let challenge_hash = utils::sha256(&challenge);
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

    let user_state = mock::user_state(Repositories::new(db), wallet_certificate::mock::setup_hsm().await);
    let certificate = account_server
        .register(certificate_signing_key, registration_message, &user_state)
        .await
        .expect("Could not process registration message at account server");

    let cert_data = certificate
        .parse_and_verify_with_sub(&(&certificate_signing_key.verifying_key().await.unwrap()).into())
        .expect("Could not parse and verify wallet certificate");

    (certificate, hw_privkey, cert_data, user_state)
}

async fn assert_instruction_data(
    repos: &Repositories,
    wallet_id: &str,
    expected_sequence_number: u64,
    has_challenge: bool,
) {
    let tx = repos.begin_transaction().await.unwrap();
    let user_result = repos.find_wallet_user_by_wallet_id(&tx, wallet_id).await.unwrap();
    match user_result {
        WalletUserQueryResult::Found(user_boxed) => {
            let user = *user_boxed;

            assert_eq!(expected_sequence_number, user.instruction_sequence_number);
            assert!(user.instruction_challenge.is_some() == has_challenge);
        }
        _ => panic!("User should have been found"),
    }
}

#[tokio::test]
#[rstest]
async fn test_instruction_challenge(
    #[values(AttestationType::Apple, AttestationType::Google)] attestation_type: AttestationType,
) {
    let db = db_from_env().await.expect("Could not connect to database");

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
    )
    .await;

    let challenge1 = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone(), 1, certificate.clone())
                .await,
            &EpochGenerator,
            &user_state,
        )
        .await
        .unwrap();

    assert_instruction_data(&user_state.repositories, &cert_data.wallet_id, 1, true).await;

    let challenge2 = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone(), 2, certificate)
                .await,
            &EpochGenerator,
            &user_state,
        )
        .await
        .unwrap();

    assert_instruction_data(&user_state.repositories, &cert_data.wallet_id, 2, true).await;

    assert_ne!(challenge1, challenge2);
}
