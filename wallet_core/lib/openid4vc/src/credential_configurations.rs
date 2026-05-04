use std::borrow::Cow;
use std::collections::HashMap;

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
use crate::metadata::issuer_metadata::ProofType;

#[derive(Debug, thiserror::Error)]
pub enum CredentialConfigurationsError {
    #[error("could not parse SD-JWT VC Type Metadata chain: {0}")]
    TypeMetadata(#[source] TypeMetadataChainError),

    #[error("unsupported credential format: {0}")]
    UnsupportedFormat(Format),

    #[error(
        "multiple credential configurations for the same combination of format and attestation type: {}",
        .0.iter().map(|((format, attestation_type), configs)| configs.join(", ")).join(" / ")
    )]
    DuplicateFormatAndAttestationType(HashMap<(Format, String), Vec<String>>),
}

#[derive(Debug)]
pub struct CredentialConfigurationParameters<K> {
    pub format: Format,
    pub attestation_type: String,
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
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
pub struct CredentialConfiguration<K> {
    format: Format,
    attestation_type: String,
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub valid_days: Days,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
    pub metadata: CredentialConfigurationMetadata,
}

#[derive(Debug)]
pub struct CredentialConfigurationMetadata {
    #[debug(skip)]
    documents: SortedTypeMetadataDocuments,
    first_document_integrity: Integrity,
    normalized: NormalizedTypeMetadata,
}

impl<K> CredentialConfiguration<K> {
    /// Create a new [`CredentialConfiguration`] and decode and validate the type metadata documents.
    pub fn try_new(
        CredentialConfigurationParameters {
            format,
            attestation_type,
            key_pair,
            valid_days,
            issuer_uri,
            attestation_qualification,
            metadata_documents,
        }: CredentialConfigurationParameters<K>,
    ) -> Result<Self, TypeMetadataChainError> {
        let metadata = CredentialConfigurationMetadata::try_new(&attestation_type, metadata_documents)?;

        let config = Self {
            format,
            attestation_type,
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
pub struct CredentialConfigurations<K> {
    configs_by_id: HashMap<String, CredentialConfiguration<K>>,
    ids_by_format_and_attestation_type: HashMap<(Format, Cow<'static, str>), String>,
}

impl<K> CredentialConfigurations<K> {
    pub fn try_new(
        configurations: impl IntoIterator<Item = (String, CredentialConfigurationParameters<K>)>,
    ) -> Result<Self, CredentialConfigurationsError> {
        let mut ids_by_format_and_attestation_type = HashMap::<_, Vec<_>>::new();

        let configs_by_id = configurations
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
                        Err(ids) => Either::Right(((format, attestation_type), ids.collect_vec())),
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

    pub fn get_by_configuration_id(&self, config_id: &str) -> Option<&CredentialConfiguration<K>> {
        self.configs_by_id.get(config_id)
    }

    pub fn get_by_format_and_attestation_type(
        &self,
        format: Format,
        attestation_type: &str,
    ) -> Option<(&str, &CredentialConfiguration<K>)> {
        self.ids_by_format_and_attestation_type
            .get(&(format, Cow::Borrowed(attestation_type)))
            .and_then(|id| self.configs_by_id.get_key_value(id))
            .map(|(id, config)| (id.as_str(), config))
    }

    pub fn to_credential_configurations_supported(&self) -> HashMap<String, issuer_metadata::CredentialConfiguration> {
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
