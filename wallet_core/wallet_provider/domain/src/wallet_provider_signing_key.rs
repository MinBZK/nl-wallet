use p256::ecdsa::{signature, signature::Signer, Error, Signature, VerifyingKey};
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

impl EcdsaKey for WalletProviderEcdsaKey {
    type Error = signature::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.0 .0.verifying_key())
    }
}

impl Signer<Signature> for WalletProviderEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, Error> {
        self.0 .0.try_sign(msg)
    }
}

impl SecureEcdsaKey for WalletProviderEcdsaKey {}
