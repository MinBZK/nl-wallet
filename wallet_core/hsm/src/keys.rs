use std::sync::Arc;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use crate::model::Hsm;
use crate::service::HsmError;
use crate::service::Pkcs11Hsm;

pub struct HsmEcdsaKey {
    identifier: String,
    hsm: Pkcs11Hsm,
}

impl HsmEcdsaKey {
    pub fn new(identifier: String, hsm: Pkcs11Hsm) -> Self {
        Self { identifier, hsm }
    }
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
