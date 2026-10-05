use std::collections::HashMap;
use std::collections::HashSet;

use attestation_data::metadata::AttestationClaims;
use attestation_data::metadata::ClaimConstraint;
use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use chrono::Days;
use crypto::server_keys::KeyPair;
use derive_more::Debug;
use itertools::Either;
use itertools::Itertools;
use oauth::issuer_identifier::IssuerUrl;
use oauth::scope::Scope;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::SortedTypeMetadataDocuments;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use ssri::Integrity;
use utils::vec_at_least::VecNonEmptyUnique;

use crate::metadata::issuer_metadata;
use crate::metadata::issuer_metadata::CredentialConfigurationId;
use crate::metadata::issuer_metadata::CredentialMetadata;
use crate::metadata::issuer_metadata::JoinCredentialConfigurationId;
use crate::metadata::issuer_metadata::ProofType;
#[derive(Debug, thiserror::Error)]
pub enum CredentialConfigurationsError {
    #[error("no credential configurations provided")]
    NoConfigurations,

    #[error("scope \"{scope}\" does not match credential configuration ID \"{config_id}\"")]
    ScopeMismatch {
        config_id: CredentialConfigurationId,
        scope: Scope,
    },

    #[error(
        "multiple credential configurations for the same combination of format and attestation type: {}",
        .0
            .iter()
            .map(|(format_type, config_ids)| format!("{format_type} - {}", config_ids.iter().join(", ")))
            .join(" / ")
    )]
    DuplicateFormatAndAttestationType(HashMap<CredentialKind, HashSet<CredentialConfigurationId>>),
}

/// The format of a credential configuration and the metadata that describes it. An mdoc is only described by Credential
/// Metadata, while an SD-JWT is described by either SD-JWT VC Type Metadata or Credential Metadata.
///
/// The attestation type (`doc_type` for an mdoc, `vct` for an SD-JWT) is only stored where the metadata does not
/// already contain it. Credential Metadata carries no identifier, so it is paired with a `doc_type` or `vct`. SD-JWT VC
/// Type Metadata carries its own `vct`, so none is stored alongside it.
#[derive(Debug, Clone)]
pub enum CredentialConfigurationFormat {
    MsoMdoc {
        doc_type: String,
        #[debug(skip)]
        credential_metadata: CredentialMetadata,
    },
    SdJwt(SdJwtMetadata),
}

/// The metadata that describes an SD-JWT credential configuration, which also determines its `vct`.
#[derive(Debug, Clone)]
pub enum SdJwtMetadata {
    /// Described by SD-JWT VC Type Metadata, of which the leaf document determines the `vct`.
    TypeMetadata(CredentialConfigurationTypeMetadata),
    CredentialMetadata {
        vct: String,
        #[debug(skip)]
        credential_metadata: CredentialMetadata,
    },
}

impl AttestationClaims for SdJwtMetadata {
    fn claim_constraints(&self) -> impl Iterator<Item = ClaimConstraint<'_>> {
        match self {
            Self::TypeMetadata(metadata) => Either::Left(metadata.normalized.claim_constraints()),
            Self::CredentialMetadata {
                credential_metadata, ..
            } => Either::Right(credential_metadata.claim_constraints()),
        }
    }
}

impl AttestationClaims for CredentialConfigurationFormat {
    fn claim_constraints(&self) -> impl Iterator<Item = ClaimConstraint<'_>> {
        match self {
            Self::MsoMdoc {
                credential_metadata, ..
            } => Either::Left(credential_metadata.claim_constraints()),
            Self::SdJwt(metadata) => Either::Right(metadata.claim_constraints()),
        }
    }
}

impl CredentialConfigurationFormat {
    /// The credential format that corresponds to this configuration.
    pub fn format(&self) -> Format {
        match self {
            Self::MsoMdoc { .. } => Format::MsoMdoc,
            Self::SdJwt(_) => Format::SdJwt,
        }
    }

    /// The identifier that names the attestation in this configuration's format.
    pub fn attestation_type(&self) -> &str {
        match self {
            Self::MsoMdoc { doc_type, .. } => doc_type,
            Self::SdJwt(metadata) => metadata.vct(),
        }
    }

