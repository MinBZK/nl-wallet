use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use async_dropper::AsyncDrop;
use async_dropper::AsyncDropper;
use async_trait::async_trait;
use config::Config;
use config::ConfigError;
use config::File;
use hsm::service::Pkcs11Client;
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
use wallet_common::utils::random_string;

#[serde_as]
#[derive(Clone, Deserialize)]
struct Settings {
    pub(crate) hsm: settings::Hsm,
}

// Default is needed for AsyncDrop
#[derive(Default)]
struct TestCase {
    identifier: String,
    hsm: Option<Pkcs11Hsm>,
}

impl TestCase {
    fn new(prefix: &str) -> Self {
        // let (hsm, settings) = setup_hsm();
        let settings = Settings::new().unwrap();
        let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).unwrap();
        Self {
            identifier: format!("{}-{}", prefix, random_string(8)),
            hsm: Some(hsm),
        }
    }

    fn test_params(&self) -> (&Pkcs11Hsm, &str) {
        (self.hsm.as_ref().unwrap(), &self.identifier)
    }
}

#[async_trait]
impl AsyncDrop for TestCase {
    async fn async_drop(&mut self) -> () {
        let (hsm, identifier) = self.test_params();
        let _ = Hsm::delete_key(hsm, identifier).await;
    }
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn sign_sha256_hmac_using_new_secret_key() {
    let test_case = TestCase::new("sign_sha256_hmac_using_new_secret_key");
    let (hsm, identifier) = test_case.test_params();

    let data = Arc::new(random_bytes(32));

    Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();

    let signature = hsm.sign_hmac(identifier, Arc::clone(&data)).await.unwrap();

    hsm.verify_hmac(identifier, Arc::clone(&data), signature).await.unwrap();

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn sign_sha256_hmac() {
    let test_case = TestCase::new("sign_sha256_hmac");
    let (hsm, identifier) = test_case.test_params();

    let data = Arc::new(random_bytes(32));

    Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();

    let signature = hsm.sign_hmac(identifier, Arc::clone(&data)).await.unwrap();

    hsm.verify_hmac(identifier, Arc::clone(&data), signature).await.unwrap();

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn encrypt_decrypt() {
    let test_case = TestCase::new("encrypt_decrypt");
    let (hsm, identifier) = test_case.test_params();

    let data = random_bytes(32);

    hsm.generate_aes_encryption_key(identifier).await.unwrap();

    let encrypted: Encrypted<Vec<u8>> = Hsm::encrypt(hsm, identifier, data.clone()).await.unwrap();

    assert_ne!(data.clone(), encrypted.data.clone());

    let decrypted = Hsm::decrypt(hsm, identifier, encrypted).await.unwrap();

    assert_eq!(data, decrypted);

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial(hsm)]
async fn encrypt_decrypt_verifying_key() {
    let test_case = TestCase::new("encrypt_decrypt_verifying_key");
    let (hsm, identifier) = test_case.test_params();

    hsm.generate_aes_encryption_key(identifier).await.unwrap();

    let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
    let encrypted = Encrypter::encrypt(hsm, identifier, verifying_key).await.unwrap();

    let decrypted = Decrypter::decrypt(hsm, identifier, encrypted).await.unwrap();

    assert_eq!(verifying_key, decrypted);

    // Explicitly drop, to capture possible errors.
    drop(AsyncDropper::new(test_case));
}
