use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;
use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;

pub trait WalletCertificateSigningKey: SecureEcdsaKey {}
pub trait InstructionResultSigningKey: SecureEcdsaKey {}

pub struct WalletCertificateSigning(pub HsmEcdsaKey);
pub struct InstructionResultSigning(pub HsmEcdsaKey);

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

#[cfg(any(test, feature = "mock_secure_keys"))]
pub mod mock {
    use p256::ecdsa::SigningKey;

    use super::InstructionResultSigningKey;
    use super::WalletCertificateSigningKey;

    impl WalletCertificateSigningKey for SigningKey {}
    impl InstructionResultSigningKey for SigningKey {}
}
