use std::sync::Arc;

use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use hsm::model::hsm::Hsm;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use crate::hsm::HsmError;
use crate::hsm::Pkcs11Hsm;

pub trait WalletCertificateSigningKey: SecureEcdsaKey {}
pub trait InstructionResultSigningKey: SecureEcdsaKey {}

pub struct WalletCertificateSigning(pub WalletProviderEcdsaKey);
pub struct InstructionResultSigning(pub WalletProviderEcdsaKey);

impl EcdsaKey for WalletCertificateSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.0.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.0.try_sign(msg).await
    }
}

impl EcdsaKey for InstructionResultSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.0.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.0.try_sign(msg).await
    }
}

impl SecureEcdsaKey for WalletCertificateSigning {}

impl SecureEcdsaKey for InstructionResultSigning {}

impl WalletCertificateSigningKey for WalletCertificateSigning {}
impl InstructionResultSigningKey for InstructionResultSigning {}

pub struct WalletProviderEcdsaKey {
    identifier: String,
    hsm: Pkcs11Hsm,
}

impl WalletProviderEcdsaKey {
    pub fn new(identifier: String, hsm: Pkcs11Hsm) -> Self {
        Self { identifier, hsm }
    }
}

impl EcdsaKey for WalletProviderEcdsaKey {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.hsm.get_verifying_key(&self.identifier).await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Hsm::sign_ecdsa(&self.hsm, &self.identifier, Arc::new(msg.into())).await
    }
}

impl SecureEcdsaKey for WalletProviderEcdsaKey {}

#[cfg(any(test, feature = "mock_secure_keys"))]
pub mod mock {
    use p256::ecdsa::SigningKey;

    use super::InstructionResultSigningKey;
    use super::WalletCertificateSigningKey;

    impl WalletCertificateSigningKey for SigningKey {}
    impl InstructionResultSigningKey for SigningKey {}
}
