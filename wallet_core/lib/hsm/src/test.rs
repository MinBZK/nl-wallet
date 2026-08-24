use std::collections::VecDeque;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use config::Config;
use config::ConfigError;
use crypto::aes_siv::AesSivBackend;
use crypto::aes_siv::AesSivKey;
use crypto::aes_siv::aes_siv_decrypt;
use crypto::aes_siv::aes_siv_encrypt;
use crypto::aes_siv::test::test_aes_cmac;
use crypto::aes_siv::test::test_aes_ctr;
use crypto::aes_siv::test::test_aes_siv_decrypt;
use crypto::aes_siv::test::test_aes_siv_encrypt;
use crypto::utils::random_bytes;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::ecdsa::signature::Verifier;
use p256::elliptic_curve::Generate;
use regex::regex;
use serde::Deserialize;
use serde_with::serde_as;
use tempfile::TempDir;
use utils::path::prefix_local_path;

use crate::model::Hsm;
use crate::model::TestHsm;
use crate::model::encrypted::Encrypted;
use crate::model::encrypter::Decrypter;
use crate::model::encrypter::Encrypter;
use crate::model::mock::MockPkcs11Client;
use crate::service::AES_BLOCK_SIZE;
use crate::service::AesKeyUsage;
use crate::service::HsmError;
use crate::service::Pkcs11Client;
use crate::service::Pkcs11Hsm;
use crate::service::SecretKeyHandle;
use crate::service::TestPkcs11Client;
use crate::settings;

pub async fn execute_hsm_test<F>(description: String, test: F)
where
    F: AsyncFnOnce(TestCase<Pkcs11Hsm>),
{
    let hsm_setup = HsmSetup::new();
    let test_case = TestCase::new(&hsm_setup, "hsm.toml", description);
    test(test_case).await;
}

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
        let caps = regex!(r#"(?m)^(directories\.tokendir) *= *(.+)$"#)
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
    identifier_prefix: String,
    hsm: H,
}

impl<H> TestCase<H> {
    /// Convenience function for test functions, returning the HSM as well as one key identifier for use in the test.
    pub fn test_params(&self) -> (&H, String) {
        let id = self.new_identifier();
        (&self.hsm, id)
    }

    pub fn new_identifier(&self) -> String {
        format!("{}-{}", self.identifier_prefix, crypto::utils::random_string(8))
    }
}

impl TestCase<MockPkcs11Client<HsmError>> {
    pub fn mock(identifier_prefix: String) -> Self {
        Self {
            identifier_prefix,
            hsm: MockPkcs11Client::default(),
        }
    }
}

impl TestCase<Pkcs11Hsm> {
    pub fn new(hsm_setup: &HsmSetup, config_file: &str, identifier_prefix: String) -> Self {
        let settings = TestSettings::new(config_file.as_ref()).unwrap();
        let hsm = hsm_setup.pkcs11_hsm(settings.hsm.clone()).unwrap();
        Self { identifier_prefix, hsm }
    }
}

// These methods are to be called by integration tests.
impl<H> TestCase<H> {
    pub async fn sign_sha256_hmac(self: TestCase<H>)
    where
        H: TestHsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = random_bytes(32);

