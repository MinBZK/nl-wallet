use derive_more::Constructor;
use derive_more::Debug;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;

use crate::model::Hsm;
use crate::service::HsmError;
use crate::service::Pkcs11Hsm;

#[derive(Debug, Clone, Constructor)]
pub struct HsmEcdsaKey {
    identifier: String,
    #[debug(skip)]
    hsm: Pkcs11Hsm,
}

impl EcdsaKey for HsmEcdsaKey {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.hsm.get_verifying_key(&self.identifier).await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Hsm::sign_ecdsa(&self.hsm, &self.identifier, msg).await
    }
}

impl SecureEcdsaKey for HsmEcdsaKey {}

#[derive(Constructor)]
pub struct HsmHmacKey {
    identifier: String,
    hsm: Pkcs11Hsm,
}

impl HsmHmacKey {
    pub async fn sign_hmac(&self, msg: &[u8]) -> Result<Vec<u8>, HsmError> {
        Hsm::sign_hmac(&self.hsm, &self.identifier, msg).await
    }
}