    /// The SD-JWT VC Type Metadata, if this configuration is described by it.
    pub fn type_metadata(&self) -> Option<&CredentialConfigurationTypeMetadata> {
        match self {
            Self::MsoMdoc { .. } | Self::SdJwt(SdJwtMetadata::CredentialMetadata { .. }) => None,
            Self::SdJwt(SdJwtMetadata::TypeMetadata(type_metadata)) => Some(type_metadata),
        }
    }
}

impl SdJwtMetadata {
    pub fn vct(&self) -> &str {
        match self {
            Self::TypeMetadata(type_metadata) => type_metadata.vct(),
            Self::CredentialMetadata { vct, .. } => vct,
        }
    }
}

/// A verified chain of SD-JWT VC Type Metadata documents for a particular `vct`, as used by a credential configuration.
#[derive(Debug, Clone)]
pub struct CredentialConfigurationTypeMetadata {
    #[debug(skip)]
    documents: SortedTypeMetadataDocuments,
    first_document_integrity: Integrity,
    normalized: NormalizedTypeMetadata,
}

impl CredentialConfigurationTypeMetadata {
    /// Verify the chain of Type Metadata documents, of which the leaf document should have the provided `vct`.
    pub fn try_new(vct: &str, documents: TypeMetadataDocuments) -> Result<Self, TypeMetadataChainError> {
        // Calculate and cache the integrity hash for the first metadata document in the chain.
        let first_document_integrity = Integrity::from(documents.as_ref().first().as_slice());
        let (normalized, documents) = documents.into_normalized(vct)?;

        let metadata = Self {
            documents,
            first_document_integrity,
            normalized,
        };

        Ok(metadata)
    }

