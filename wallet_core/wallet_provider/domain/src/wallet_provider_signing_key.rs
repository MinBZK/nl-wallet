use async_trait::async_trait;
use p256::ecdsa::{signature, Signature, VerifyingKey};
use wallet_common::{
    account::serialization::DerSigningKey,
    keys::{EcdsaKey, SecureEcdsaKey},
};

pub struct WalletProviderEcdsaKey(DerSigningKey);

impl From<DerSigningKey> for WalletProviderEcdsaKey {
    fn from(val: DerSigningKey) -> Self {
        WalletProviderEcdsaKey(val)
    }
}

#[async_trait]
impl EcdsaKey for WalletProviderEcdsaKey {
    type Error = signature::Error;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.0 .0.verifying_key())
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        p256::ecdsa::signature::Signer::try_sign(&self.0 .0, msg)
    }
}

impl SecureEcdsaKey for WalletProviderEcdsaKey {}
