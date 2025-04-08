use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;

use chrono::Days;
use derive_more::AsRef;
use derive_more::From;
use futures::future::join_all;
use rustls_pki_types::TrustAnchor;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;

use crypto::trust_anchor::BorrowingTrustAnchor;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use hsm::service::Pkcs11Hsm;
use mdoc::utils::x509::CertificateType;
use mdoc::AttestationQualification;
use openid4vc::issuer::AttestationTypeConfig;
use openid4vc::issuer::AttestationTypesConfig;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::verify_key_pairs;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::Settings;
use wallet_common::generator::TimeGenerator;
use wallet_common::urls::HttpsUri;
use wallet_common::utils;

#[derive(Clone, Deserialize)]
pub struct IssuerSettings {
    pub attestation_settings: AttestationTypesConfigSettings,

    #[serde(deserialize_with = "deserialize_type_metadata")]
    pub metadata: HashMap<String, Vec<u8>>,

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

    #[serde(default)]
    pub attestation_qualification: AttestationQualification,

    /// Which of the SAN fields in the issuer certificate to use as the `issuer_uri`/`iss` field in the mdoc/SD-JWT.
    /// If the certificate contains exactly one SAN, then this may be left blank.
    pub certificate_san: Option<HttpsUri>,
}

fn deserialize_type_metadata<'de, D>(deserializer: D) -> Result<HashMap<String, Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let path = Vec::<String>::deserialize(deserializer)?;

    // Map the contents of each JSON file by the `vct` field by decoding the JSON and extracting just that field.
    let documents = path
        .iter()
        .map(|path| {
            let json = fs::read(utils::prefix_local_path(path.as_ref())).map_err(de::Error::custom)?;
            let metadata =
                serde_json::from_slice::<UncheckedTypeMetadata>(json.as_slice()).map_err(de::Error::custom)?;

            Ok((metadata.vct, json))
        })
        .collect::<Result<_, _>>()?;

    Ok(documents)
}

impl AttestationTypesConfigSettings {
    pub async fn parse(
        self,
        hsm: &Option<Pkcs11Hsm>,
        metadata: &HashMap<String, Vec<u8>>,
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

                let metadata_document = metadata
                    .get(&typ)
                    .ok_or_else(|| PrivateKeySettingsError::MissingMetadata(typ.clone()))?;
                // This `.unwrap()` is guaranteed to succeed because we are supplying exactly one entry.
                let metadata_documents =
                    TypeMetadataDocuments::new(vec![metadata_document.clone()].try_into().unwrap());

                let config = AttestationTypeConfig::try_new(
                    &typ,
                    attestation.keypair.parse(hsm.clone()).await?,
                    Days::new(attestation.valid_days),
                    attestation.copy_count,
                    issuer_uri,
                    attestation.attestation_qualification,
                    metadata_documents,
                )?;

                Ok((typ, config))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<HashMap<String, AttestationTypeConfig<PrivateKeyVariant>>, PrivateKeySettingsError>>()?;

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

        for (typ, attestation) in self.attestation_settings.as_ref() {
            if !self.metadata.contains_key(typ) {
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

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .attestation_settings
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.as_ref(), &attestation.keypair))
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;

    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::server_keys::generate::Ca;
    use mdoc::server_keys::generate::mock::generate_issuer_mock;
    use mdoc::utils::issuer_auth::IssuerRegistration;
    use mdoc::AttestationQualification;
    use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use server_utils::settings::CertificateVerificationError;
    use server_utils::settings::Server;
    use server_utils::settings::Settings;
    use server_utils::settings::Storage;
    use wallet_common::urls::HttpsUri;

    use super::AttestationTypeConfigSettings;
    use super::IssuerSettings;
    use crate::settings::IssuerSettingsError;

    fn mock_settings() -> IssuerSettings {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let keypair = generate_issuer_mock(&issuer_ca, Some(IssuerRegistration::new_mock()))
            .expect("generate issuer cert failed")
            .into();

        IssuerSettings {
            attestation_settings: HashMap::from([(
                "com.example.pid".to_string(),
                AttestationTypeConfigSettings {
                    keypair,
                    valid_days: 365,
                    copy_count: 10.try_into().unwrap(),
                    attestation_qualification: AttestationQualification::PubEAA,
                    certificate_san: Some(("https://".to_string() + ISSUANCE_CERT_CN).parse().unwrap()),
                },
            )])
            .into(),
            metadata: HashMap::from([(
                "com.example.pid".to_string(),
                TypeMetadataDocuments::pid_example().1.into_inner().first().clone(),
            )]),
            wallet_client_ids: vec![MOCK_WALLET_CLIENT_ID.to_string()],
            server_settings: Settings {
                wallet_server: Server {
                    ip: "127.0.0.1".parse().unwrap(),
                    port: 42,
                },
                public_url: "https://example.com".parse().unwrap(),
                log_requests: false,
                structured_logging: false,
                storage: Storage {
                    url: "memory://".parse().unwrap(),
                    expiration_minutes: 10.try_into().unwrap(),
                    successful_deletion_minutes: 10.try_into().unwrap(),
                    failed_deletion_minutes: 10.try_into().unwrap(),
                },
                issuer_trust_anchors: vec![issuer_ca.as_borrowing_trust_anchor().clone()],
                hsm: None,
            },
        }
    }

    #[test]
    fn test_validate() {
        mock_settings().validate().unwrap();
    }

    #[test]
    fn test_no_issuer_trust_anchors() {
        let mut settings = mock_settings();

        settings.server_settings.issuer_trust_anchors = vec![];

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
        );
    }

