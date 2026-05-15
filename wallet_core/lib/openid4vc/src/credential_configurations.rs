use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;

use attestation_types::qualification::AttestationQualification;
use chrono::Days;
use crypto::server_keys::KeyPair;
use derive_more::Debug;
use derive_more::From;
use http_utils::urls::HttpsUri;
use itertools::Either;
use itertools::Itertools;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use ssri::Integrity;

use crate::Format;
use crate::metadata::issuer_metadata;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::ProofType;

#[derive(Debug, thiserror::Error)]
pub enum CredentialConfigurationsError {
    #[error("no credential configuration parameters provided")]
    NoConfigurations,

    #[error("could not parse SD-JWT VC Type Metadata chain: {0}")]
    TypeMetadata(#[source] TypeMetadataChainError),

    #[error("unsupported credential format: {0}")]
    UnsupportedFormat(Format),

    #[error(
        "multiple credential configurations for the same combination of format and attestation type: {}",
        .0.values().map(|config_ids| config_ids.iter().join(", ")).join(" / ")
    )]
    DuplicateFormatAndAttestationType(HashMap<(Format, String), HashSet<CredentialConfigurationId>>),
}

#[derive(Debug)]
pub struct CredentialConfigurationParameters<K, L> {
    pub format: Format,
    pub attestation_type: String,
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub status_list: L,
    pub valid_days: Days,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
    #[debug(skip)]
    pub metadata_documents: TypeMetadataDocuments,
}

/// Static attestation data shared across all instances of an attestation type for a particular format. Parts of this
/// configuration are represented in the `credential_configurations_supported` section of the issuer metadata.
///
/// When performing issuance, the issuer augments the [`CredentialConfiguration`] with an [`IssuableDocument`] to form
/// the attestation.
#[derive(Debug)]
pub(crate) struct CredentialConfiguration<K, L> {
    pub format: Format,
    pub attestation_type: String,
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub status_list: L,
    pub valid_days: Days,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
    pub metadata: CredentialConfigurationMetadata,
}

#[derive(Debug)]
pub(crate) struct CredentialConfigurationMetadata {
    #[debug(skip)]
    documents: SortedTypeMetadataDocuments,
    first_document_integrity: Integrity,
    normalized: NormalizedTypeMetadata,
}

impl<K, L> CredentialConfiguration<K, L> {
    /// Create a new [`CredentialConfiguration`] and decode and validate the type metadata documents.
    pub fn try_new(
        CredentialConfigurationParameters {
            format,
            attestation_type,
            key_pair,
            status_list,
            valid_days,
            issuer_uri,
            attestation_qualification,
            metadata_documents,
        }: CredentialConfigurationParameters<K, L>,
    ) -> Result<Self, TypeMetadataChainError> {
        let metadata = CredentialConfigurationMetadata::try_new(&attestation_type, metadata_documents)?;

        let config = Self {
            format,
            attestation_type,
            status_list,
            key_pair,
            valid_days,
            issuer_uri,
            attestation_qualification,
            metadata,
        };

        Ok(config)
    }
}

impl CredentialConfigurationMetadata {
    fn try_new(attestation_type: &str, documents: TypeMetadataDocuments) -> Result<Self, TypeMetadataChainError> {
        // Calculate and cache the integrity hash for the first metadata document in the chain.
        let first_document_integrity = Integrity::from(documents.as_ref().first().as_slice());
        let (normalized, documents) = documents.into_normalized(attestation_type)?;

        let metadata = Self {
            documents,
            first_document_integrity,
            normalized,
        };

        Ok(metadata)
    }

    pub fn documents(&self) -> &SortedTypeMetadataDocuments {
        &self.documents
    }

    pub fn first_document_integrity(&self) -> &Integrity {
        &self.first_document_integrity
    }

    pub fn normalized(&self) -> &NormalizedTypeMetadata {
        &self.normalized
    }
}

