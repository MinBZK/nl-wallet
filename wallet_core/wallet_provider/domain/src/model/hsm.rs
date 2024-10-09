use std::{error::Error, sync::Arc};

use futures::future;
use p256::ecdsa::{Signature, VerifyingKey};

use crate::model::{encrypted::Encrypted, wallet_user::WalletId, wrapped_key::WrappedKey};

pub fn key_identifier(prefix: &str, identifier: &str) -> String {
    format!("{prefix}_{identifier}")
}

pub trait WalletUserHsm {
    type Error: Error + Send + Sync;

    async fn generate_wrapped_key(&self) -> Result<(VerifyingKey, WrappedKey), Self::Error>;

    async fn generate_wrapped_keys(
        &self,
        identifiers: &[&str],
    ) -> Result<Vec<(String, VerifyingKey, WrappedKey)>, Self::Error> {
        future::try_join_all(identifiers.iter().map(|identifier| async move {
            let result = self.generate_wrapped_key().await;
            result.map(|(pub_key, wrapped)| (String::from(*identifier), pub_key, wrapped))
        }))
        .await
    }

    async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error>;

    async fn generate_keys(
        &self,
        wallet_id: &WalletId,
        identifiers: &[&str],
    ) -> Result<Vec<(String, VerifyingKey)>, Self::Error> {
        future::try_join_all(identifiers.iter().map(|identifier| async move {
            let result = self.generate_key(wallet_id, identifier).await;
            result.map(|pub_key| (String::from(*identifier), pub_key))
        }))
        .await
    }

    async fn sign_wrapped(&self, wrapped_key: WrappedKey, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error>;

    async fn sign(&self, wallet_id: &WalletId, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error>;

    async fn sign_multiple(
        &self,
        wallet_id: &WalletId,
        identifiers: &[&str],
        data: Arc<Vec<u8>>,
    ) -> Result<Vec<(String, Signature)>, Self::Error> {
        future::try_join_all(identifiers.iter().map(|identifier| async {
            self.sign(wallet_id, identifier, Arc::clone(&data))
                .await
                .map(|signature| (String::from(*identifier), signature))
        }))
        .await
    }
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
    use std::{error::Error, marker::PhantomData, sync::Arc};

    use dashmap::DashMap;
    use hmac::{digest::MacError, Hmac, Mac};
    use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use sha2::Sha256;

    use wallet_common::utils::random_bytes;

    use crate::model::{
        encrypted::{Encrypted, InitializationVector},
        encrypter::{Decrypter, Encrypter},
        hsm::{key_identifier, Hsm, WalletUserHsm},
        wallet_user::WalletId,
        wrapped_key::WrappedKey,
    };

    type HmacSha256 = Hmac<Sha256>;

    pub struct MockPkcs11Client<E>(DashMap<String, SigningKey>, DashMap<String, Vec<u8>>, PhantomData<E>);

    impl<E> MockPkcs11Client<E> {
        pub fn get_key(&self, key_prefix: &str, identifier: &str) -> Result<SigningKey, E> {
            let entry = self.0.get(&key_identifier(key_prefix, identifier)).unwrap();
            let key = entry.value().clone();
            Ok(key)
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

    impl<E: Error + Send + Sync + From<MacError>> WalletUserHsm for MockPkcs11Client<E> {
        type Error = E;

        async fn generate_wrapped_key(&self) -> Result<(VerifyingKey, WrappedKey), Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            Ok((verifying_key, WrappedKey::new(key.to_bytes().to_vec(), verifying_key)))
        }

        async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            self.0.insert(key_identifier(wallet_id, identifier), key);
            Ok(verifying_key)
        }

        async fn sign_wrapped(&self, wrapped_key: WrappedKey, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error> {
            let wrapped_key: Vec<u8> = wrapped_key.into();
            let key = SigningKey::from_slice(&wrapped_key).unwrap();
            let signature = Signer::sign(&key, data.as_ref());
            Ok(signature)
        }

        async fn sign(
            &self,
            wallet_id: &WalletId,
            identifier: &str,
            data: Arc<Vec<u8>>,
        ) -> Result<Signature, Self::Error> {
            Hsm::sign_ecdsa(self, &key_identifier(wallet_id, identifier), data).await
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
