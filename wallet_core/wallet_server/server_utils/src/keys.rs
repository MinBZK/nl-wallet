use p256::ecdsa;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;

use crypto::keys::EcdsaKey;
use crypto::keys::EcdsaKeySend;
use crypto::x509::CertificateError;
use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use sd_jwt_vc_metadata::TypeMetadataChainError;

use crate::settings::PrivateKey;

pub enum PrivateKeyVariant {
    Software(SigningKey),
    Hsm(HsmEcdsaKey),
}

#[derive(Debug, thiserror::Error)]
pub enum PrivateKeyVariantError {
    #[error("software key error: {0}")]
    Software(#[from] ecdsa::Error),
    #[error("hardware key error: {0}")]
    Hardware(#[from] HsmError),
}

impl EcdsaKeySend for PrivateKeyVariant {
    type Error = PrivateKeyVariantError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let verifying_key = match self {
            PrivateKeyVariant::Software(signing_key) => EcdsaKeySend::verifying_key(signing_key).await?,
            PrivateKeyVariant::Hsm(hsm_key) => hsm_key.verifying_key().await?,
        };
        Ok(verifying_key)
    }
    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let signature = match self {
            PrivateKeyVariant::Software(signing_key) => EcdsaKeySend::try_sign(signing_key, msg).await?,
            PrivateKeyVariant::Hsm(hsm_key) => hsm_key.try_sign(msg).await?,
        };
        Ok(signature)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PrivateKeySettingsError {
    #[error("missing `hsm` settings for hardware key with identifier: {0}")]
    MissingHsmSettings(String),
    #[error("invalid certificate settings: {0}")]
    InvalidCertificate(#[from] CertificateError),
    #[error("missing metadata for attestation type {0}")]
    MissingMetadata(String),
    #[error("type metadata is not valid: {0}")]
    TypeMetadata(#[from] TypeMetadataChainError),
}

impl PrivateKeyVariant {
    pub fn from_settings(settings: PrivateKey, hsm: Option<Pkcs11Hsm>) -> Result<Self, PrivateKeySettingsError> {
        let pk = match settings {
            PrivateKey::Software { private_key } => Self::Software(private_key.into_inner()),
            PrivateKey::Hsm { private_key } => {
                let hsm = hsm.ok_or(PrivateKeySettingsError::MissingHsmSettings(private_key.clone()))?;
                Self::Hsm(HsmEcdsaKey::new(private_key, hsm))
            }
        };
        Ok(pk)
    }
}
