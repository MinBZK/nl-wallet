use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroU8;
use std::time::Duration;

use android_attest::attestation_extension::key_description::KeyDescription;
use attestation_types::status_claim::StatusClaim;
use base64::prelude::*;
use chrono::Utc;
use crypto::PublicKey;
use crypto::keys::EcdsaKey;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use db_test::DbSetup;
use hsm::model::mock::MockPkcs11Client;
use hsm::service::HsmError;
use itertools::Itertools;
use jwt::KeyWithKid;
use jwt::nonce::Nonce;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use platform_support::attested_key::mock::MockAppleAttestedKey;
use rstest::rstest;
use status_lists::config::StatusListConfig;
use status_lists::postgres::PostgresStatusListService;
use utils::generator::UuidV4AndTimeGenerator;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;
use wallet_account::messages::instructions::ChangePinCommit;
use wallet_account::messages::instructions::ChangePinStart;
use wallet_account::messages::instructions::CheckPin;
use wallet_account::messages::instructions::IssuanceKeySetRequest;
use wallet_account::messages::instructions::IssueWia;
use wallet_account::messages::instructions::PerformIssuance;
use wallet_account::messages::instructions::Sign;
use wallet_account::messages::registration::Registration;
use wallet_account::messages::registration::WalletCertificate;
use wallet_account::messages::registration::WalletCertificateClaims;
use wallet_account::signed::ChallengeResponse;
use wallet_provider_domain::keys::Kid;
use wallet_provider_domain::keys::KidPair;
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
use wallet_provider_persistence::wallet_user_wia;
use wallet_provider_service::account_server::CurrentCertificateSigningKey;
use wallet_provider_service::account_server::InstructionError;
use wallet_provider_service::account_server::PreviousCertificateSigningKey;
use wallet_provider_service::account_server::UserState;
use wallet_provider_service::account_server::WalletCertificateError;
use wallet_provider_service::account_server::mock;
use wallet_provider_service::account_server::mock::AttestationCa;
use wallet_provider_service::account_server::mock::AttestationType;
use wallet_provider_service::account_server::mock::MOCK_APPLE_CA;
use wallet_provider_service::account_server::mock::MOCK_GOOGLE_CA_CHAIN;
use wallet_provider_service::account_server::mock::MockAccountServer;
use wallet_provider_service::account_server::mock::MockHardwareKey;
use wallet_provider_service::flags::mock::StubWalletFlags;
use wallet_provider_service::keys::WalletCertificateSigningKey;
use wallet_provider_service::wallet_certificate;
use wallet_provider_service::wia_issuer::WIA_ATTESTATION_TYPE_IDENTIFIER;

async fn do_registration(
    account_server: &MockAccountServer,
    certificate_signing_key: &impl WalletCertificateSigningKey,
    pin_privkey: &SigningKey,
    db: Db,
    attestation_ca: AttestationCa<'_>,
    wrapping_kid: Kid,
) -> (
    WalletCertificate,
    MockHardwareKey,
    WalletCertificateClaims,
    UserState<
        Repositories,
        StubWalletFlags,
        MockPkcs11Client<HsmError>,
        SigningKey,
        PostgresStatusListService<SigningKey, StubWalletFlags>,
    >,
) {
    let wia_issuer_ca = Ca::generate_issuer_mock_ca().unwrap();
    let key_pair = wia_issuer_ca.generate_issuer_status_list_mock().unwrap();

    let db_connection = db.to_connection();
    let wia_status_list_config = StatusListConfig {
        list_size: 100.try_into().unwrap(),
        create_threshold: 10.try_into().unwrap(),
        expiry: Duration::from_secs(3600),
        refresh_threshold: Duration::from_secs(600),
        ttl: None,

        base_url: "http://example.com".parse().unwrap(), // unused
        context_path: "tsl".to_string(),
        publish_dir: std::env::temp_dir().to_path_buf().try_into().unwrap(),
        key_pair, // unused
    };

    let flags = StubWalletFlags::default();
    let status_list_service = PostgresStatusListService::try_new(
        WIA_ATTESTATION_TYPE_IDENTIFIER,
        db_connection,
        wia_status_list_config,
        flags.clone(),
    )
    .await
    .unwrap();

    let user_state = mock::user_state(
        Repositories::from(db),
        flags,
        wallet_certificate::mock::setup_hsm().await,
        wrapping_kid,
        TrustAnchors::empty(),
        status_list_service,
    );

    let challenge = account_server
        .registration_challenge(certificate_signing_key, &user_state)
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

    let (certificate, _recovery_code) = account_server
        .register(
            certificate_signing_key,
            registration_message,
            &user_state,
            &MockTimeGenerator::epoch(),
        )
        .await
        .expect("Could not process registration message at account server");

    let (_, cert_data) = certificate
        .parse_and_verify_with_sub_by_kid(&HashMap::from([(
            certificate_signing_key.kid().to_owned(),
            PublicKey::from(certificate_signing_key.verifying_key().await.unwrap()),
        )]))
        .expect("Could not parse and verify wallet certificate");

    (certificate, hw_privkey, cert_data, user_state)
}

