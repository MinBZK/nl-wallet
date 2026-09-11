use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;
use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;
use jwt::KeyWithKid;
use nutype::nutype;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;

const WALLET_CERTIFICATE_SIGNING_KEY_PREFIX: &str = "wallet_certificate_signing_";
const PIN_HMAC_KEY_PREFIX: &str = "pin_hmac_";
const PIN_PUBKEY_ENCRYPTION_KEY_PREFIX: &str = "pin_pubkey_encryption_";
const ATTESTATION_WRAPPING_KEY_PREFIX: &str = "attestation_wrapping_";

#[nutype(
    derive(Debug, Clone, TryFrom, AsRef, Hash, PartialEq, Eq, Deserialize),
    validate(regex = r"^[\w-]+$")
)]
pub struct Kid(String);

pub fn certificate_signing_key_identifier(kid: &Kid) -> String {
    format!("{}{}", WALLET_CERTIFICATE_SIGNING_KEY_PREFIX, kid.as_ref())
}

pub fn pin_hmac_key_identifier(kid: &Kid) -> String {
    format!("{}{}", PIN_HMAC_KEY_PREFIX, kid.as_ref())
}

pub fn attestation_wrapping_key_identifier(kid: &Kid) -> String {
    format!("{}{}", ATTESTATION_WRAPPING_KEY_PREFIX, kid.as_ref())
}

pub fn pin_pubkey_encryption_key_identifier(kid: &Kid) -> String {
    format!("{}{}", PIN_PUBKEY_ENCRYPTION_KEY_PREFIX, kid.as_ref())
}

/// A pair of key identifiers ("kid"s) for a symmetric encryption key: the one currently used to encrypt, and
/// optionally the previous one, still needed to decrypt data that hasn't been re-encrypted yet as part of a
/// key rollover.
#[derive(Debug, Clone, Deserialize)]
pub struct KidPair {
    pub current: Kid,
    #[serde(default)]
    pub previous: Option<Kid>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown key kid: {0:?}")]
pub struct UnknownKid(pub Kid);

impl KidPair {
    /// Errors if `kid` is neither the current nor the previous kid.
    pub fn validate(&self, kid: &Kid) -> Result<(), UnknownKid> {
        if kid == &self.current || self.previous.as_ref() == Some(kid) {
            Ok(())
        } else {
            Err(UnknownKid(kid.clone()))
        }
    }
}

pub trait WalletCertificateSigningKey: SecureEcdsaKey + KeyWithKid {}
pub trait InstructionResultSigningKey: SecureEcdsaKey + KeyWithKid {}

pub struct WalletCertificateSigning {
    pub kid: Kid,
    pub key: HsmEcdsaKey,
}

pub struct InstructionResultSigning {
    pub kid: Kid,
    pub key: HsmEcdsaKey,
}

impl EcdsaKey for WalletCertificateSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.key.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.key.try_sign(msg).await
    }
}

impl EcdsaKey for InstructionResultSigning {
    type Error = HsmError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.key.verifying_key().await
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        self.key.try_sign(msg).await
    }
}

impl SecureEcdsaKey for WalletCertificateSigning {}

impl KeyWithKid for WalletCertificateSigning {
    fn kid(&self) -> &str {
        self.kid.as_ref()
    }
}

impl SecureEcdsaKey for InstructionResultSigning {}

impl KeyWithKid for InstructionResultSigning {
    fn kid(&self) -> &str {
        self.kid.as_ref()
    }
}

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
