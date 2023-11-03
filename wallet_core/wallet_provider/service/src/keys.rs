use std::sync::Arc;

use async_trait::async_trait;
use p256::ecdsa::{Signature, VerifyingKey};

use wallet_common::keys::{EcdsaKey, SecureEcdsaKey, WithIdentifier};
use wallet_provider_domain::model::hsm::Hsm;

use crate::hsm::{HsmError, Pkcs11Hsm};

pub trait CertificateSigningKey: SecureEcdsaKey {}
pub trait InstructionResultSigningKey: SecureEcdsaKey {}

pub struct CertificateSigning(pub WalletProviderEcdsaKey);
pub struct InstructionResultSigning(pub WalletProviderEcdsaKey);

#[async_trait]
impl EcdsaKey for CertificateSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.0.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.0.try_sign(msg).await
    }
}

#[async_trait]
impl EcdsaKey for InstructionResultSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.0.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.0.try_sign(msg).await
    }
}

impl SecureEcdsaKey for CertificateSigning {}
impl WithIdentifier for CertificateSigning {
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

impl CertificateSigningKey for CertificateSigning {}
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

#[async_trait]
impl EcdsaKey for WalletProviderEcdsaKey {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.hsm.get_verifying_key(&self.identifier).await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        Hsm::sign(&self.hsm, &self.identifier, Arc::new(msg.into())).await
    }
}

impl WithIdentifier for WalletProviderEcdsaKey {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl SecureEcdsaKey for WalletProviderEcdsaKey {}

#[cfg(feature = "mock")]
pub mod mock {
    use wallet_common::keys::software::SoftwareEcdsaKey;

    use crate::keys::{CertificateSigningKey, InstructionResultSigningKey};

    impl CertificateSigningKey for SoftwareEcdsaKey {}
    impl InstructionResultSigningKey for SoftwareEcdsaKey {}
}