type TestUserState = UserState<
    Repositories,
    StubWalletFlags,
    MockPkcs11Client<HsmError>,
    SigningKey,
    PostgresStatusListService<SigningKey, StubWalletFlags>,
>;

#[expect(clippy::too_many_arguments, reason = "test helper")]
async fn do_pin_change(
    account_server: &MockAccountServer,
    certificate_signing_key: &SigningKey,
    hw_privkey: &MockHardwareKey,
    wallet_id: WalletId,
    wallet_certificate: WalletCertificate,
    pin_privkey: &SigningKey,
    user_state: &TestUserState,
    start_seq: u64,
) -> (SigningKey, WalletCertificate) {
    let new_pin_privkey = SigningKey::generate();
    let new_pin_pubkey = *new_pin_privkey.verifying_key();

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<ChangePinStart>(wallet_id.clone(), start_seq, wallet_certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            user_state,
        )
        .await
        .unwrap();

    let pop_pin_pubkey = new_pin_privkey.try_sign(challenge.as_slice()).await.unwrap();

    let new_certificate_result = account_server
        .handle_change_pin_start_instruction(
            hw_privkey
                .sign_instruction(
                    ChangePinStart {
                        pin_pubkey: new_pin_pubkey.into(),
                        pop_pin_pubkey: pop_pin_pubkey.into(),
                    },
                    challenge,
                    start_seq + 1,
                    pin_privkey,
                    wallet_certificate,
                )
                .await,
            (certificate_signing_key, certificate_signing_key),
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            user_state,
        )
        .await
        .expect("ChangePinStart should succeed");

    let new_certificate = new_certificate_result
        .parse_and_verify_with_sub(&PublicKey::from(*certificate_signing_key.verifying_key()).into())
        .expect("Could not parse and verify instruction result")
        .1
        .result;

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<ChangePinCommit>(wallet_id, start_seq + 2, new_certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            user_state,
        )
        .await
        .unwrap();

    account_server
        .handle_instruction(
            hw_privkey
                .sign_instruction(
                    ChangePinCommit {},
                    challenge,
                    start_seq + 3,
                    &new_pin_privkey,
                    new_certificate.clone(),
                )
                .await,
            certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            user_state,
        )
        .await
        .expect("ChangePinCommit should succeed");

    (new_pin_privkey, new_certificate)
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
    let wrapping_kid = Kid::try_from("0").unwrap();

    let certificate_signing_key = SigningKey::generate();
    let certificate_signing_pubkey = certificate_signing_key.verifying_key();

    let account_server = mock::setup_account_server(
        certificate_signing_pubkey,
        Kid::try_from(certificate_signing_key.kid()).unwrap(),
        Default::default(),
    );
    let pin_privkey = SigningKey::generate();

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
        wrapping_kid,
    )
    .await;

    let challenge1 = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 1, certificate.clone())
                .await,
            &MockTimeGenerator::epoch(),
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
            &MockTimeGenerator::epoch(),
            &user_state,
        )
        .await
        .unwrap();

    assert_instruction_data(&user_state.repositories, &cert_data.wallet_id.into(), 2, true).await;

    assert_ne!(challenge1, challenge2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_wia_status() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_kid = Kid::try_from("0").unwrap();

    let certificate_signing_key = SigningKey::generate();
    let certificate_signing_pubkey = certificate_signing_key.verifying_key();

    let account_server = mock::setup_account_server(
        certificate_signing_pubkey,
        Kid::try_from(certificate_signing_key.kid()).unwrap(),
        Default::default(),
    );
    let pin_privkey = SigningKey::generate();

    let (certificate, hw_privkey, cert_data, user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        AttestationCa::Apple(&MOCK_APPLE_CA),
        wrapping_kid,
    )
    .await;

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<IssueWia>(cert_data.wallet_id.clone().into(), 1, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();

    let instruction = hw_privkey
        .sign_instruction(
            IssueWia {
                aud: "aud".to_string(),
                nonce: Some(Nonce::from("nonce".to_string())),
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
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .unwrap();

    // fetch all WIA IDs for this wallet directly from the database
    let tx = user_state.repositories.begin_transaction().await.unwrap();
    let wallet_user_ids = wallet_user::find_wallet_user_id_by_wallet_ids(&tx, &HashSet::from([cert_data.wallet_id]))
        .await
        .unwrap()
        .into_values()
        .collect_vec();
    let wia_ids = wallet_user_wia::find_wia_ids_for_wallet_users(&tx, wallet_user_ids)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // assert that one WIA has been stored in the database, linked to this wallet
    assert_eq!(wia_ids.len(), 1);

    assert!(matches!(
        result
            .dangerous_parse_unverified()
            .unwrap()
            .1
            .result
            .wia_disclosure
            .wia()
            .dangerous_parse_unverified()
            .unwrap()
            .1
            .client_status
            .status,
        StatusClaim::StatusList(_)
    ));
}

// Rollover the server's signing keys
fn rollover_signing_keys(
    server: &mut MockAccountServer,
    current: CurrentCertificateSigningKey,
    previous: HashMap<Kid, PreviousCertificateSigningKey>,
) {
    server.keys.current_certificate_signing_key = current;
    server.keys.previous_certificate_signing_keys = previous;
}

/// Tests that a wallet certificate issued with a previous certificate signing key can still be
/// used during a key rollover, and is rejected once the old key has expired or is removed.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_certificate_signing_key_rollover() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_kid = Kid::try_from("0").unwrap();

    let certificate_signing_key = SigningKey::generate();
    let kid = Kid::try_from(certificate_signing_key.kid()).unwrap();
    let mut account_server =
        mock::setup_account_server(certificate_signing_key.verifying_key(), kid.clone(), Default::default());

    // Register with the current certificate signing key, before rollover
    let pin_privkey = SigningKey::generate();
    let (certificate, hw_privkey, cert_data, user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        AttestationCa::Apple(&MOCK_APPLE_CA),
        wrapping_kid,
    )
    .await;

    // The new current key that the WP is rolling over to
    let new_current_key = CurrentCertificateSigningKey {
        kid: Kid::try_from("1").unwrap(),
        public_key: PublicKey::from(*SigningKey::generate().verifying_key()),
    };

    let now = Utc::now();

    // Set up old key with an expiry in one hour
    rollover_signing_keys(
        &mut account_server,
        new_current_key.clone(),
        HashMap::from([(
            kid,
            PreviousCertificateSigningKey {
                certificate_public_key: PublicKey::from(*certificate_signing_key.verifying_key()),
                exp: now + Duration::from_hours(1),
            },
        )]),
    );
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 1, certificate.clone())
                .await,
            &MockTimeGenerator::new(now),
            &user_state,
        )
        .await
        .expect("certificate with non-expired old kid should be accepted");

    // Also fully handle an instruction so that the PIN public key stored in the certificate is verified
    let instruction = hw_privkey
        .sign_instruction(CheckPin, challenge, 2, &pin_privkey, certificate.clone())
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("PIN public key hashed with the non-expired old kid's HMAC key should be accepted");

    // Use a future time to test that the old kid is now expired
    account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 3, certificate.clone())
                .await,
            &MockTimeGenerator::new(now + Duration::from_hours(2)),
            &user_state,
        )
        .await
        .expect_err("certificate with expired old kid should be rejected");

    // Remove old kid from the key map
    rollover_signing_keys(&mut account_server, new_current_key, HashMap::new());
    account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 4, certificate)
                .await,
            &MockTimeGenerator::new(now),
            &user_state,
        )
        .await
        .expect_err("certificate with unknown kid should be rejected");
}