        TestHsm::generate_generic_secret_key(hsm, &identifier).await.unwrap();
        let signature = hsm.sign_hmac(&identifier, &data).await.unwrap();
        hsm.verify_hmac(&identifier, &data, signature).await.unwrap();
    }

    pub async fn sign_ecdsa(self: TestCase<H>)
    where
        H: TestHsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = Arc::new(random_bytes(32));

        TestHsm::generate_signing_key_pair(hsm, &identifier).await.unwrap();

        let signature = hsm.sign_ecdsa(&identifier, &data).await.unwrap();
        let verifying_key = Hsm::get_verifying_key(hsm, &identifier).await.unwrap();
        verifying_key.verify(&data, &signature).unwrap();
    }

    pub async fn encrypt_decrypt(self: TestCase<H>)
    where
        H: TestHsm,
    {
        let (hsm, identifier) = self.test_params();
        let data = random_bytes(32);

        TestHsm::generate_aes_key(hsm, &identifier, AesKeyUsage::Encrypt)
            .await
            .unwrap();

        let encrypted: Encrypted<Vec<u8>> = Hsm::encrypt(hsm, &identifier, data.clone()).await.unwrap();
        assert_ne!(data.clone(), encrypted.data.clone());

        let decrypted = Hsm::decrypt(hsm, &identifier, encrypted).await.unwrap();
        assert_eq!(data, decrypted);
    }

    pub async fn encrypt_decrypt_verifying_key(self: TestCase<H>)
    where
        H: TestHsm + Encrypter<VerifyingKey> + Decrypter<VerifyingKey>,
    {
        let (hsm, identifier) = self.test_params();

        TestHsm::generate_aes_key(hsm, &identifier, AesKeyUsage::Encrypt)
            .await
            .unwrap();

        let verifying_key = *SigningKey::generate().verifying_key();
        let encrypted = Encrypter::encrypt(hsm, &identifier, verifying_key).await.unwrap();

        let decrypted = Decrypter::decrypt(hsm, &identifier, encrypted).await.unwrap();

        assert_eq!(verifying_key, decrypted);
    }

    pub async fn encrypt_ctr(self: TestCase<H>)
    where
        H: TestPkcs11Client,
    {
        let (hsm, identifier) = self.test_params();

        let key_handle = hsm.generate_aes_key(&identifier, AesKeyUsage::Encrypt).await.unwrap();

        let data = random_bytes(32);
        let counter_block: [u8; AES_BLOCK_SIZE] = random_bytes(AES_BLOCK_SIZE).try_into().unwrap();
        let encrypted = hsm.encrypt_ctr(&key_handle, counter_block, data.clone()).await.unwrap();
        assert_ne!(data, encrypted);

        // When encrypting something with the same key and same counter block, CTR encryption should be
        // deterministic.
        let encrypted_again = hsm.encrypt_ctr(&key_handle, counter_block, data.clone()).await.unwrap();
        assert_eq!(encrypted_again, encrypted);

        // CTR turns the block cipher into a stream cipher, so there is no padding and the
        // ciphertext is exactly as long as the plaintext.
        assert_eq!(data.len(), encrypted.len());

        // AES-CTR is symmetric, so encrypting the ciphertext under the same counter block returns
        // the plaintext.
        let decrypted = hsm.encrypt_ctr(&key_handle, counter_block, encrypted).await.unwrap();
        assert_eq!(data, decrypted);
    }

    pub async fn cmac(self: TestCase<H>)
    where
        H: TestPkcs11Client,
    {
        let (hsm, identifier) = self.test_params();

        // Note that a CMAC key is generated here, not an encryption key: the two usages are
        // mutually exclusive. SoftHSM does not enforce CKA_SIGN, so it would accept an encryption
        // key here, but a stricter HSM will not.
        let key_handle = hsm.generate_aes_key(&identifier, AesKeyUsage::Cmac).await.unwrap();

        let data = random_bytes(32);

        // The length is checked by the return type: a CMAC is exactly one block.
        let cmac: [u8; 16] = hsm.cmac(&key_handle, data.clone()).await.unwrap();

        // The same message under the same key gives the same tag, ...
        let cmac_again = hsm.cmac(&key_handle, data).await.unwrap();
        assert_eq!(cmac, cmac_again);

        // ... and a different message a different one.
        let other_cmac = hsm.cmac(&key_handle, random_bytes(32)).await.unwrap();
        assert_ne!(cmac, other_cmac);
    }

    pub async fn wrap_key_and_sign(self: TestCase<H>)
    where
        H: TestPkcs11Client,
    {
        let (hsm, identifier) = self.test_params();

        let _ = TestPkcs11Client::generate_aes_key(hsm, &identifier, AesKeyUsage::Encrypt)
            .await
            .unwrap();

        let wrapped = hsm.generate_wrapped_key(&identifier).await.unwrap();
        let public_key = *wrapped.public_key();

        let data = random_bytes(32);
        let signature = Pkcs11Client::sign_wrapped(hsm, &identifier, wrapped, &data)
            .await
            .unwrap();

        public_key.verify(data.as_ref(), &signature).unwrap();
    }

    pub async fn aes_siv(self: TestCase<H>)
    where
        H: TestPkcs11Client + AesSivBackend<MacKey = SecretKeyHandle, EncryptionKey = SecretKeyHandle>,
    {
        let mac_key_id = self.new_identifier();
        let enc_key_id = self.new_identifier();

        let hsm = &self.hsm;

        // Note the usages: CMAC needs CKA_SIGN, CTR needs CKA_ENCRYPT, and the two halves of K
        // must be distinct keys.
        let mac_key = hsm.generate_aes_key(&mac_key_id, AesKeyUsage::Cmac).await.unwrap();
        let encryption_key = hsm.generate_aes_key(&enc_key_id, AesKeyUsage::Encrypt).await.unwrap();

        let key = AesSivKey::try_new(mac_key, encryption_key).unwrap();

        // Test for some sizes. 16 bytes is the minimum supported plaintext size.
        for len in [16, 32, 255] {
            let plaintext: Vec<u8> = (0..len).map(|i| i as u8).collect();

            let ciphertext = aes_siv_encrypt(hsm, &key, plaintext.clone()).await.unwrap();
            assert_eq!(ciphertext.len(), 16 + plaintext.len());

            // AES-SIV is deterministic: encrypting the plaintext two times results in equal ciphertexts.
            let ciphertext_again = aes_siv_encrypt(hsm, &key, plaintext.clone()).await.unwrap();
            assert_eq!(ciphertext, ciphertext_again);

            assert_eq!(
                aes_siv_decrypt(hsm, &key, ciphertext).await.unwrap().as_slice(),
                plaintext.as_slice()
            );
        }
    }
}

