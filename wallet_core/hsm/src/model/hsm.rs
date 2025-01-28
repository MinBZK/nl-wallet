use std::error::Error;
use std::sync::Arc;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::model::encrypted::Encrypted;

pub fn key_identifier(prefix: &str, identifier: &str) -> String {
    format!("{prefix}_{identifier}")
}

pub trait Hsm {
    type Error: Error + Send + Sync;

    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<(), Self::Error>;
    async fn get_verifying_key(&self, identifier: &str) -> Result<VerifyingKey, Self::Error>;
    async fn delete_key(&self, identifier: &str) -> Result<(), Self::Error>;
    async fn sign_ecdsa(&self, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error>;
    async fn sign_hmac(&self, identifier: &str, data: Arc<Vec<u8>>) -> Result<Vec<u8>, Self::Error>;
    async fn verify_hmac(&self, identifier: &str, data: Arc<Vec<u8>>, signature: Vec<u8>) -> Result<(), Self::Error>;
    async fn encrypt<T>(&self, identifier: &str, data: Vec<u8>) -> Result<Encrypted<T>, Self::Error>;
    async fn decrypt<T>(&self, identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>, Self::Error>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::error::Error;
    use std::marker::PhantomData;
    use std::sync::Arc;

    use dashmap::DashMap;
    use hmac::digest::MacError;
    use hmac::Hmac;
    use hmac::Mac;
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use sha2::Sha256;

    use wallet_common::utils::random_bytes;

    use crate::model::encrypted::Encrypted;
    use crate::model::encrypted::InitializationVector;
    use crate::model::encrypter::Decrypter;
    use crate::model::encrypter::Encrypter;
    use crate::model::hsm::key_identifier;
    use crate::model::hsm::Hsm;

    type HmacSha256 = Hmac<Sha256>;

    pub struct MockPkcs11Client<E>(DashMap<String, SigningKey>, DashMap<String, Vec<u8>>, PhantomData<E>);

    impl<E> MockPkcs11Client<E> {
        pub fn get_key(&self, key_prefix: &str, identifier: &str) -> Result<SigningKey, E> {
            let entry = self.0.get(&key_identifier(key_prefix, identifier)).unwrap();
            let key = entry.value().clone();
            Ok(key)
        }

        pub fn insert(&self, identifier: String, signing_key: SigningKey) -> Option<SigningKey> {
            self.0.insert(identifier, signing_key)
        }
    }

    impl<E> Default for MockPkcs11Client<E> {
        fn default() -> Self {
            Self(DashMap::new(), DashMap::new(), PhantomData)
        }
    }

    impl<E: Error + Send + Sync> Encrypter<VerifyingKey> for MockPkcs11Client<E> {
        type Error = E;

        async fn encrypt(
            &self,
            _key_identifier: &str,
            data: VerifyingKey,
        ) -> std::result::Result<Encrypted<VerifyingKey>, Self::Error> {
            let encrypted = Encrypted::new(data.to_sec1_bytes().to_vec(), InitializationVector(random_bytes(32)));
            Ok(encrypted)
        }
    }

    impl<E: Error + Send + Sync> Decrypter<VerifyingKey> for MockPkcs11Client<E> {
        type Error = E;

        async fn decrypt(
            &self,
            _key_identifier: &str,
            encrypted: Encrypted<VerifyingKey>,
        ) -> std::result::Result<VerifyingKey, Self::Error> {
            Ok(VerifyingKey::from_sec1_bytes(&encrypted.data).unwrap())
        }
    }

    impl<E: Error + Send + Sync + From<MacError>> Hsm for MockPkcs11Client<E> {
        type Error = E;

        async fn generate_generic_secret_key(&self, identifier: &str) -> Result<(), Self::Error> {
            self.1.insert(String::from(identifier), random_bytes(32));
            Ok(())
        }

        async fn get_verifying_key(&self, identifier: &str) -> Result<VerifyingKey, Self::Error> {
            let entry = self.0.get(identifier).unwrap();
            let key = entry.value();
            let verifying_key = key.verifying_key();
            Ok(*verifying_key)
        }

        async fn delete_key(&self, identifier: &str) -> Result<(), Self::Error> {
            self.0.remove(identifier).unwrap();
            Ok(())
        }

        async fn sign_ecdsa(&self, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error> {
            let entry = self.0.get(identifier).unwrap();
            let key = entry.value();

            let signature = Signer::sign(key, &data);
            Ok(signature)
        }

        async fn sign_hmac(&self, identifier: &str, data: Arc<Vec<u8>>) -> Result<Vec<u8>, Self::Error> {
            let entry = self.1.get(identifier).unwrap();
            let key = entry.value();

            let mut mac = HmacSha256::new_from_slice(key).unwrap();
            mac.update(&data);
            let signature = mac.finalize().into_bytes();

            Ok(signature.to_vec())
        }

        async fn verify_hmac(
            &self,
            identifier: &str,
            data: Arc<Vec<u8>>,
            signature: Vec<u8>,
        ) -> Result<(), Self::Error> {
            let entry = self.1.get(identifier).unwrap();
            let key = entry.value();

            let mut mac = HmacSha256::new_from_slice(key).unwrap();
            mac.update(&data);
            Ok(mac.verify_slice(&signature)?)
        }

        async fn encrypt<T>(&self, _identifier: &str, data: Vec<u8>) -> Result<Encrypted<T>, Self::Error> {
            Ok(Encrypted::new(data, InitializationVector(random_bytes(32))))
        }

        async fn decrypt<T>(&self, _identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>, Self::Error> {
            Ok(encrypted.data)
        }
    }
}
