use std::sync::Arc;

use p256::ecdsa::signature::Verifier;
use serial_test::serial;

use wallet_common::utils::{random_bytes, random_string};
use wallet_provider::settings::Settings;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_service::hsm::{Hsm, Pkcs11Hsm, WalletUserHsm};

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn generate_key_and_sign() {
    let settings = Settings::new().unwrap();
    let hsm = Pkcs11Hsm::new(
        settings.hsm.library_path,
        settings.hsm.user_pin,
        settings.attestation_wrapping_key_identifier,
    )
    .unwrap();

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
#[serial]
#[cfg_attr(not(feature = "db_test"), ignore)]
async fn wrap_key_and_sign() {
    let settings = Settings::new().unwrap();
    let hsm = Pkcs11Hsm::new(
        settings.hsm.library_path,
        settings.hsm.user_pin,
        settings.attestation_wrapping_key_identifier,
    )
    .unwrap();

    let (public_key, wrapped) = hsm.generate_wrapped_key().await.unwrap();

    let data = Arc::new(random_bytes(32));
    let signature = WalletUserHsm::sign_wrapped(&hsm, wrapped, Arc::clone(&data))
        .await
        .unwrap();

    public_key.verify(data.as_ref(), &signature).unwrap();
}