    #[test]
    fn test_no_issuer_registration() {
        let mut settings = mock_settings();

        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
        let issuer_cert_no_registration =
            generate_issuer_mock(&issuer_ca, None).expect("generate issuer cert without issuer registration");

        settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
        settings.attestation_settings = HashMap::from([(
            "com.example.no_registration".to_string(),
            AttestationTypeConfigSettings {
                keypair: issuer_cert_no_registration.into(),
                valid_days: 365,
                copy_count: 4.try_into().unwrap(),
                attestation_qualification: Default::default(),
                certificate_san: None,
            },
        )])
        .into();

        let no_registration_metadata = UncheckedTypeMetadata {
            vct: "com.example.no_registration".to_string(),
            ..UncheckedTypeMetadata::empty_example()
        };
        let pid_metadata = TypeMetadata::pid_example();

        settings.metadata = HashMap::from([
            (
                no_registration_metadata.vct.clone(),
                serde_json::to_vec(&no_registration_metadata).unwrap(),
            ),
            (
                pid_metadata.as_ref().vct.clone(),
                serde_json::to_vec(&pid_metadata).unwrap(),
            ),
        ]);

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsError::CertificateVerification(CertificateVerificationError::IncompleteCertificateType(key))
                if key == "com.example.no_registration"
        );
    }

    #[test]
    fn test_missing_metadata() {
        let mut settings = mock_settings();

        settings.metadata.clear();

        let error = settings.validate().expect_err("should fail");
        assert_matches!(error, IssuerSettingsError::MissingMetadata { .. });
    }

    #[test]
    fn test_wrong_san_field() {
        let mut settings = mock_settings();

        let wrong_san: HttpsUri = "https://wrong.san.example.com".parse().unwrap();

        let (typ, attestation_settings) = settings.attestation_settings.as_ref().iter().next().unwrap();
        let mut attestation_settings = attestation_settings.clone();
        attestation_settings.certificate_san = Some(wrong_san.clone());
        settings.attestation_settings = HashMap::from([(typ.clone(), attestation_settings)]).into();

        let error = settings.validate().expect_err("should fail");
        assert_matches!(error, IssuerSettingsError::CertificateMissingSan { san, .. } if san == wrong_san);
    }
}
