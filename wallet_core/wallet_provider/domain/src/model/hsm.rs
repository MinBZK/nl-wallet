use std::error::Error;
use std::sync::Arc;

use futures::future;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use hsm::service::HsmError;
use hsm::service::Pkcs11Client;
use hsm::service::Pkcs11Hsm;
use hsm::service::SigningMechanism;

use crate::model::wallet_user::WalletId;

pub trait WalletUserHsm: Pkcs11Client {
    type Error: Error + Send + Sync;

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

    async fn sign(&self, wallet_id: &WalletId, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature, Self::Error>;

    async fn sign_multiple(
        &self,
        wallet_id: &WalletId,
        identifiers: &[&str],
        data: Arc<Vec<u8>>,
    ) -> Result<Vec<(String, Signature)>, Self::Error> {
        future::try_join_all(identifiers.iter().map(|identifier| async {
            WalletUserHsm::sign(self, wallet_id, identifier, Arc::clone(&data))
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

    async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error> {
        let key_identifier = key_identifier(wallet_id, identifier);
        let (public_handle, _private_handle) = self.generate_signing_key_pair(&key_identifier).await?;
        Pkcs11Client::get_verifying_key(self, public_handle).await
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
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand::rngs::OsRng;

    use hsm::model::Hsm;
    use hsm::model::mock::MockPkcs11Client;

    use crate::model::hsm::WalletUserHsm;
    use crate::model::wallet_user::WalletId;

    fn key_identifier(wallet_id: &WalletId, identifier: &str) -> String {
        format!("{wallet_id}_{identifier}")
    }

    impl<E: Error + Send + Sync + From<MacError>> WalletUserHsm for MockPkcs11Client<E> {
        type Error = E;

        async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey, Self::Error> {
            let key = SigningKey::random(&mut OsRng);
            let verifying_key = *key.verifying_key();
            let key_identifier = key_identifier(wallet_id, identifier);
            self.insert(key_identifier, key);
            Ok(verifying_key)
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
