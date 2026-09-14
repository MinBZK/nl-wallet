use crypto::keys::EcdsaKey;
use crypto::keys::SecureEcdsaKey;
use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;
use jwt::KeyWithKid;
use nutype::nutype;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;

const WALLET_CERTIFICATE_SIGNING_KEY_PREFIX: &str = "wallet_certificate_signing_";
const PIN_HMAC_KEY_PREFIX: &str = "pin_hmac_";

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