/// Tests that a PIN public key encrypted with a previous `pin_pubkey_encryption` kid can still be
/// decrypted during a key rollover, and is rejected once the old kid is removed from settings. Then
/// also tests that a PIN change re-encrypts the PIN public key with the current kid, making it possible
/// for the wallet provider to remove the old kid from settings.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_pin_pubkey_encryption_key_rollover() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_kid = Kid::try_from("0").unwrap();

    let certificate_signing_key = SigningKey::generate();
    let kid = Kid::try_from(certificate_signing_key.kid()).unwrap();
    let mut account_server =
        mock::setup_account_server(certificate_signing_key.verifying_key(), kid, Default::default());

    // Register with the current PIN public key encryption kid ("0"), before rollover
    let pin_privkey = SigningKey::generate();
    let (certificate, hw_privkey, cert_data, user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        AttestationCa::Apple(&MOCK_APPLE_CA),
        wrapping_kid,
    )
    .await;

    // Before rollover, the stored PIN public key is encrypted with kid "0"
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 1, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(CheckPin, challenge, 2, &pin_privkey, certificate.clone())
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("PIN public key encrypted with the current kid should be accepted");

    // Roll over
    account_server.keys.pin_pubkey_encryption_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: Some(Kid::try_from("0").unwrap()),
    };

    // The stored PIN public key is still encrypted with the previous kid
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 3, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(CheckPin, challenge, 4, &pin_privkey, certificate.clone())
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("PIN public key encrypted with the previous kid should still be accepted");

    // Remove the old kid from settings entirely
    account_server.keys.pin_pubkey_encryption_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: None,
    };

    // Kid "0" is now unknown to the server, so decryption will fail
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 5, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(CheckPin, challenge, 6, &pin_privkey, certificate.clone())
        .await;
    let error = account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect_err("PIN public key encrypted with an unknown kid should be rejected");
    assert!(matches!(
        error,
        InstructionError::WalletCertificate(WalletCertificateError::UnknownKid(_))
    ));

    // Restore kid "0" as the previous kid, to test migration by pin change
    account_server.keys.pin_pubkey_encryption_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: Some(Kid::try_from("0").unwrap()),
    };

    // Changing the PIN obtains a new certificate, which is encrypted with the current kid ("1")
    let (new_pin_privkey, new_certificate) = do_pin_change(
        &account_server,
        &certificate_signing_key,
        &hw_privkey,
        cert_data.wallet_id.clone().into(),
        certificate,
        &pin_privkey,
        &user_state,
        7,
    )
    .await;

    // Remove kid "0" again, rollover is now complete
    account_server.keys.pin_pubkey_encryption_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: None,
    };

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<CheckPin>(cert_data.wallet_id.clone().into(), 11, new_certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(CheckPin, challenge, 12, &new_pin_privkey, new_certificate)
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("PIN public key migrated to the current kid should still be accepted");
}

