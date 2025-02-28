use std::collections::HashMap;

use futures::future::join_all;
use p256::ecdsa;
use p256::ecdsa::Signature;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;

use hsm::keys::HsmEcdsaKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use nl_wallet_mdoc::server_keys::KeyPair as ParsedKeyPair;
use nl_wallet_mdoc::utils::x509::CertificateError;
use openid4vc_server::issuer::IssuerKeyRing;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::EcdsaKeySend;

use crate::settings::KeyPair;
use crate::settings::PrivateKey;
use crate::settings::TryFromKeySettings;

pub enum PrivateKeyVariant {
    Software(SigningKey),
    Hardware(HsmEcdsaKey),
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
            PrivateKeyVariant::Hardware(hsm_key) => hsm_key.verifying_key().await?,
        };
        Ok(verifying_key)
    }
    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        let signature = match self {
            PrivateKeyVariant::Software(signing_key) => EcdsaKeySend::try_sign(signing_key, msg).await?,
            PrivateKeyVariant::Hardware(hsm_key) => hsm_key.try_sign(msg).await?,
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
}

impl PrivateKeyVariant {
    pub fn from_settings(settings: PrivateKey, hsm: Option<Pkcs11Hsm>) -> Result<Self, PrivateKeySettingsError> {
        let pk = match settings {
            PrivateKey::Software(signing_key) => Self::Software(signing_key.into_inner()),
            PrivateKey::Hardware(identifier) => {
                let hsm = hsm.ok_or(PrivateKeySettingsError::MissingHsmSettings(identifier.clone()))?;
                Self::Hardware(HsmEcdsaKey::new(identifier, hsm))
            }
        };
        Ok(pk)
    }
}

impl TryFromKeySettings<HashMap<String, KeyPair>> for IssuerKeyRing<PrivateKeyVariant> {
    type Error = PrivateKeySettingsError;

    async fn try_from_key_settings(
        private_keys: HashMap<String, KeyPair>,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<Self, Self::Error> {
        let iter = private_keys.into_iter().map(|(doctype, key_pair)| async {
            let result = (
                doctype,
                ParsedKeyPair::try_from_key_settings(key_pair, hsm.clone()).await?,
            );
            Ok(result)
        });

        let issuer_keys = join_all(iter)
            .await
            .into_iter()
            .collect::<Result<HashMap<String, ParsedKeyPair<PrivateKeyVariant>>, Self::Error>>()?;

        Ok(issuer_keys.into())
    }
}
