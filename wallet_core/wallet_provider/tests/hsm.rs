use std::sync::Arc;

use p256::ecdsa::signature::Verifier;
use serial_test::serial;

use hsm::model::Hsm;
use hsm::test::AsyncDropper;
use hsm::test::TestCase;
use wallet_common::utils::random_bytes;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::WalletId;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn generate_key_and_sign() {
    let test_case = TestCase::new("wallet_provider.toml", "generate_key_and_sign");
    let (hsm, identifier) = test_case.test_params();

    let wallet_id: WalletId = String::from("wallet_user_1");
    let public_key = hsm.generate_key(&wallet_id, identifier).await.unwrap();

    let data = Arc::new(random_bytes(32));
    let signature = WalletUserHsm::sign(hsm, &wallet_id, identifier, Arc::clone(&data))
        .await
        .unwrap();
    public_key.verify(data.as_ref(), &signature).unwrap();

    Hsm::delete_key(hsm, &format!("{wallet_id}_{identifier}"))
        .await
        .unwrap();

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}
