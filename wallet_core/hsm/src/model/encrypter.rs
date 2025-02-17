use std::error::Error;

use p256::ecdsa::VerifyingKey;

use crate::model::encrypted::Encrypted;
use crate::model::Hsm;
use crate::service::HsmError;
use crate::service::Pkcs11Hsm;

pub trait Encrypter<T> {
    type Error: Error;

    async fn encrypt(&self, key_identifier: &str, data: T) -> Result<Encrypted<T>, Self::Error>;
}

pub trait Decrypter<T> {
    type Error: Error;

    async fn decrypt(&self, key_identifier: &str, encrypted: Encrypted<T>) -> Result<T, Self::Error>;
}

impl Encrypter<VerifyingKey> for Pkcs11Hsm {
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

impl Decrypter<VerifyingKey> for Pkcs11Hsm {
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