    pub fn vct(&self) -> &str {
        self.normalized.vct()
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

/// Static attestation data shared across all instances of an attestation type for a particular format. Parts of this
/// configuration are represented in the `credential_configurations_supported` section of the issuer metadata.
///
/// When performing issuance, the issuer augments the [`CredentialConfiguration`] with an [`IssuableDocument`] to form
/// the attestation.
#[derive(Debug)]
pub struct CredentialConfiguration<K, L> {
    pub scope: Scope,
    #[debug(skip)]
    pub key_pair: KeyPair<K>,
    pub status_list: L,
    pub valid_days: Days,
    pub format: CredentialConfigurationFormat,
}

impl<K, L> CredentialConfiguration<K, L> {
    /// The combination of the attestation type and the format it is expressed in.
    pub(crate) fn credential_kind(&self) -> CredentialKind {
        CredentialKind::new(self.format.format(), self.format.attestation_type().to_string())
    }
}

/// Static credential configurations indexed by their identifier.
#[derive(Debug)]
pub(crate) struct CredentialConfigurations<K, L> {
    configs_by_id: HashMap<CredentialConfigurationId, CredentialConfiguration<K, L>>,
    ids_by_credential_kind: HashMap<CredentialKind, CredentialConfigurationId>,
}

impl<K, L> CredentialConfigurations<K, L> {
    /// Create the configurations. The scope of each configuration has to equal its Credential Configuration ID, which
    /// allows [`Self::get_by_scope()`] to look up a configuration by scope without a separate index.
    pub fn try_new(
        configs_by_id: HashMap<CredentialConfigurationId, CredentialConfiguration<K, L>>,
    ) -> Result<Self, CredentialConfigurationsError> {
        if configs_by_id.is_empty() {
            return Err(CredentialConfigurationsError::NoConfigurations);
        }

        let mut ids_by_credential_kind = HashMap::<_, Vec<_>>::new();

        for (config_id, config) in &configs_by_id {
            if config.scope.as_ref() != config_id.as_ref() {
                return Err(CredentialConfigurationsError::ScopeMismatch {
                    config_id: config_id.clone(),
                    scope: config.scope.clone(),
                });
            }

            ids_by_credential_kind
                .entry(config.credential_kind())
                .or_default()
                .push(config_id.clone());
        }

        let (ids_by_credential_kind, duplicate_credential_kind) = ids_by_credential_kind
            .into_iter()
            .partition_map::<_, HashMap<_, _>, _, _, _>(|(credential_kind, ids)| match ids.into_iter().exactly_one() {
                Ok(id) => Either::Left((credential_kind, id)),
                Err(ids) => Either::Right((credential_kind, ids.collect())),
            });

        if !duplicate_credential_kind.is_empty() {
            return Err(CredentialConfigurationsError::DuplicateFormatAndAttestationType(
                duplicate_credential_kind,
            ));
        }

        let credential_configurations = Self {
            configs_by_id,
            ids_by_credential_kind,
        };

        Ok(credential_configurations)
    }

    pub fn configurations(&self) -> impl Iterator<Item = &CredentialConfiguration<K, L>> {
        self.configs_by_id.values()
    }

    pub fn all_configuration_ids(&self) -> VecNonEmptyUnique<&CredentialConfigurationId> {
        self.configs_by_id
            .keys()
            .collect_vec()
            .try_into()
            .expect("a non-zero amount of credential configurations is guaranteed by this type's constructor")
    }

    pub fn get_by_configuration_id(
        &self,
        config_id: &CredentialConfigurationId,
    ) -> Option<&CredentialConfiguration<K, L>> {
        self.configs_by_id.get(config_id)
    }

    pub fn get_by_scope(&self, scope: &Scope) -> Option<(&CredentialConfigurationId, &CredentialConfiguration<K, L>)> {
        // The `CredentialConfigurations::try_new()` constructor (which is the only way to create
        // `CredentialConfigurations`) guarantees that the Credential Configuration ID is equal to the scope, so we can
        // use it as a shortcut.
        self.configs_by_id.get_key_value(scope.as_ref())
    }

    pub fn get_by_credential_kind(
        &self,
        credential_kind: &CredentialKind,
    ) -> Option<(&CredentialConfigurationId, &CredentialConfiguration<K, L>)> {
        self.ids_by_credential_kind
            .get(credential_kind)
            .and_then(|id| self.configs_by_id.get_key_value(id))
    }

    pub fn to_credential_configurations_supported(
        &self,
        type_metadata_base_url: &IssuerUrl,
    ) -> HashMap<CredentialConfigurationId, issuer_metadata::CredentialConfiguration> {
        self.configs_by_id
            .iter()
            .map(|(config_id, config)| {
                let scope = config.scope.clone();

                // TODO (PVW-5548): Add "attestation" proof type.
                let proof_types = vec![ProofType::Jwt];

                let credential_configuration = match &config.format {
                    CredentialConfigurationFormat::MsoMdoc {
                        doc_type,
                        credential_metadata,
                    } => issuer_metadata::CredentialConfiguration::new_mdoc_ecdsa_p256_sha256(
                        doc_type.clone(),
                        scope,
                        proof_types,
                        credential_metadata.clone(),
                    ),
                    // As the Type Metadata URI is not part of the OpenID4VCI specification, Credential Metadata derived
                    // from the Type Metadata is published as well.
                    CredentialConfigurationFormat::SdJwt(SdJwtMetadata::TypeMetadata(type_metadata)) => {
                        issuer_metadata::CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                            type_metadata.vct().to_string(),
                            scope,
                            proof_types,
                            CredentialMetadata::from(type_metadata.normalized()),
                            Some(type_metadata_base_url.join_config_id(config_id)),
                        )
                    }
                    CredentialConfigurationFormat::SdJwt(SdJwtMetadata::CredentialMetadata {
                        vct,
                        credential_metadata,
                    }) => issuer_metadata::CredentialConfiguration::new_sd_jwt_ecdsa_p256_sha256(
                        vct.clone(),
                        scope,
                        proof_types,
                        credential_metadata.clone(),
                        None,
                    ),
                };

                (config_id.clone(), credential_configuration)
            })
            .collect::<HashMap<_, _>>()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::collections::HashSet;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use chrono::Days;
    use crypto::server_keys::generate::Ca;
    use p256::ecdsa::SigningKey;
    use sd_jwt_vc_metadata::TypeMetadataChainError;
    use sd_jwt_vc_metadata::TypeMetadataDocuments;
    use token_status_list::status_list_service::mock::MockStatusListService;

    use super::CredentialConfiguration;
    use super::CredentialConfigurationFormat;
    use super::CredentialConfigurationTypeMetadata;
    use super::CredentialConfigurations;
    use super::CredentialConfigurationsError;
    use super::SdJwtMetadata;
    use crate::metadata::issuer_metadata::CredentialConfigurationId;
    use crate::metadata::issuer_metadata::CredentialFormat;
    use crate::metadata::issuer_metadata::CredentialMetadata;
    use crate::metadata::issuer_metadata::ProofType;

    fn degree_type_metadata() -> CredentialConfigurationTypeMetadata {
        let (_, degree_documents) = TypeMetadataDocuments::degree_example();

        CredentialConfigurationTypeMetadata::try_new("com.example.degree", degree_documents)
            .expect("example type metadata should verify")
    }

    fn credential_configurations_by_id()
    -> HashMap<CredentialConfigurationId, CredentialConfiguration<SigningKey, MockStatusListService>> {
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        [Format::MsoMdoc, Format::SdJwt]
            .into_iter()
            .map(|format| {
                let id = format!("degree_{format}");

                let key_pair = generate_issuer_mock_with_registration(&ca, &IssuerRegistration::new_mock()).unwrap();
                let format = match format {
                    Format::MsoMdoc => CredentialConfigurationFormat::MsoMdoc {
                        doc_type: "com.example.degree".to_string(),
                        credential_metadata: CredentialMetadata::new_mdoc_example(
                            "com.example.degree",
                            &["university", "education", "graduation_date", "grade", "cum_laude"],
                        ),
                    },
                    Format::SdJwt => {
                        CredentialConfigurationFormat::SdJwt(SdJwtMetadata::TypeMetadata(degree_type_metadata()))
                    }
                };

                let config = CredentialConfiguration {
                    scope: id.parse().unwrap(),
                    format,
                    key_pair,
                    status_list: MockStatusListService::new(),
                    valid_days: Days::new(1),
                };

                (id.into(), config)
            })
            .collect()
    }

    #[test]
    fn test_credential_configurations() {
        let configs = credential_configurations_by_id();

        let configs =
            CredentialConfigurations::try_new(configs).expect("creating credential configurations should succeed");

        let config = configs
            .get_by_configuration_id(&"degree_mso_mdoc".to_string().into())
            .expect("configuration should exist");
        assert_matches!(config.format, CredentialConfigurationFormat::MsoMdoc { .. });

        let config = configs
            .get_by_configuration_id(&"degree_dc+sd-jwt".to_string().into())
            .expect("configuration should exist");
        assert_matches!(config.format, CredentialConfigurationFormat::SdJwt { .. });

        let (id, config) = configs
            .get_by_scope(&"degree_mso_mdoc".parse().unwrap())
            .expect("configuration should exist");
        assert_eq!(*id, "degree_mso_mdoc".to_string().into());
        assert_matches!(config.format, CredentialConfigurationFormat::MsoMdoc { .. });

        let (id, config) = configs
            .get_by_scope(&"degree_dc+sd-jwt".parse().unwrap())
            .expect("configuration should exist");
        assert_eq!(*id, "degree_dc+sd-jwt".to_string().into());
        assert_matches!(config.format, CredentialConfigurationFormat::SdJwt { .. });

        let (id, config) = configs
            .get_by_credential_kind(&CredentialKind::new(Format::MsoMdoc, "com.example.degree".to_string()))
            .expect("configuration should exist");
        assert_eq!(id.as_ref(), "degree_mso_mdoc");
        assert_matches!(config.format, CredentialConfigurationFormat::MsoMdoc { .. });

        let (id, config) = configs
            .get_by_credential_kind(&CredentialKind::new(Format::SdJwt, "com.example.degree".to_string()))
            .expect("configuration should exist");
        assert_eq!(id.as_ref(), "degree_dc+sd-jwt");
        assert_matches!(config.format, CredentialConfigurationFormat::SdJwt { .. });

        let type_metadata_base_url = "https://example.com".parse().unwrap();
        let metadata_configs = configs.to_credential_configurations_supported(&type_metadata_base_url);
        assert_eq!(metadata_configs.len(), 2);

        assert_matches!(
            &metadata_configs
                .get("degree_mso_mdoc")
                .expect("metadata configuration should exist")
                .format,
            CredentialFormat::MsoMdoc { doctype, .. } if doctype == "com.example.degree"
        );

        assert_matches!(
            &metadata_configs
                .get("degree_dc+sd-jwt")
                .expect("metadata configuration should exist")
                .format,
            CredentialFormat::SdJwt { vct, .. } if vct == "com.example.degree"
        );

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
        }

        let mdoc_config = metadata_configs
            .get("degree_mso_mdoc")
            .expect("metadata configuration should exist");

        assert!(mdoc_config.credential_metadata.is_some());
        assert_matches!(mdoc_config.format, CredentialFormat::MsoMdoc { .. });

        let sd_jwt_config = metadata_configs
            .get("degree_dc+sd-jwt")
            .expect("metadata configuration should exist");

        assert_eq!(
            sd_jwt_config.credential_metadata,
            Some(CredentialMetadata::from(degree_type_metadata().normalized())),
            "an SD-JWT described by Type Metadata should also contain Credential Metadata derived from it"
        );
        assert_matches!(
            &sd_jwt_config.format,
            CredentialFormat::SdJwt {
                type_metadata_uri: Some(_),
                ..
            },
            "an SD-JWT described by Type Metadata should contain type_metadata_uri"
        );
    }

    #[test]
    fn test_credential_configurations_sd_jwt_described_by_credential_metadata() {
        let mut configs = credential_configurations_by_id();
        configs.get_mut("degree_dc+sd-jwt").unwrap().format =
            CredentialConfigurationFormat::SdJwt(SdJwtMetadata::CredentialMetadata {
                vct: "com.example.degree".to_string(),
                credential_metadata: CredentialMetadata::new_example(&["university", "education"]),
            });

        let configs = CredentialConfigurations::try_new(configs)
            .expect("an SD-JWT described by Credential Metadata should create credential configurations");

        let metadata_configs = configs.to_credential_configurations_supported(&"https://example.com".parse().unwrap());

        let sd_jwt_config = metadata_configs
            .get("degree_dc+sd-jwt")
            .expect("metadata configuration should exist");

        assert_matches!(
            &sd_jwt_config.format,
            CredentialFormat::SdJwt {
                vct,
                type_metadata_uri,
                ..
            } if vct == "com.example.degree" && type_metadata_uri.is_none()
        );
        assert!(sd_jwt_config.credential_metadata.is_some());
    }

    #[test]
    fn test_credential_configurations_try_new_error_no_configurations() {
        let configs =
            HashMap::<CredentialConfigurationId, CredentialConfiguration<SigningKey, MockStatusListService>>::new();
        let error =
            CredentialConfigurations::try_new(configs).expect_err("creating credential configurations should fail");

        assert_matches!(error, CredentialConfigurationsError::NoConfigurations);
    }

    #[test]
    fn test_credential_configuration_type_metadata_try_new() {
        let type_metadata = degree_type_metadata();

        assert_eq!(type_metadata.vct(), "com.example.degree");
    }

    #[test]
    fn test_credential_configuration_type_metadata_try_new_error_vct_not_found() {
        let (_, degree_documents) = TypeMetadataDocuments::degree_example();

        let error = CredentialConfigurationTypeMetadata::try_new("foobar", degree_documents)
            .expect_err("verifying type metadata for another vct should fail");

        assert_matches!(error, TypeMetadataChainError::VctNotFound(vct) if vct == "foobar");
    }

    #[test]
    fn test_credential_configurations_try_new_error_scope_mismatch() {
        let mut configs = credential_configurations_by_id();

        // Change one of the Credential Configuration IDs so that it no longer equals the scope.
        let config = configs.remove("degree_mso_mdoc").unwrap();
        configs.insert("other_id".to_string().into(), config);

        let error =
            CredentialConfigurations::try_new(configs).expect_err("creating credential configurations should fail");

        assert_matches!(
            error,
            CredentialConfigurationsError::ScopeMismatch { config_id, scope }
                if config_id.as_ref() == "other_id" && scope.as_ref() == "degree_mso_mdoc"
        );
    }

    #[test]
    fn test_credential_configurations_try_new_error_duplicate_credential_kind() {
        let mut configs = credential_configurations_by_id();
        for config in configs.values_mut() {
            config.format = CredentialConfigurationFormat::SdJwt(SdJwtMetadata::TypeMetadata(degree_type_metadata()));
        }

        let error =
            CredentialConfigurations::try_new(configs).expect_err("creating credential configurations should fail");

        let duplicate_configs = HashMap::from([(
            CredentialKind::new(Format::SdJwt, "com.example.degree".to_string()),
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
