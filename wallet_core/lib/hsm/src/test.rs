use std::path::Path;

use async_dropper::AsyncDrop;
use async_dropper::AsyncDropper;
use async_trait::async_trait;
use config::Config;
use config::ConfigError;
use config::File;
use serde::Deserialize;
use serde_with::serde_as;

use utils::path::prefix_local_path;

use crate::model::Hsm;
use crate::model::mock::MockPkcs11Client;
use crate::service::HsmError;
use crate::service::Pkcs11Hsm;
use crate::settings;

#[serde_as]
#[derive(Clone, Deserialize)]
struct TestSettings {
    pub(crate) hsm: settings::Hsm,
}

impl TestSettings {
    fn new(config_file: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(prefix_local_path(config_file).as_ref()).required(true))
            .build()?
            .try_deserialize()
    }
}

// Default is needed for AsyncDrop
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
    pub fn new(config_file: &str, identifier_prefix: &str) -> Self {
        let settings = TestSettings::new(config_file.as_ref()).unwrap();
        let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).unwrap();
        Self {
            identifier: format!("{}-{}", identifier_prefix, crypto::utils::random_string(8)),
            hsm: Some(hsm),
        }
    }
}

#[cfg(feature = "integration")]
mod integration {
    use std::sync::Arc;

    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use p256::ecdsa::signature::Verifier;
    use rand_core::OsRng;

    use crypto::utils::random_bytes;

    use crate::model::Hsm;
    use crate::model::encrypted::Encrypted;
    use crate::model::encrypter::Decrypter;
    use crate::model::encrypter::Encrypter;
    use crate::service::Pkcs11Client;

    use super::TestCase;

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
}
