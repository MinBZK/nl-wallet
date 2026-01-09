use std::borrow::Cow;

use p256::ecdsa;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use ring::hmac;
use ring::hmac::HMAC_SHA256;

use crypto::keys::EcdsaKey;
use crypto::keys::EcdsaKeySend;
use crypto::x509::CertificateError;
use hsm::keys::HsmEcdsaKey;
use hsm::keys::HsmHmacKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use sd_jwt_vc_metadata::TypeMetadataChainError;

use crate::settings::PrivateKey;
use crate::settings::SecretKey;

#[derive(Debug, Clone)]
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

#[derive(Debug, thiserror::Error)]
pub enum SecretKeyVariantError {
    #[error("HMAC verification failed")]
    SoftwareVerification,
    #[error("HSM error: {0}")]
    Hsm(#[from] HsmError),
}

pub enum SecretKeyVariant {
    Software(hmac::Key),
    Hsm(HsmHmacKey),
}

impl SecretKeyVariant {
    pub async fn sign_hmac(&self, data: &[u8]) -> Result<Vec<u8>, HsmError> {
        match self {
            SecretKeyVariant::Software(key) => Ok(hmac::sign(key, data).as_ref().to_vec()),
            SecretKeyVariant::Hsm(key) => Ok(key.sign_hmac(data).await?),
        }
    }

    pub async fn verify_hmac(&self, data: &[u8], tag: Cow<'_, [u8]>) -> Result<(), SecretKeyVariantError> {
        match self {
            SecretKeyVariant::Software(key) => {
                hmac::verify(key, data, &tag).map_err(|_| SecretKeyVariantError::SoftwareVerification)?
            }
            SecretKeyVariant::Hsm(key) => key.verify_hmac(data, tag.into_owned()).await?,
        };

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecretKeySettingsError {
    #[error("missing `hsm` settings for hardware key with identifier: {0}")]
    MissingHsmSettings(String),
}

impl SecretKeyVariant {
    pub fn from_settings(settings: SecretKey, hsm: Option<Pkcs11Hsm>) -> Result<Self, SecretKeySettingsError> {
        let key = match settings {
            SecretKey::Software { secret_key } => {
                SecretKeyVariant::Software(hmac::Key::new(HMAC_SHA256, secret_key.as_ref()))
            }
            SecretKey::Hsm { secret_key } => {
                let hsm = hsm.ok_or(SecretKeySettingsError::MissingHsmSettings(secret_key.clone()))?;
                SecretKeyVariant::Hsm(HsmHmacKey::new(secret_key, hsm))
            }
        };
        Ok(key)
    }
}

#[cfg(feature = "test")]
pub mod test {
    use crypto::server_keys::KeyPair;

    use super::*;

    pub async fn private_key_variant(pair: KeyPair<SigningKey>) -> KeyPair<PrivateKeyVariant> {
        KeyPair::new(
            PrivateKeyVariant::Software(pair.private_key().clone()),
            pair.certificate().clone(),
        )
        .await
        .unwrap()
    }
}
