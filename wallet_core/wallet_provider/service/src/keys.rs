use std::sync::Arc;

use p256::ecdsa::{Signature, VerifyingKey};

use wallet_common::keys::{EcdsaKey, SecureEcdsaKey, WithIdentifier};
use wallet_provider_domain::model::hsm::Hsm;

use crate::hsm::{HsmError, Pkcs11Hsm};

pub trait WalletCertificateSigningKey: SecureEcdsaKey + WithIdentifier {}
pub trait InstructionResultSigningKey: SecureEcdsaKey + WithIdentifier {}

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
impl WithIdentifier for WalletCertificateSigning {
    fn identifier(&self) -> &str {
        &self.0.identifier
    }
}

impl SecureEcdsaKey for InstructionResultSigning {}
impl WithIdentifier for InstructionResultSigning {
    fn identifier(&self) -> &str {
        &self.0.identifier
    }
}

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

impl WithIdentifier for WalletProviderEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl SecureEcdsaKey for WalletProviderEcdsaKey {}

#[cfg(any(test, feature = "software_keys"))]
pub mod mock {
    use wallet_common::keys::software::SoftwareEcdsaKey;

    use crate::keys::{InstructionResultSigningKey, WalletCertificateSigningKey};

    impl WalletCertificateSigningKey for SoftwareEcdsaKey {}
    impl InstructionResultSigningKey for SoftwareEcdsaKey {}
}
