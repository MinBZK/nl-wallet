use p256::ecdsa::{signature, signature::Signer, Error, Signature, VerifyingKey};
use wallet_common::account::{
    serialization::DerSigningKey,
    signing_key::{EcdsaKey, SecureEcdsaKey},
};

pub struct HsmEcdsaKey(DerSigningKey);

impl From<DerSigningKey> for HsmEcdsaKey {
    fn from(val: DerSigningKey) -> Self {
        HsmEcdsaKey(val)
    }
}

impl EcdsaKey for HsmEcdsaKey {
    type Error = signature::Error;

    fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.0 .0.verifying_key())
    }
}

impl Signer<Signature> for HsmEcdsaKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, Error> {
        self.0 .0.try_sign(msg)
    }
}

impl SecureEcdsaKey for HsmEcdsaKey {}
