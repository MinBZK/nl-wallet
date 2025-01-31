use std::sync::Arc;

use p256::ecdsa::signature::Verifier;
use serial_test::serial;

use hsm::model::Hsm;
use hsm::service::Pkcs11Hsm;
use wallet_common::utils::random_bytes;
use wallet_common::utils::random_string;
use wallet_provider::settings::Settings;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_service::hsm::WalletUserPkcs11Hsm;

fn setup_hsm() -> (WalletUserPkcs11Hsm, Settings) {
    let settings = Settings::new().unwrap();
    let hsm = WalletUserPkcs11Hsm::new(
        Pkcs11Hsm::from_settings(settings.hsm).unwrap(),
        settings.attestation_wrapping_key_identifier,
    );
    (hsm, Settings::new().unwrap())
}

#[tokio::test]
#[serial(hsm)]
async fn generate_key_and_sign() {
    let (hsm, _) = setup_hsm();

    let wallet_id: WalletId = String::from("wallet_user_1");
    let identifier = random_string(8);
    let public_key = hsm.generate_key(&wallet_id, &identifier).await.unwrap();

    let data = Arc::new(random_bytes(32));
    let signature = WalletUserHsm::sign(&hsm, &wallet_id, &identifier, Arc::clone(&data))
        .await
        .unwrap();
    public_key.verify(data.as_ref(), &signature).unwrap();

    Hsm::delete_key(&hsm, &format!("{wallet_id}_{identifier}"))
        .await
        .unwrap();
}

#[tokio::test]
#[serial(hsm)]
async fn wrap_key_and_sign() {
    let (hsm, _) = setup_hsm();

    let (public_key, wrapped) = hsm.generate_wrapped_key().await.unwrap();

    let data = Arc::new(random_bytes(32));
    let signature = WalletUserHsm::sign_wrapped(&hsm, wrapped, Arc::clone(&data))
        .await
        .unwrap();

    public_key.verify(data.as_ref(), &signature).unwrap();
}
