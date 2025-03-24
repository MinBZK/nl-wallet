use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;

use chrono::Days;
use derive_more::AsRef;
use derive_more::From;
use futures::future::join_all;
use indexmap::IndexMap;
use rustls_pki_types::TrustAnchor;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;

use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use hsm::service::Pkcs11Hsm;
use mdoc::utils::x509::CertificateType;
use openid4vc::issuer::AttestationTypeConfig;
use openid4vc::issuer::AttestationTypesConfig;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::TypeMetadataChain;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::verify_key_pairs;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::Settings;
use wallet_common::generator::TimeGenerator;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::urls::HttpsUri;
use wallet_common::utils;

#[derive(Clone, Deserialize)]
pub struct IssuerSettings {
    pub attestation_settings: AttestationTypesConfigSettings,

    #[serde(deserialize_with = "deserialize_type_metadata")]
    pub metadata: Vec<TypeMetadata>,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: Vec<String>,

    #[serde(flatten)]
    pub server_settings: Settings,
}

#[derive(Clone, Deserialize, From, AsRef)]
pub struct AttestationTypesConfigSettings(HashMap<String, AttestationTypeConfigSettings>);

#[derive(Clone, Deserialize)]
pub struct AttestationTypeConfigSettings {
    #[serde(flatten)]
    pub keypair: KeyPair,

    pub valid_days: u64,
    pub copy_count: NonZeroU8,

    /// Which of the SAN fields in the issuer certificate to use as the `issuer_uri`/`iss` field in the mdoc/SD-JWT.
    /// If the certificate contains exactly one SAN, then this may be left blank.
    pub certificate_san: Option<HttpsUri>,
}

fn deserialize_type_metadata<'de, D>(deserializer: D) -> Result<Vec<TypeMetadata>, D::Error>
where
    D: Deserializer<'de>,
{
    let path = Vec::<String>::deserialize(deserializer)?;

    let metadatas = path
        .iter()
        .map(|path| {
            let metadata_file = fs::read(utils::prefix_local_path(path.as_ref())).map_err(de::Error::custom)?;
            serde_json::from_slice(metadata_file.as_slice())
        })
        .collect::<Result<_, _>>()
        .map_err(de::Error::custom)?;

    Ok(metadatas)
}

impl IssuerSettings {
    pub fn metadata(&self) -> IndexMap<String, TypeMetadata> {
        self.metadata
            .iter()
            .map(|type_metadata| (type_metadata.as_ref().vct.clone(), type_metadata.clone()))
            .collect()
    }
}

impl AttestationTypesConfigSettings {
    pub async fn parse(
        self,
        hsm: &Option<Pkcs11Hsm>,
        metadata: &IndexMap<String, TypeMetadata>,
    ) -> Result<AttestationTypesConfig<PrivateKeyVariant>, PrivateKeySettingsError> {
        let issuer_keys = join_all(self.0.into_iter().map(|(typ, attestation)| {
            async move {
                // Take the SAN from the settings if specified, or otherwise take the first SAN from the certificate.
                // NB: the settings validation function will have verified before this that the certificate contains
                // just one SAN.
                let issuer_uri = attestation
                    .certificate_san
                    .map(Ok::<_, CertificateError>) // Make it a result as the next closure is fallible
                    .unwrap_or_else(|| Ok(attestation.keypair.certificate.san_dns_name_or_uris()?.first().clone()))?;

                let metadata = metadata
                    .get(&typ)
                    .ok_or_else(|| PrivateKeySettingsError::MissingMetadata(typ.clone()))?;
                let metadata = TypeMetadataChain::create(metadata.clone(), vec![])?;

                Ok((
                    typ.clone(),
                    AttestationTypeConfig {
                        key_pair: attestation.keypair.parse(hsm.clone()).await?,
                        valid_days: Days::new(attestation.valid_days),
                        copy_count: attestation.copy_count,
                        issuer_uri: issuer_uri.clone(),
                        metadata,
                    },
                ))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<IndexMap<String, AttestationTypeConfig<PrivateKeyVariant>>, PrivateKeySettingsError>>()?;

        Ok(issuer_keys.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerSettingsError {
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
    #[error("error verifying certificate: {0}")]
    CertificateVerification(#[from] CertificateVerificationError),
    #[error("certificate for {attestation_type} missing SAN {san}")]
    CertificateMissingSan { attestation_type: String, san: HttpsUri },
    #[error("multiple SANs in issuer certificate for {attestation_type}: which one to use was not specified")]
    CertificateSanUnspecified { attestation_type: String },
    #[error("missing metadata for attestation {attestation_type}")]
    MissingMetadata { attestation_type: String },
}

impl IssuerSettings {
    pub fn validate(&self) -> Result<(), IssuerSettingsError> {
        tracing::debug!("verifying issuer settings");

        let metadata = self.metadata();
        for (typ, attestation) in self.attestation_settings.as_ref() {
            if !metadata.contains_key(typ) {
                // TODO PVW-3824: recursively check the presence of metadata on which the current metadata depends
                return Err(IssuerSettingsError::MissingMetadata {
                    attestation_type: typ.clone(),
                });
            }

            if let Some(certificate_san) = attestation.certificate_san.as_ref() {
                // If the certificate SAN to be used has been specified, then it has to be present in the certificate.
                if !attestation
                    .keypair
                    .certificate
                    .san_dns_name_or_uris()?
                    .as_ref()
                    .contains(certificate_san)
                {
                    return Err(IssuerSettingsError::CertificateMissingSan {
                        attestation_type: typ.clone(),
                        san: certificate_san.clone(),
                    });
                }
            } else {
                // If not, then there must be only one SAN in the certificate so there is no disambiguation.
                if attestation.keypair.certificate.san_dns_name_or_uris()?.len().get() > 1 {
                    return Err(IssuerSettingsError::CertificateSanUnspecified {
                        attestation_type: typ.clone(),
                    });
                }
            }
        }

        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'_>> = self
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(String, KeyPair)> = self
            .attestation_settings
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.clone(), attestation.keypair.clone()))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::Mdl,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::Mdl(Some(_))),
        )?;

        Ok(())
    }
}
