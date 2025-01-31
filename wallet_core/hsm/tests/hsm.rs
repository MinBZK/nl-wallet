use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use config::Config;
use config::ConfigError;
use config::File;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use serde::Deserialize;
use serde_with::serde_as;
use serial_test::serial;

use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Decrypter;
use hsm::model::encrypter::Encrypter;
use hsm::model::Hsm;
use hsm::service::Pkcs11Hsm;
use hsm::settings;
use wallet_common::utils::random_bytes;

#[serde_as]
#[derive(Clone, Deserialize)]
struct Settings {
    pub(crate) pin_pubkey_encryption_key_identifier: String,
    pub(crate) pin_public_disclosure_protection_key_identifier: String,
    pub(crate) hsm: settings::Hsm,
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .add_source(File::from(config_path.join("hsm.toml")).required(true))
            .build()?
            .try_deserialize()
    }
}

fn setup_hsm() -> (Pkcs11Hsm, Settings) {
    let settings = Settings::new().unwrap();
    let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).unwrap();
    (hsm, settings)
}

#[tokio::test]
#[serial(hsm)]
async fn sign_sha256_hmac_using_new_secret_key() {
    let (hsm, _) = setup_hsm();

    let secret_key = "generic_secret_key_1";
    let data = Arc::new(random_bytes(32));

    hsm.generate_generic_secret_key(secret_key).await.unwrap();

    let signature = hsm.sign_hmac(secret_key, Arc::clone(&data)).await.unwrap();

    hsm.verify_hmac(secret_key, Arc::clone(&data), signature).await.unwrap();
}

#[tokio::test]
#[serial(hsm)]
async fn sign_sha256_hmac() {
    let (hsm, settings) = setup_hsm();

    let data = Arc::new(random_bytes(32));

    let signature = hsm
        .sign_hmac(
            &settings.pin_public_disclosure_protection_key_identifier,
            Arc::clone(&data),
        )
        .await
        .unwrap();

    hsm.verify_hmac(
        &settings.pin_public_disclosure_protection_key_identifier,
        Arc::clone(&data),
        signature,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[serial(hsm)]
async fn encrypt_decrypt() {
    let (hsm, settings) = setup_hsm();

    let data = random_bytes(32);
    let encrypted: Encrypted<Vec<u8>> =
        Hsm::encrypt(&hsm, &settings.pin_pubkey_encryption_key_identifier, data.clone())
            .await
            .unwrap();

    assert_ne!(data.clone(), encrypted.data.clone());

    let decrypted = Hsm::decrypt(&hsm, &settings.pin_pubkey_encryption_key_identifier, encrypted)
        .await
        .unwrap();

    assert_eq!(data, decrypted);
}

#[tokio::test]
#[serial(hsm)]
async fn encrypt_decrypt_verifying_key() {
    let (hsm, settings) = setup_hsm();

    let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
    let encrypted = Encrypter::encrypt(&hsm, &settings.pin_pubkey_encryption_key_identifier, verifying_key)
        .await
        .unwrap();

    let decrypted = Decrypter::decrypt(&hsm, &settings.pin_pubkey_encryption_key_identifier, encrypted)
        .await
        .unwrap();

    assert_eq!(verifying_key, decrypted);
}
