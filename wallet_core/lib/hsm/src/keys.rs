use std::sync::Arc;

use derive_more::Constructor;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;

use crate::model::Hsm;
use crate::service::HsmError;
use crate::service::Pkcs11Hsm;

#[derive(Constructor)]
pub struct HsmEcdsaKey {
    identifier: String,
    hsm: Pkcs11Hsm,
}

impl EcdsaKey for HsmEcdsaKey {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.hsm.get_verifying_key(&self.identifier).await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Hsm::sign_ecdsa(&self.hsm, &self.identifier, Arc::new(msg.into())).await
    }
}

impl SecureEcdsaKey for HsmEcdsaKey {}
