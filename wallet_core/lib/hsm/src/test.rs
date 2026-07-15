use std::collections::VecDeque;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use async_dropper::AsyncDrop;
use async_dropper::AsyncDropper;
use async_trait::async_trait;
use config::Config;
use config::ConfigError;
use crypto::utils::random_bytes;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Verifier;
use rand_core::OsRng;
use regex::Regex;
use serde::Deserialize;
use serde_with::serde_as;
use tempfile::TempDir;
use utils::path::prefix_local_path;

use crate::model::Hsm;
use crate::model::encrypted::Encrypted;
use crate::model::encrypter::Decrypter;
use crate::model::encrypter::Encrypter;
use crate::model::mock::MockPkcs11Client;
use crate::service::HsmError;
use crate::service::Pkcs11Client;
use crate::service::Pkcs11Hsm;
use crate::settings;

static HSM_SETUP: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct HsmSetup {
    _temp_dir: Option<TempDir>,
}

impl HsmSetup {
    pub fn new() -> HsmSetup {
        // Check for nextest, as this setup does not work with normal cargo test. The reason is that
        // nextest has a process per test setup instead of single process for each test binary used
        // by cargo test. Tests using the HSM should have a `#[serial(hsm)]` macro to ensure serial
        // execution when running via cargo test.
        match std::env::var("NEXTEST") {
            Ok(val) if &val == "1" => {}
            _ => return HsmSetup { _temp_dir: None },
        }

        // Should only run once
        if HSM_SETUP.swap(true, std::sync::atomic::Ordering::SeqCst) {
            panic!("HSM setup should only be ran once")
        }

        // Read config
        let home_dir = std::env::home_dir().expect("no home directory");
        let mut config = String::with_capacity(1024);
        std::fs::File::open(home_dir.join(".config/softhsm2/softhsm2.conf"))
            .expect("could not open softhsm2 config file")
            .read_to_string(&mut config)
            .expect("could not read softhsm2 config file");

        // Create config dir and token dir
        let temp_dir = TempDir::new().expect("failed to create temporary directory");
        let token_dir = temp_dir.path().join("tokens");

        // Get current token dir
        let caps = Regex::new(r#"(?m)^(directories\.tokendir) *= *(.+)$"#)
            .expect("could not compile regex")
            .captures(config.as_str())
            .expect("could not find token dir pattern");

        // Replace token dir in our own config
        let mut temp_config = String::with_capacity(config.len());
        temp_config.push_str(&config[..caps.get_match().start()]);
        temp_config.push_str(&caps[1]);
        temp_config.push_str(" = ");
        temp_config.push_str(token_dir.to_str().expect("unicode path error"));
        temp_config.push_str(&config[caps.get_match().end()..]);

        // Copy source token dir to destination
        let source_dir = caps[2].parse().expect("unicode path error");
        copy_dir(source_dir, token_dir).expect("failed to copy tokens directory");

        // Create config file
        let config_file = temp_dir.path().join("softhsm2.conf");
        std::fs::File::create(&config_file)
            .expect("could not create config file")
            .write_all(temp_config.as_bytes())
            .expect("could not write config file");

        // Set env var
        let env_value = config_file.to_str().expect("unicode path error");
        unsafe { std::env::set_var("SOFTHSM2_CONF", env_value) };

        HsmSetup {
            _temp_dir: Some(temp_dir),
        }
    }

    pub fn pkcs11_hsm(&self, settings: settings::Hsm) -> Result<Pkcs11Hsm, HsmError> {
        Pkcs11Hsm::from_settings(settings)
    }
}

fn copy_dir(src: PathBuf, dst: PathBuf) -> std::io::Result<()> {
    let mut queue = VecDeque::from([(src, dst)]);
    while let Some((src, dst)) = queue.pop_front() {
        std::fs::create_dir(dst.as_path())?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if entry.file_type()?.is_dir() {
                queue.push_back((src_path, dst_path));
            } else {
                std::fs::copy(src_path, dst_path)?;
            }
        }
    }
    Ok(())
}

#[serde_as]
#[derive(Clone, Deserialize)]
struct TestSettings {
    pub(crate) hsm: settings::Hsm,
}

impl TestSettings {
    fn new(config_file: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::from(prefix_local_path(config_file).as_ref()).required(true))
            .build()?
            .try_deserialize()
    }
}

pub struct TestCase<H> {
    identifier: String,
    hsm: Option<H>,
}

