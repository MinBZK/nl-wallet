pub mod encrypted;
pub mod encrypter;
pub mod wrapped_key;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crate::model::encrypted::Encrypted;

pub trait Hsm {
    type Error: std::error::Error + Send + Sync;

    async fn get_verifying_key(&self, identifier: &str) -> Result<VerifyingKey, Self::Error>;
    async fn sign_ecdsa(&self, identifier: &str, data: &[u8]) -> Result<Signature, Self::Error>;
    async fn sign_hmac(&self, identifier: &str, data: &[u8]) -> Result<Vec<u8>, Self::Error>;
    async fn verify_hmac(&self, identifier: &str, data: &[u8], signature: Vec<u8>) -> Result<(), Self::Error>;
    async fn encrypt<T>(&self, identifier: &str, data: Vec<u8>) -> Result<Encrypted<T>, Self::Error>;
    async fn decrypt<T>(&self, identifier: &str, encrypted: Encrypted<T>) -> Result<Vec<u8>, Self::Error>;
}

#[cfg(any(feature = "test", feature = "mock"))]
pub trait TestHsm: Hsm {
    async fn generate_generic_secret_key(&self, identifier: &str) -> Result<(), Self::Error>;
    async fn generate_aes_key(&self, identifier: &str, usage: crate::service::AesKeyUsage) -> Result<(), Self::Error>;
    async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(), Self::Error>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::error::Error;
    use std::marker::PhantomData;

    use crypto::utils::random_bytes;
    use dashmap::DashMap;
    use hmac::Hmac;
    use hmac::Mac;
    use hmac::digest::MacError;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use p256::ecdsa::signature::Signer;
    use p256::elliptic_curve::Generate;
    use sha2::Sha256;

    use crate::model::Hsm;
    use crate::model::encrypted::Encrypted;
    use crate::model::encrypted::InitializationVector;
    use crate::model::encrypter::Decrypter;
    use crate::model::encrypter::Encrypter;
    use crate::model::wrapped_key::WrappedKey;
    use crate::service::AES_BLOCK_SIZE;
    use crate::service::AesKeyUsage;
    use crate::service::HsmError;
    use crate::service::KeyHandle;
    use crate::service::Pkcs11Client;
    use crate::service::PrivateKeyHandle;
    use crate::service::PublicKeyHandle;
    use crate::service::SecretKeyHandle;
    use crate::service::SignVerifyKeyHandle;

    type HmacSha256 = Hmac<Sha256>;

    pub struct MockPkcs11Client<E>(DashMap<String, SigningKey>, DashMap<String, Vec<u8>>, PhantomData<E>);

    impl<E> MockPkcs11Client<E> {
        pub fn get_signing_key(&self, key_prefix: &str, identifier: &str) -> Result<SigningKey, E> {
            let key_identifier = format!("{key_prefix}_{identifier}");
            let entry = self.0.get(&key_identifier).unwrap();
            let key = entry.value().clone();
            Ok(key)
        }

        pub fn insert_signing_key(&self, identifier: String, signing_key: SigningKey) -> Option<SigningKey> {
            self.0.insert(identifier, signing_key)
        }

        pub fn remove_symmetric_key(&self, identifier: &str) -> Option<(String, Vec<u8>)> {
            self.1.remove(identifier)
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
        ) -> Result<Encrypted<VerifyingKey>, Self::Error> {
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
        ) -> Result<VerifyingKey, Self::Error> {
            Ok(VerifyingKey::from_sec1_bytes(&encrypted.data).unwrap())
        }
    }

    impl<E: Error + Send + Sync + From<MacError>> crate::model::TestHsm for MockPkcs11Client<E> {
        async fn generate_generic_secret_key(&self, identifier: &str) -> Result<(), Self::Error> {
            self.1.insert(String::from(identifier), random_bytes(32));
            Ok(())
        }

        async fn generate_aes_key(&self, identifier: &str, _usage: AesKeyUsage) -> Result<(), Self::Error> {
            self.1.insert(String::from(identifier), random_bytes(32));
            Ok(())
        }

        async fn generate_signing_key_pair(&self, identifier: &str) -> Result<(), Self::Error> {
            let key = SigningKey::generate();
            self.0.insert(String::from(identifier), key);
            Ok(())
        }
    }

    impl<E: Error + Send + Sync + From<MacError>> Hsm for MockPkcs11Client<E> {
        type Error = E;

