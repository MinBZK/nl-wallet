use std::sync::Arc;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use hsm::model::encrypted::Encrypted;
use hsm::model::encrypter::Decrypter;
use hsm::model::encrypter::Encrypter;
use hsm::model::hsm::Hsm;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Client;
use hsm::service::Pkcs11Hsm;
use hsm::service::SigningMechanism;
use wallet_provider_domain::model::hsm::WalletUserHsm;
use wallet_provider_domain::model::wallet_user::WalletId;

type Result<T> = std::result::Result<T, HsmError>;

#[derive(Clone)]
pub struct WalletUserPkcs11Hsm {
    hsm: Pkcs11Hsm,
    wrapping_key_identifier: String,
}

impl WalletUserPkcs11Hsm {
    pub fn new(hsm: Pkcs11Hsm, wrapping_key_identifier: String) -> Self {
        Self {
            hsm,
            wrapping_key_identifier,
        }
    }

    pub fn hsm(&self) -> &Pkcs11Hsm {
        &self.hsm
    }
}

impl Hsm for WalletUserPkcs11Hsm {
    type Error = HsmError;

    async fn generate_generic_secret_key(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Hsm::generate_generic_secret_key(&self.hsm, identifier).await
    }
    async fn get_verifying_key(&self, identifier: &str) -> std::result::Result<VerifyingKey, Self::Error> {
        Hsm::get_verifying_key(&self.hsm, identifier).await
    }
    async fn delete_key(&self, identifier: &str) -> std::result::Result<(), Self::Error> {
        Hsm::delete_key(&self.hsm, identifier).await
    }
    async fn sign_ecdsa(&self, identifier: &str, data: Arc<Vec<u8>>) -> std::result::Result<Signature, Self::Error> {
        Hsm::sign_ecdsa(&self.hsm, identifier, data).await
    }
    async fn sign_hmac(&self, identifier: &str, data: Arc<Vec<u8>>) -> std::result::Result<Vec<u8>, Self::Error> {
        Hsm::sign_hmac(&self.hsm, identifier, data).await
    }
    async fn verify_hmac(
        &self,
        identifier: &str,
        data: Arc<Vec<u8>>,
        signature: Vec<u8>,
    ) -> std::result::Result<(), Self::Error> {
        Hsm::verify_hmac(&self.hsm, identifier, data, signature).await
    }
    async fn encrypt<T>(&self, identifier: &str, data: Vec<u8>) -> std::result::Result<Encrypted<T>, Self::Error> {
        Hsm::encrypt(&self.hsm, identifier, data).await
    }
    async fn decrypt<T>(&self, identifier: &str, encrypted: Encrypted<T>) -> std::result::Result<Vec<u8>, Self::Error> {
        Hsm::decrypt(&self.hsm, identifier, encrypted).await
    }
}

impl Encrypter<VerifyingKey> for WalletUserPkcs11Hsm {
    type Error = HsmError;

    async fn encrypt(
        &self,
        key_identifier: &str,
        data: VerifyingKey,
    ) -> std::result::Result<Encrypted<VerifyingKey>, Self::Error> {
        let bytes: Vec<u8> = data.to_sec1_bytes().to_vec();
        Hsm::encrypt(self, key_identifier, bytes).await
    }
}

impl Decrypter<VerifyingKey> for WalletUserPkcs11Hsm {
    type Error = HsmError;

    async fn decrypt(
        &self,
        key_identifier: &str,
        encrypted: Encrypted<VerifyingKey>,
    ) -> std::result::Result<VerifyingKey, Self::Error> {
        let decrypted = Hsm::decrypt(self, key_identifier, encrypted).await?;
        Ok(VerifyingKey::from_sec1_bytes(&decrypted)?)
    }
}

impl WalletUserHsm for WalletUserPkcs11Hsm {
    type Error = HsmError;

    async fn generate_wrapped_key(&self) -> Result<(VerifyingKey, WrappedKey)> {
        let private_wrapping_handle = self.hsm.get_private_key_handle(&self.wrapping_key_identifier).await?;
        let (public_handle, private_handle) = self.hsm.generate_session_signing_key_pair().await?;
        let verifying_key = Pkcs11Client::get_verifying_key(&self.hsm, public_handle).await?;

        let wrapped = self
            .hsm
            .wrap_key(private_wrapping_handle, private_handle, verifying_key)
            .await?;

        Ok((verifying_key, wrapped))
    }

    async fn generate_key(&self, wallet_id: &WalletId, identifier: &str) -> Result<VerifyingKey> {
        let key_identifier = hsm::model::hsm::key_identifier(wallet_id, identifier);
        let (public_handle, _private_handle) = self.hsm.generate_signing_key_pair(&key_identifier).await?;
        Pkcs11Client::get_verifying_key(&self.hsm, public_handle).await
    }

    async fn sign_wrapped(&self, wrapped_key: WrappedKey, data: Arc<Vec<u8>>) -> Result<Signature> {
        let private_wrapping_handle = self.hsm.get_private_key_handle(&self.wrapping_key_identifier).await?;
        let private_handle = self
            .hsm
            .unwrap_signing_key(private_wrapping_handle, wrapped_key)
            .await?;
        let signature = Pkcs11Client::sign(&self.hsm, private_handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }

    async fn sign(&self, wallet_id: &WalletId, identifier: &str, data: Arc<Vec<u8>>) -> Result<Signature> {
        let key_identifier = hsm::model::hsm::key_identifier(wallet_id, identifier);
        let handle = self.hsm.get_private_key_handle(&key_identifier).await?;
        let signature = Pkcs11Client::sign(&self.hsm, handle, SigningMechanism::Ecdsa256, data).await?;
        Ok(Signature::from_slice(&signature)?)
    }
}
