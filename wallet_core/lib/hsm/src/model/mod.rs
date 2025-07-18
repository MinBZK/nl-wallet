pub mod encrypted;
pub mod encrypter;
pub mod wrapped_key;

use std::error::Error;
use std::sync::Arc;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::model::encrypted::Encrypted;

pub trait Hsm {
    type Error: Error + Send + Sync;

    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<(), Self::Error>;
    async fn generate_aes_encryption_key(&self, identifier: &str) -> Result<(), Self::Error>;
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(), Self::Error>;
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
    use hmac::Hmac;
    use hmac::Mac;
    use hmac::digest::MacError;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use p256::ecdsa::signature::Signer;
    use rand::rngs::OsRng;
    use sha2::Sha256;

    use crypto::utils::random_bytes;

    use crate::model::Hsm;
    use crate::model::encrypted::Encrypted;
    use crate::model::encrypted::InitializationVector;
    use crate::model::encrypter::Decrypter;
    use crate::model::encrypter::Encrypter;
    use crate::model::wrapped_key::WrappedKey;
    use crate::service::HsmError;
    use crate::service::Pkcs11Client;
    use crate::service::PrivateKeyHandle;

    type HmacSha256 = Hmac<Sha256>;

    pub struct MockPkcs11Client<E>(DashMap<String, SigningKey>, DashMap<String, Vec<u8>>, PhantomData<E>);

    impl<E> MockPkcs11Client<E> {
        pub fn get_key(&self, key_prefix: &str, identifier: &str) -> Result<SigningKey, E> {
            let key_identifier = format!("{key_prefix}_{identifier}");
            let entry = self.0.get(&key_identifier).unwrap();
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

        async fn generate_aes_encryption_key(&self, identifier: &str) -> Result<(), Self::Error> {
            self.1.insert(String::from(identifier), random_bytes(32));
            Ok(())
        }

        async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(), Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            self.0.insert(String::from(identifier), key);
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

        async fn encrypt<T>(&self, _identifier: &str, mut data: Vec<u8>) -> Result<Encrypted<T>, Self::Error> {
            // add byte to data, so that the encrypted representation is different from the original
            data.push(0);
            Ok(Encrypted::new(data, InitializationVector(random_bytes(32))))
        }

        async fn decrypt<T>(&self, _identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>, Self::Error> {
            // strip added byte to get the original back
            let mut data = encrypted.data;
            data.pop();
            Ok(data)
        }
    }

    impl<E> Pkcs11Client for MockPkcs11Client<E> {
        async fn generate_aes_encryption_key(&self, _identifier: &str) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn generate_generic_secret_key(&self, _identifier: &str) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn generate_session_signing_key_pair(
            &self,
        ) -> Result<(crate::service::PublicKeyHandle, PrivateKeyHandle), HsmError> {
            todo!()
        }

        async fn generate_signing_key_pair(
            &self,
            _identifier: &str,
        ) -> Result<(crate::service::PublicKeyHandle, PrivateKeyHandle), HsmError> {
            todo!()
        }

        async fn get_private_key_handle(&self, _identifier: &str) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn get_public_key_handle(&self, _identifier: &str) -> Result<crate::service::PublicKeyHandle, HsmError> {
            todo!()
        }

        async fn get_verifying_key(
            &self,
            _public_key_handle: crate::service::PublicKeyHandle,
        ) -> Result<VerifyingKey, HsmError> {
            todo!()
        }

        async fn delete_key(&self, _private_key_handle: PrivateKeyHandle) -> Result<(), HsmError> {
            todo!()
        }

        async fn sign(
            &self,
            _private_key_handle: PrivateKeyHandle,
            _mechanism: crate::service::SigningMechanism,
            _data: Arc<Vec<u8>>,
        ) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn verify(
            &self,
            _private_key_handle: PrivateKeyHandle,
            _mechanism: crate::service::SigningMechanism,
            _data: Arc<Vec<u8>>,
            _signature: Vec<u8>,
        ) -> Result<(), HsmError> {
            todo!()
        }

        async fn random_bytes(&self, _length: u32) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn encrypt(
            &self,
            _key_handle: PrivateKeyHandle,
            _iv: InitializationVector,
            _data: Vec<u8>,
        ) -> Result<(Vec<u8>, InitializationVector), HsmError> {
            todo!()
        }

        async fn decrypt(
            &self,
            _key_handle: PrivateKeyHandle,
            _iv: InitializationVector,
            _encrypted_data: Vec<u8>,
        ) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn wrap_key(
            &self,
            _wrapping_key: PrivateKeyHandle,
            _key: PrivateKeyHandle,
            _public_key: VerifyingKey,
        ) -> Result<WrappedKey, HsmError> {
            todo!()
        }

        async fn unwrap_signing_key(
            &self,
            _unwrapping_key: PrivateKeyHandle,
            _wrapped_key: WrappedKey,
        ) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn generate_wrapped_key(
            &self,
            _wrapping_key_identifier: &str,
        ) -> Result<(VerifyingKey, WrappedKey), HsmError> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            Ok((verifying_key, WrappedKey::new(key.to_bytes().to_vec(), verifying_key)))
        }

        async fn sign_wrapped(
            &self,
            _wrapping_key_identifier: &str,
            wrapped_key: WrappedKey,
            data: Arc<Vec<u8>>,
        ) -> Result<Signature, HsmError> {
            let key = SigningKey::from_slice(wrapped_key.wrapped_private_key()).unwrap();
            let signature = Signer::sign(&key, data.as_ref());
            Ok(signature)
        }
    }
}
