use std::error::Error;
use std::sync::Arc;

use futures::future;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use hsm::model::encrypted::Encrypted;

use crate::model::wallet_user::WalletId;
use crate::model::wrapped_key::WrappedKey;

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
    use std::error::Error;
    use std::sync::Arc;

    use hmac::digest::MacError;
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand::rngs::OsRng;

    use hsm::model::hsm::key_identifier;
    use hsm::model::hsm::mock::MockPkcs11Client;
    use hsm::model::hsm::Hsm;

    use crate::model::hsm::WalletUserHsm;
    use crate::model::wallet_user::WalletId;
    use crate::model::wrapped_key::WrappedKey;

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
            self.insert(key_identifier(wallet_id, identifier), key);
            Ok(verifying_key)
        }

        async fn sign_wrapped(&self, wrapped_key: WrappedKey, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error> {
            let key = SigningKey::from_slice(wrapped_key.wrapped_private_key()).unwrap();
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
}