/// Tests that a wallet attestation key wrapped with a previous `attestation_wrapping` kid can still
/// be used for signing during a key rollover, and is rejected once the old kid is removed from
/// settings. Also tests that a key issued after the rollover is wrapped with the current kid, and
/// remains usable regardless of what happens to the old kid.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_attestation_wrapping_key_rollover() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;
    let wrapping_kid = Kid::try_from("0").unwrap();

    let certificate_signing_key = SigningKey::generate();
    let kid = Kid::try_from(certificate_signing_key.kid()).unwrap();
    let account_server = mock::setup_account_server(certificate_signing_key.verifying_key(), kid, Default::default());

    // Register and issue a key wrapped with the current attestation wrapping kid ("0")
    let pin_privkey = SigningKey::generate();
    let (certificate, hw_privkey, cert_data, mut user_state) = do_registration(
        &account_server,
        &certificate_signing_key,
        &pin_privkey,
        db,
        AttestationCa::Apple(&MOCK_APPLE_CA),
        wrapping_kid,
    )
    .await;

    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<PerformIssuance>(
                    cert_data.wallet_id.clone().into(),
                    1,
                    certificate.clone(),
                )
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            PerformIssuance {
                aud: "aud".to_owned(),
                key_requests: vec_nonempty![IssuanceKeySetRequest {
                    key_count: NonZeroU8::MIN,
                    proof_nonce: None,
                }],
            },
            challenge,
            2,
            &pin_privkey,
            certificate.clone(),
        )
        .await;
    let result = account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .unwrap();
    let key_identifier = result
        .dangerous_parse_unverified()
        .unwrap()
        .1
        .result
        .keys
        .into_first()
        .into_first()
        .key_identifier;

    let sign_instruction = |key_identifier: String| Sign {
        messages_with_identifiers: vec![(b"message".to_vec(), vec![key_identifier])],
        poa_nonce: None,
        poa_aud: "aud".to_string(),
    };

    // Sign with the key wrapped under the current kid ("0")
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<Sign>(cert_data.wallet_id.clone().into(), 3, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            sign_instruction(key_identifier.clone()),
            challenge,
            4,
            &pin_privkey,
            certificate.clone(),
        )
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("signing with a key wrapped under the current kid should succeed");

    // Roll over
    user_state.attestation_wrapping_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: Some(Kid::try_from("0").unwrap()),
    };

    // Sign with the key wrapped under the previous kid ("0")
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<Sign>(cert_data.wallet_id.clone().into(), 5, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            sign_instruction(key_identifier.clone()),
            challenge,
            6,
            &pin_privkey,
            certificate.clone(),
        )
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("signing with a key wrapped under the previous kid should still succeed");

    // Issue a second key while current = "1" and previous = "0"
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<PerformIssuance>(
                    cert_data.wallet_id.clone().into(),
                    7,
                    certificate.clone(),
                )
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            PerformIssuance {
                aud: "aud".to_owned(),
                key_requests: vec_nonempty![IssuanceKeySetRequest {
                    key_count: NonZeroU8::MIN,
                    proof_nonce: None,
                }],
            },
            challenge,
            8,
            &pin_privkey,
            certificate.clone(),
        )
        .await;
    let result = account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .unwrap();
    let new_key_identifier = result
        .dangerous_parse_unverified()
        .unwrap()
        .1
        .result
        .keys
        .into_first()
        .into_first()
        .key_identifier;

    // Remove the old kid from settings entirely
    user_state.attestation_wrapping_kids = KidPair {
        current: Kid::try_from("1").unwrap(),
        previous: None,
    };

    // Kid "0" is now unknown to the server, so unwrapping the first key must fail
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<Sign>(cert_data.wallet_id.clone().into(), 9, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            sign_instruction(key_identifier),
            challenge,
            10,
            &pin_privkey,
            certificate.clone(),
        )
        .await;
    let error = account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect_err("signing with a key wrapped under an unknown kid should be rejected");
    assert!(matches!(error, InstructionError::UnknownKid(_)));

    // The second key was wrapped with the current kid ("1"), so it remains usable
    let challenge = account_server
        .instruction_challenge(
            hw_privkey
                .sign_instruction_challenge::<Sign>(cert_data.wallet_id.clone().into(), 11, certificate.clone())
                .await,
            &UuidV4AndTimeGenerator,
            &user_state,
        )
        .await
        .unwrap();
    let instruction = hw_privkey
        .sign_instruction(
            sign_instruction(new_key_identifier),
            challenge,
            12,
            &pin_privkey,
            certificate,
        )
        .await;
    account_server
        .handle_instruction(
            instruction,
            &certificate_signing_key,
            &UuidV4AndTimeGenerator,
            &TimeoutPinPolicy,
            &user_state,
        )
        .await
        .expect("signing with a key wrapped under the current kid should be unaffected");
}
