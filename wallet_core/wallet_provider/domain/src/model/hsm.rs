use std::error::Error;
use std::sync::Arc;

use futures::future;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Client;
use hsm::service::Pkcs11Hsm;
use hsm::service::SigningMechanism;

use crate::model::wallet_user::WalletId;

pub trait WalletUserHsm {
    type Error: Error + Send + Sync;

    async fn generate_wrapped_key(
        &self,
        wrapping_key_identifier: &str,
    ) -> Result<(VerifyingKey, WrappedKey), Self::Error>;

    async fn generate_wrapped_keys(
        &self,
        wrapping_key_identifier: &str,
        identifiers: &[&str],
    ) -> Result<Vec<(String, VerifyingKey, WrappedKey)>, Self::Error> {
        future::try_join_all(identifiers.iter().map(|identifier| async move {
            let result = self.generate_wrapped_key(wrapping_key_identifier).await;
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

    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: Arc<Vec<u8>>,
    ) -> Result<Signature, Self::Error>;

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

fn key_identifier(wallet_id: &WalletId, identifier: &str) -> String {
    format!("{wallet_id}_{identifier}")
}

impl WalletUserHsm for Pkcs11Hsm {
    type Error = HsmError;

    async fn generate_wrapped_key(
        &self,
        wrapping_key_identifier: &str,
    ) -> Result<(VerifyingKey, WrappedKey), Self::Error> {
        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let (public_handle, private_handle) = self.generate_session_signing_key_pair().await?;
        let verifying_key = Pkcs11Client::get_verifying_key(self, public_handle).await?;

        let wrapped = self
            .wrap_key(private_wrapping_handle, private_handle, verifying_key)
            .await?;

        Ok((verifying_key, wrapped))
    }

    async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error> {
        let key_identifier = key_identifier(wallet_id, identifier);
        let (public_handle, _private_handle) = self.generate_signing_key_pair(&key_identifier).await?;
        Pkcs11Client::get_verifying_key(self, public_handle).await
    }

    async fn sign_wrapped(
        &self,
        wrapping_key_identifier: &str,
        wrapped_key: WrappedKey,
        data: Arc<Vec<u8>>,
    ) -> Result<Signature, Self::Error> {
        let private_wrapping_handle = self.get_private_key_handle(wrapping_key_identifier).await?;
        let private_handle = self.unwrap_signing_key(private_wrapping_handle, wrapped_key).await?;
        let signature = Pkcs11Client::sign(self, private_handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }

    async fn sign(&self, wallet_id: &WalletId, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error> {
        let key_identifier = key_identifier(wallet_id, identifier);
        let handle = self.get_private_key_handle(&key_identifier).await?;
        let signature = Pkcs11Client::sign(self, handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }
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

    use hsm::model::mock::MockPkcs11Client;
    use hsm::model::wrapped_key::WrappedKey;
    use hsm::model::Hsm;

    use crate::model::hsm::WalletUserHsm;
    use crate::model::wallet_user::WalletId;

    fn key_identifier(wallet_id: &WalletId, identifier: &str) -> String {
        format!("{wallet_id}_{identifier}")
    }

    impl<E: Error + Send + Sync + From<MacError>> WalletUserHsm for MockPkcs11Client<E> {
        type Error = E;

        async fn generate_wrapped_key(
            &self,
            _wrapping_key_identifier: &str,
        ) -> Result<(VerifyingKey, WrappedKey), Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            Ok((verifying_key, WrappedKey::new(key.to_bytes().to_vec(), verifying_key)))
        }

        async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            let key_identifier = key_identifier(wallet_id, identifier);
            self.insert(key_identifier, key);
            Ok(verifying_key)
        }

        async fn sign_wrapped(
            &self,
            _wrapping_key_identifier: &str,
            wrapped_key: WrappedKey,
            data: Arc<Vec<u8>>,
        ) -> Result<Signature, Self::Error> {
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
            let key_identifier = key_identifier(wallet_id, identifier);
            Hsm::sign_ecdsa(self, &key_identifier, data).await
        }
    }
}