/// Static credential configurations indexed by their identifier.
#[derive(Debug, From)]
pub(crate) struct CredentialConfigurations<K, L> {
    configs_by_id: HashMap<CredentialConfigurationId, CredentialConfiguration<K, L>>,
    ids_by_format_and_attestation_type: HashMap<(Format, Cow<'static, str>), CredentialConfigurationId>,
}

impl<K, L> CredentialConfigurations<K, L> {
    pub fn try_new(
        config_params: HashMap<CredentialConfigurationId, CredentialConfigurationParameters<K, L>>,
    ) -> Result<Self, CredentialConfigurationsError> {
        if config_params.is_empty() {
            return Err(CredentialConfigurationsError::NoConfigurations);
        }

        let mut ids_by_format_and_attestation_type = HashMap::<_, Vec<_>>::new();

        let configs_by_id = config_params
            .into_iter()
            .map(|(config_id, params)| {
                if !params.format.is_supported() {
                    return Err(CredentialConfigurationsError::UnsupportedFormat(params.format));
                }

                let format_and_attestation_type = (params.format, params.attestation_type.clone());
                ids_by_format_and_attestation_type
                    .entry(format_and_attestation_type)
                    .or_default()
                    .push(config_id.clone());

                let config =
                    CredentialConfiguration::try_new(params).map_err(CredentialConfigurationsError::TypeMetadata)?;

                Ok((config_id, config))
            })
            .try_collect()?;

        let (ids_by_format_and_attestation_type, duplicate_format_and_attestation_type) =
            ids_by_format_and_attestation_type
                .into_iter()
                .partition_map::<_, HashMap<_, _>, _, _, _>(|((format, attestation_type), ids)| {
                    match ids.into_iter().exactly_one() {
                        Ok(id) => Either::Left(((format, Cow::Owned(attestation_type)), id)),
                        Err(ids) => Either::Right(((format, attestation_type), ids.collect())),
                    }
                });

        if !duplicate_format_and_attestation_type.is_empty() {
            return Err(CredentialConfigurationsError::DuplicateFormatAndAttestationType(
                duplicate_format_and_attestation_type,
            ));
        }

        let credential_configurations = Self {
            configs_by_id,
            ids_by_format_and_attestation_type,
        };

        Ok(credential_configurations)
    }

    pub fn configurations(&self) -> impl Iterator<Item = &CredentialConfiguration<K, L>> {
        self.configs_by_id.values()
    }

    pub fn get_by_configuration_id(
        &self,
        config_id: &CredentialConfigurationId,
    ) -> Option<&CredentialConfiguration<K, L>> {
        self.configs_by_id.get(config_id)
    }

    pub fn get_by_format_and_attestation_type(
        &self,
        format: Format,
        attestation_type: &str,
    ) -> Option<(&CredentialConfigurationId, &CredentialConfiguration<K, L>)> {
        self.ids_by_format_and_attestation_type
            .get(&(format, Cow::Borrowed(attestation_type)))
            .and_then(|id| self.configs_by_id.get_key_value(id))
    }