// This test is not generic over `H`, unlike those above, because it needs to import keys with known
// values, which only `Pkcs11Hsm` can do.
impl TestCase<Pkcs11Hsm> {
    /// Runs the pinned AES-SIV vectors from the `crypto` crate through the HSM, checking that the
    /// PKCS#11 primitives produce RFC 5297 output rather than merely output they agree with
    /// themselves on.
    ///
    /// This also tests the `CK_AES_CTR_PARAMS` marshalling in [`Pkcs11Hsm::encrypt_ctr()`], and
    /// with it the `cryptoki-sys` patch in the workspace manifest that keeps that struct's layout
    /// in step with the one `cryptoki` itself uses. Neither has anything else watching it.
    ///
    /// Known answers need keys with known values, so this imports its keys instead of generating
    /// them. SoftHSM supports this, but a production HSM will not. If these tests would be run
    /// against an actual HSM, these tests will fail and will have to be disabled.
    pub async fn aes_siv_encrypt_test_vectors(self: TestCase<Pkcs11Hsm>) {
        let hsm = &self.hsm;

        self.ensure_keys_can_be_imported(hsm).await;

        test_aes_siv_encrypt(hsm, self.hsm_siv_key_generator(hsm)).await;
    }

    pub async fn aes_siv_decrypt_test_vectors(self: TestCase<Pkcs11Hsm>) {
        let hsm = &self.hsm;

        self.ensure_keys_can_be_imported(hsm).await;

        test_aes_siv_decrypt(hsm, self.hsm_siv_key_generator(hsm)).await;
    }

    pub async fn aes_cmac_test_vectors(self: TestCase<Pkcs11Hsm>) {
        let hsm = &self.hsm;

        self.ensure_keys_can_be_imported(hsm).await;

        test_aes_cmac(hsm, self.hsm_key_generator(hsm, AesKeyUsage::Cmac)).await;
    }

    pub async fn aes_ctr_test_vectors(self: TestCase<Pkcs11Hsm>) {
        let hsm = &self.hsm;

        self.ensure_keys_can_be_imported(hsm).await;

        test_aes_ctr(hsm, self.hsm_key_generator(hsm, AesKeyUsage::Encrypt)).await;
    }

    /// Ensure that we can import keys into the HSM. If not, this function will panic.
    async fn ensure_keys_can_be_imported(&self, hsm: &Pkcs11Hsm) {
        match hsm
            .import_aes_key(&self.new_identifier(), AesKeyUsage::Cmac, [0; 32])
            .await
        {
            Ok(_) => {}
            Err(error) if error.is_key_import_unsupported() => {
                // These tests are currently only run against SoftHSM, which can import keys.
                panic!(
                    "Cannot import keys. This HSM (simulator) does not accept imported key material ({error}). \
                     Disable this test."
                );
            }
            Err(error) => panic!("failed to import AES key: {error}"),
        }
    }

    /// Imports a key for the CMAC and CTR vectors.
    fn hsm_key_generator(&self, hsm: &Pkcs11Hsm, usage: AesKeyUsage) -> impl AsyncFn([u8; 32]) -> SecretKeyHandle {
        async move |key| {
            hsm.import_aes_key(&self.new_identifier(), usage, key)
                .await
                .expect("failed to import AES key")
        }
    }

    /// The two-key counterpart of [`Self::hsm_key_generator()`], for the AES-SIV vectors,
    /// which need a CMAC and a CTR key per case.
    fn hsm_siv_key_generator(
        &self,
        hsm: &Pkcs11Hsm,
    ) -> impl AsyncFn(([u8; 32], [u8; 32])) -> (SecretKeyHandle, SecretKeyHandle) {
        let cmac_key_generator = self.hsm_key_generator(hsm, AesKeyUsage::Cmac);
        let ctr_key_generator = self.hsm_key_generator(hsm, AesKeyUsage::Encrypt);

        async move |(mac_key, encryption_key)| {
            (
                cmac_key_generator(mac_key).await,
                ctr_key_generator(encryption_key).await,
            )
        }
    }
}
