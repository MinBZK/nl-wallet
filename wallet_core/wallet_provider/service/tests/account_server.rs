use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use uuid::Uuid;

use wallet_common::{
    account::{
        messages::{
            auth::{Registration, WalletCertificate, WalletCertificateClaims},
            instructions::InstructionChallengeRequest,
        },
        signed::ChallengeRequest,
    },
    generator::Generator,
    keys::{software::SoftwareEcdsaKey, EcdsaKey},
};
use wallet_provider_database_settings::Settings;
use wallet_provider_domain::{
    model::{hsm::mock::MockPkcs11Client, wallet_user::WalletUserQueryResult},
    repository::{PersistenceError, TransactionStarter, WalletUserRepository},
    EpochGenerator,
};
use wallet_provider_persistence::{database::Db, repositories::Repositories};
use wallet_provider_service::{
    account_server::{mock, AccountServer},
    hsm::HsmError,
    keys::CertificateSigningKey,
};

struct UuidGenerator;
impl Generator<Uuid> for UuidGenerator {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

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
    account_server: &AccountServer,
    hsm: &MockPkcs11Client<HsmError>,
    certificate_signing_key: &impl CertificateSigningKey,
    hw_privkey: &SigningKey,
    pin_privkey: &SigningKey,
    repos: &Repositories,
) -> (WalletCertificate, WalletCertificateClaims) {
    let challenge = account_server
        .registration_challenge(certificate_signing_key)
        .await
        .expect("Could not get registration challenge");

    let registration_message = Registration::new_signed(hw_privkey, pin_privkey, challenge)
        .await
        .expect("Could not sign new registration");

    let certificate = account_server
        .register(
            certificate_signing_key,
            &UuidGenerator,
            repos,
            hsm,
            registration_message,
        )
        .await
        .expect("Could not process registration message at account server");

    let cert_data = certificate
        .parse_and_verify_with_sub(&(&certificate_signing_key.verifying_key().await.unwrap()).into())
        .expect("Could not parse and verify wallet certificate");

    (certificate, cert_data)
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
async fn test_instruction_challenge() {
    let db = db_from_env().await.expect("Could not connect to database");
    let repos = Repositories::new(db);

    let certificate_signing_key = SoftwareEcdsaKey::new_random("certificate_signing_key".to_string());
    let certificate_signing_pubkey = certificate_signing_key.verifying_key().await.unwrap();

    let (account_server, hsm) = mock::account_server_and_hsm((&certificate_signing_pubkey).into()).await;
    let hw_privkey = SigningKey::random(&mut OsRng);
    let pin_privkey = SigningKey::random(&mut OsRng);

    let (certificate, cert_data) = do_registration(
        &account_server,
        &hsm,
        &certificate_signing_key,
        &hw_privkey,
        &pin_privkey,
        &repos,
    )
    .await;

    let challenge1 = account_server
        .instruction_challenge(
            InstructionChallengeRequest {
                request: ChallengeRequest::sign(1, &hw_privkey).await.unwrap(),
                certificate: certificate.clone(),
            },
            &repos,
            &EpochGenerator,
            &hsm,
        )
        .await
        .unwrap();

    assert_instruction_data(&repos, &cert_data.wallet_id, 1, true).await;

    let challenge2 = account_server
        .instruction_challenge(
            InstructionChallengeRequest {
                request: ChallengeRequest::sign(2, &hw_privkey).await.unwrap(),
                certificate,
            },
            &repos,
            &EpochGenerator,
            &hsm,
        )
        .await
        .unwrap();

    assert_instruction_data(&repos, &cert_data.wallet_id, 2, true).await;

    assert_ne!(challenge1, challenge2);
}