    pub fn to_credential_configurations_supported(
        &self,
    ) -> HashMap<CredentialConfigurationId, issuer_metadata::CredentialConfiguration> {
        self.configs_by_id
            .iter()
            .map(|(config_id, config)| {
                // TODO (PVW-5548): Add "attestation" proof type.
                let proof_types = vec![ProofType::Jwt];
                let display = config.metadata.normalized.display().to_vec();
                let claims = config.metadata.normalized.claims().to_vec();

                let credential_configuration = match config.format {
                    Format::MsoMdoc => issuer_metadata::CredentialConfiguration::new_mdoc_ecdsa_p256_sha256(
                        config.attestation_type.clone(),
                        proof_types,
                        display,
                        claims,
                    ),
                    Format::SdJwt => issuer_metadata::CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                        config.attestation_type.clone(),
                        proof_types,
                        display,
                        claims,
                    ),
                    // Unsupported formats are filtered out in this type's constructor.
                    _ => unreachable!(),
                };

                (config_id.clone(), credential_configuration)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::qualification::AttestationQualification;
    use chrono::Days;
    use crypto::server_keys::generate::Ca;
    use p256::ecdsa::SigningKey;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use token_status_list::status_list_service::mock::MockStatusListService;

    use super::CredentialConfigurationParameters;
    use super::CredentialConfigurations;
    use super::CredentialConfigurationsError;
    use crate::Format;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::CredentialFormat;
    use crate::metadata::issuer_metadata::ProofType;

    fn credential_configuration_parameters()
    -> HashMap<CredentialConfigurationId, CredentialConfigurationParameters<SigningKey, MockStatusListService>> {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        [Format::MsoMdoc, Format::SdJwt]
            .into_iter()
            .map(|format| {
                let id = format!("degree_{format}").into();

                let key_pair = generate_issuer_mock_with_registration(&ca, IssuerRegistration::new_mock()).unwrap();
                let (_, metadata_documents) = TypeMetadataDocuments::degree_example();

                let params = CredentialConfigurationParameters {
                    format,
                    attestation_type: "com.example.degree".to_string(),
                    key_pair,
                    status_list: MockStatusListService::new(),
                    valid_days: Days::new(1),
                    issuer_uri: "https://example.com".parse().unwrap(),
                    attestation_qualification: AttestationQualification::default(),
                    metadata_documents,
                };

                (id, params)
            })
            .collect()
    }

    #[test]
    fn test_credential_configurations() {
        let params = credential_configuration_parameters();

        let configs = CredentialConfigurations::try_new(params)
            .expect("creating credential configurations from parameters should succeed");

        let config = configs
            .get_by_configuration_id(&"degree_mso_mdoc".to_string().into())
            .expect("configuration should exist");
        assert_eq!(config.format, Format::MsoMdoc);

        let config = configs
            .get_by_configuration_id(&"degree_dc+sd-jwt".to_string().into())
            .expect("configuration should exist");
        assert_eq!(config.format, Format::SdJwt);

        let (id, config) = configs
            .get_by_format_and_attestation_type(Format::MsoMdoc, "com.example.degree")
            .expect("configuration should exist");
        assert_eq!(id.as_ref(), "degree_mso_mdoc");
        assert_eq!(config.format, Format::MsoMdoc);

        let (id, config) = configs
            .get_by_format_and_attestation_type(Format::SdJwt, "com.example.degree")
            .expect("configuration should exist");
        assert_eq!(id.as_ref(), "degree_dc+sd-jwt");
        assert_eq!(config.format, Format::SdJwt);

        let metadata_configs = configs.to_credential_configurations_supported();
        assert_eq!(metadata_configs.len(), 2);

        assert_matches!(
            &metadata_configs
                .get(&"degree_mso_mdoc".to_string().into())
                .expect("metadata configuration should exist")
                .format,
            CredentialFormat::MsoMdoc { doctype, .. } if doctype == "com.example.degree"
        );

        assert_matches!(
            &metadata_configs
                .get(&"degree_dc+sd-jwt".to_string().into())
                .expect("metadata configuration should exist")
                .format,
            CredentialFormat::SdJwt { vct, .. } if vct == "com.example.degree"
        );

        let (_, metadata_docs) = TypeMetadataDocuments::degree_example();
        let (metadata, _) = metadata_docs.into_normalized("com.example.degree").unwrap();

        for metadata_config in metadata_configs.values() {
            let proof_types = metadata_config
                .cryptographic_binding
                .as_ref()
                .expect("cryptographic binding should be present")
                .proof_types_supported
                .keys()
                .cloned()
                .collect::<HashSet<_>>();
            assert_eq!(proof_types, HashSet::from([ProofType::Jwt]));

            let credential_metadata = metadata_config
                .credential_metadata
                .as_ref()
                .expect("credential metadata should be present");

            assert_eq!(
                credential_metadata
                    .display
                    .as_ref()
                    .map(|display| display.len().get())
                    .unwrap_or_default(),
                metadata.display().len()
            );
            assert_eq!(
                credential_metadata
                    .claims
                    .as_ref()
                    .map(|claims| claims.len().get())
                    .unwrap_or_default(),
                metadata.claims().len()
            );
        }
    }

    #[test]
    fn test_credential_configurations_try_new_error_no_configurations() {
        let params = HashMap::<
            CredentialConfigurationId,
            CredentialConfigurationParameters<SigningKey, MockStatusListService>,
        >::new();
        let error = CredentialConfigurations::try_new(params)
            .expect_err("creating credential configurations from parameters should fail");

        assert_matches!(error, CredentialConfigurationsError::NoConfigurations);
    }

    #[test]
    fn test_credential_configurations_try_new_error_type_metadata() {
        let mut params = credential_configuration_parameters();
        for params in params.values_mut() {
            params.attestation_type = "foobar".to_string();
        }

        let error = CredentialConfigurations::try_new(params)
            .expect_err("creating credential configurations from parameters should fail");

        assert_matches!(error, CredentialConfigurationsError::TypeMetadata(_));
    }

    #[test]
    fn test_credential_configurations_try_new_error_unsupported_format() {
        let mut params = credential_configuration_parameters();
        for params in params.values_mut() {
            params.format = Format::AcVc;
        }

        let error = CredentialConfigurations::try_new(params)
            .expect_err("creating credential configurations from parameters should fail");

        assert_matches!(error, CredentialConfigurationsError::UnsupportedFormat(Format::AcVc));
    }

    #[test]
    fn test_credential_configurations_try_new_error_duplicate_format_and_attestation_type() {
        let mut params = credential_configuration_parameters();
        for params in params.values_mut() {
            params.format = Format::SdJwt;
        }

        let error = CredentialConfigurations::try_new(params)
            .expect_err("creating credential configurations from parameters should fail");

        let duplicate_configs = HashMap::from([(
            (Format::SdJwt, "com.example.degree".to_string()),
            HashSet::from([
                "degree_mso_mdoc".to_string().into(),
                "degree_dc+sd-jwt".to_string().into(),
            ]),
        )]);
        assert_matches!(
            error,
            CredentialConfigurationsError::DuplicateFormatAndAttestationType(duplicates)
                if duplicates == duplicate_configs
        );
    }
}