        async fn get_verifying_key(&self, identifier: &str) -> Result<VerifyingKey, Self::Error> {
            let entry = self.0.get(identifier).unwrap();
            let key = entry.value();
            let verifying_key = key.verifying_key();
            Ok(*verifying_key)
        }

        async fn sign_ecdsa(&self, identifier: &str, data: &[u8]) -> Result<Signature, Self::Error> {
            let entry = self.0.get(identifier).unwrap();
            let key = entry.value();

            let signature = Signer::sign(key, data);
            Ok(signature)
        }

        async fn sign_hmac(&self, identifier: &str, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
            let entry = self.1.get(identifier).ok_or(MacError)?;
            let key = entry.value();

            let mut mac = HmacSha256::new_from_slice(key).unwrap();
            mac.update(data);
            let signature = mac.finalize().into_bytes();

            Ok(signature.to_vec())
        }

        async fn verify_hmac(&self, identifier: &str, data: &[u8], signature: Vec<u8>) -> Result<(), Self::Error> {
            let entry = self.1.get(identifier).unwrap();
            let key = entry.value();

            let mut mac = HmacSha256::new_from_slice(key).unwrap();
            mac.update(data);
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
        async fn generate_session_signing_key_pair(&self) -> Result<(PublicKeyHandle, PrivateKeyHandle), HsmError> {
            todo!()
        }
        async fn get_secret_key_handle(&self, _identifier: &str) -> Result<SecretKeyHandle, HsmError> {
            todo!()
        }

        async fn get_private_key_handle(&self, _identifier: &str) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn get_public_key_handle(&self, _identifier: &str) -> Result<PublicKeyHandle, HsmError> {
            todo!()
        }

        async fn get_verifying_key(&self, _public_key_handle: &PublicKeyHandle) -> Result<VerifyingKey, HsmError> {
            todo!()
        }

        async fn delete_key(&self, _key_handle: impl KeyHandle) -> Result<(), HsmError> {
            todo!()
        }

        async fn sign<KH: SignVerifyKeyHandle>(&self, _key_handle: &KH, _data: &[u8]) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn verify<KH: SignVerifyKeyHandle>(
            &self,
            _key_handle: &KH,
            _data: &[u8],
            _signature: Vec<u8>,
        ) -> Result<(), HsmError> {
            todo!()
        }

        async fn encrypt(
            &self,
            _key_handle: &SecretKeyHandle,
            _iv: InitializationVector,
            _data: Vec<u8>,
        ) -> Result<(Vec<u8>, InitializationVector), HsmError> {
            todo!()
        }

        async fn decrypt(
            &self,
            _key_handle: &SecretKeyHandle,
            _iv: InitializationVector,
            _encrypted_data: Vec<u8>,
        ) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn encrypt_ctr(
            &self,
            _key_handle: &SecretKeyHandle,
            _counter_block: [u8; AES_BLOCK_SIZE],
            _data: impl AsRef<[u8]> + Send + 'static,
        ) -> Result<Vec<u8>, HsmError> {
            todo!()
        }

        async fn cmac(
            &self,
            _key_handle: &SecretKeyHandle,
            _data: impl AsRef<[u8]> + Send + 'static,
        ) -> Result<[u8; AES_BLOCK_SIZE], HsmError> {
            todo!()
        }

        async fn wrap_key(
            &self,
            _wrapping_key: &SecretKeyHandle,
            _key: &PrivateKeyHandle,
            _public_key: VerifyingKey,
        ) -> Result<WrappedKey, HsmError> {
            todo!()
        }

        async fn unwrap_signing_key(
            &self,
            _unwrapping_key: &SecretKeyHandle,
            _wrapped_key: WrappedKey,
        ) -> Result<PrivateKeyHandle, HsmError> {
            todo!()
        }

        async fn generate_wrapped_key(&self, _wrapping_key_identifier: &str) -> Result<WrappedKey, HsmError> {
            let key = SigningKey::generate();
            Ok(WrappedKey::new(key.to_bytes().to_vec(), *key.verifying_key()))
        }

        async fn sign_wrapped(
            &self,
            _wrapping_key_identifier: &str,
            wrapped_key: WrappedKey,
            data: &[u8],
        ) -> Result<Signature, HsmError> {
            let key = SigningKey::from_slice(wrapped_key.wrapped_private_key()).unwrap();
            let signature = Signer::sign(&key, data);
            Ok(signature)
        }
    }
}