// Default is needed for AsyncDrop
impl<H> Default for TestCase<H> {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            hsm: None,
        }
    }
}

impl<H> TestCase<H> {
    pub fn test_params(&self) -> (&H, &str) {
        (self.hsm.as_ref().unwrap(), &self.identifier)
    }
}

impl TestCase<Pkcs11Hsm> {
    pub fn drop(self) {
        drop(AsyncDropper::new(self));
    }
}

#[async_trait]
impl AsyncDrop for TestCase<Pkcs11Hsm> {
    async fn async_drop(&mut self) -> () {
        let (hsm, identifier) = self.test_params();
        let _ = Hsm::delete_key(hsm, identifier).await;
    }
}

impl TestCase<MockPkcs11Client<HsmError>> {
    pub fn mock(identifier_prefix: &str) -> Self {
        Self {
            identifier: identifier_prefix.to_string(),
            hsm: Some(MockPkcs11Client::default()),
        }
    }
}

impl TestCase<Pkcs11Hsm> {
    pub fn new(hsm_setup: &HsmSetup, config_file: &str, identifier_prefix: &str) -> Self {
        let settings = TestSettings::new(config_file.as_ref()).unwrap();
        let hsm = hsm_setup.pkcs11_hsm(settings.hsm.clone()).unwrap();
        Self {
            identifier: format!("{}-{}", identifier_prefix, crypto::utils::random_string(8)),
            hsm: Some(hsm),
        }
    }
}

// These methods are to be called by integration tests.
impl<H> TestCase<H> {
    pub async fn sign_sha256_hmac(self: TestCase<H>) -> TestCase<H>
    where
        H: Hsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = random_bytes(32);

        Hsm::generate_generic_secret_key(hsm, identifier).await.unwrap();
        let signature = hsm.sign_hmac(identifier, &data).await.unwrap();
        hsm.verify_hmac(identifier, &data, signature).await.unwrap();

        self
    }

    pub async fn sign_ecdsa(self: TestCase<H>) -> TestCase<H>
    where
        H: Hsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = Arc::new(random_bytes(32));

        Hsm::generate_signing_key_pair(hsm, identifier).await.unwrap();

        let signature = hsm.sign_ecdsa(identifier, &data).await.unwrap();
        let verifying_key = Hsm::get_verifying_key(hsm, identifier).await.unwrap();
        verifying_key.verify(&data, &signature).unwrap();

        self
    }

    pub async fn encrypt_decrypt(self: TestCase<H>) -> TestCase<H>
    where
        H: Hsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = random_bytes(32);

        Hsm::generate_aes_encryption_key(hsm, identifier).await.unwrap();

        let encrypted: Encrypted<Vec<u8>> = Hsm::encrypt(hsm, identifier, data.clone()).await.unwrap();
        assert_ne!(data.clone(), encrypted.data.clone());

        let decrypted = Hsm::decrypt(hsm, identifier, encrypted).await.unwrap();
        assert_eq!(data, decrypted);

        self
    }

    pub async fn encrypt_decrypt_verifying_key(self: TestCase<H>) -> TestCase<H>
    where
        H: Hsm + Encrypter<VerifyingKey> + Decrypter<VerifyingKey>,
    {
        let (hsm, identifier) = self.test_params();

        Hsm::generate_aes_encryption_key(hsm, identifier).await.unwrap();

        let verifying_key = *SigningKey::random(&mut OsRng).verifying_key();
        let encrypted = Encrypter::encrypt(hsm, identifier, verifying_key).await.unwrap();

        let decrypted = Decrypter::decrypt(hsm, identifier, encrypted).await.unwrap();

        assert_eq!(verifying_key, decrypted);

        self
    }

    pub async fn wrap_key_and_sign(self: TestCase<H>) -> TestCase<H>
    where
        H: Pkcs11Client,
    {
        let (hsm, identifier) = self.test_params();

        let _ = Pkcs11Client::generate_aes_encryption_key(hsm, identifier)
            .await
            .unwrap();

        let wrapped = hsm.generate_wrapped_key(identifier).await.unwrap();
        let public_key = *wrapped.public_key();

        let data = random_bytes(32);
        let signature = Pkcs11Client::sign_wrapped(hsm, identifier, wrapped, &data)
            .await
            .unwrap();

        public_key.verify(data.as_ref(), &signature).unwrap();

        self
    }
}
