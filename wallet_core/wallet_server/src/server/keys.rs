use p256::ecdsa;
use p256::ecdsa::SigningKey;

use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use nl_wallet_mdoc::utils::x509::CertificateError;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::EcdsaKeySend;

use crate::settings::PrivateKey;

pub enum PrivateKeyType {
    Software(SigningKey),
    Hardware(HsmEcdsaKey),
}

#[derive(Debug, thiserror::Error)]
pub enum PrivateKeyTypeError {
    #[error("software key error: {0}")]
    Software(#[from] ecdsa::Error),
    #[error("hardware key error: {0}")]
    Hardware(#[from] HsmError),
}

impl EcdsaKeySend for PrivateKeyType {
    type Error = PrivateKeyTypeError;

    async fn verifying_key(&self) -> Result<p256::ecdsa::VerifyingKey, Self::Error> {
        let verifying_key = match self {
            PrivateKeyType::Software(signing_key) => EcdsaKeySend::verifying_key(signing_key).await?,
            PrivateKeyType::Hardware(hsm_key) => hsm_key.verifying_key().await?,
        };
        Ok(verifying_key)
    }
    async fn try_sign(&self, msg: &[u8]) -> Result<p256::ecdsa::Signature, Self::Error> {
        let signature = match self {
            PrivateKeyType::Software(signing_key) => EcdsaKeySend::try_sign(signing_key, msg).await?,
            PrivateKeyType::Hardware(hsm_key) => hsm_key.try_sign(msg).await?,
        };
        Ok(signature)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("missing `hsm` settings for hardware key with identifier: {0}")]
    MissingHsmSettings(String),
    #[error("invalid certificate settings: {0}")]
    InvalidCertificate(#[from] CertificateError),
}

impl PrivateKeyType {
    pub fn from_settings(settings: PrivateKey, hsm: Option<Pkcs11Hsm>) -> Result<Self, KeyError> {
        let pk = match settings {
            PrivateKey::Software(signing_key) => Self::Software(signing_key.into_inner()),
            PrivateKey::Hardware(identifier) => {
                let hsm = hsm.ok_or(KeyError::MissingHsmSettings(identifier.clone()))?;
                Self::Hardware(HsmEcdsaKey::new(identifier, hsm))
            }
        };
        Ok(pk)
    }
}
