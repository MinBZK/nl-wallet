use p256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use std::env;
use uuid::Uuid;
use wallet_common::{
    account::messages::{
        auth::{Registration, WalletCertificate, WalletCertificateClaims},
        instructions::{InstructionChallengeRequest, InstructionChallengeRequestMessage},
    },
    generator::Generator,
};
use wallet_provider_domain::{
    model::wallet_user::WalletUserQueryResult,
    repository::{PersistenceError, TransactionStarter, WalletUserRepository},
};

use wallet_provider_persistence::{database::Db, repositories::Repositories};
use wallet_provider_service::account_server::{stub, AccountServer};

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

    Db::new(
        &env::var("WALLET_PROVIDER_DATABASE__HOST").unwrap_or("localhost".to_string()),
        &env::var("WALLET_PROVIDER_DATABASE__NAME").unwrap_or("wallet_provider".to_string()),
        Some(&env::var("WALLET_PROVIDER_DATABASE__USERNAME").unwrap_or("postgres".to_string())),
        Some(&env::var("WALLET_PROVIDER_DATABASE__PASSWORD").unwrap_or("postgres".to_string())),
    )
    .await
}

async fn do_registration(
    account_server: &AccountServer,
    hw_privkey: &SigningKey,
    pin_privkey: &SigningKey,
    repos: &Repositories,
) -> (WalletCertificate, WalletCertificateClaims) {
    let challenge = account_server
        .registration_challenge()
        .expect("Could not get registration challenge");

    let registration_message =
        Registration::new_signed(hw_privkey, pin_privkey, &challenge).expect("Could not sign new registration");

    let certificate = account_server
        .register(&UuidGenerator, repos, registration_message)
        .await
        .expect("Could not process registration message at account server");

    let cert_data = certificate
        .parse_and_verify(&account_server.certificate_pubkey)
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

#[cfg_attr(not(feature = "db_test"), ignore)]
#[tokio::test]
async fn test_instruction_challenge() {
    let db = db_from_env().await.expect("Could not connect to database");
    let repos = Repositories::new(db);

    let account_server = stub::account_server();
    let hw_privkey = SigningKey::random(&mut OsRng);
    let pin_privkey = SigningKey::random(&mut OsRng);

    let (certificate, cert_data) = do_registration(&account_server, &hw_privkey, &pin_privkey, &repos).await;

    let challenge1 = account_server
        .instruction_challenge(
            InstructionChallengeRequestMessage {
                certificate: certificate.clone(),
                message: InstructionChallengeRequest::new_signed(1, "wallet", &hw_privkey).unwrap(),
            },
            &repos,
        )
        .await
        .unwrap();

    assert_instruction_data(&repos, &cert_data.wallet_id, 1, true).await;

    let challenge2 = account_server
        .instruction_challenge(
            InstructionChallengeRequestMessage {
                certificate,
                message: InstructionChallengeRequest::new_signed(2, "wallet", &hw_privkey).unwrap(),
            },
            &repos,
        )
        .await
        .unwrap();

    assert_instruction_data(&repos, &cert_data.wallet_id, 2, true).await;

    assert_ne!(challenge1, challenge2);
}
