use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::sync::Arc;

use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use chrono::Days;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use derive_more::AsRef;
use derive_more::Debug;
use derive_more::From;
use derive_more::IntoIterator;
use futures::future::try_join_all;
use health_checkers::postgres::DatabaseChecker;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use http_utils::urls::BaseUrl;
use itertools::Itertools;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::scope::Scope;
use oauth::scope::ScopeInvalid;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::credential_configurations::CredentialConfiguration;
use openid4vc::credential_configurations::CredentialConfigurationFormat;
use openid4vc::credential_configurations::CredentialConfigurationTypeMetadata;
use openid4vc::credential_configurations::CredentialConfigurationsError;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use sd_jwt_vc_metadata::TypeMetadataChainError;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_with::TryFromInto;
use serde_with::TryFromIntoRef;
use serde_with::serde_as;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::Settings;
use server_utils::settings::verify_key_pairs;
use server_utils::store::SessionStoreVariant;
use server_utils::store::StoreConnection;
use server_utils::store::StoreError;
use server_utils::store::postgres::new_connection;
use status_lists::postgres::NoRevokeAll;
use status_lists::postgres::PostgresStatusListService;
use status_lists::postgres::StatusListServiceError;
use status_lists::publish::PublishDir;
use status_lists::settings::ExpiryLessThanTtl;
use status_lists::settings::StatusListsSettings;
use url::Url;
use utils::generator::TimeGenerator;
use utils::path::prefix_local_path;
use utils::vec_at_least::VecNonEmpty;

use crate::nonce_store::ProofNonceStore;
use crate::par_store::IssuerParStore;

/// Settings for an authorizing (Authorization Phase) issuer: the shared [`IssuerSettings`] plus the
/// parameters that only the auth-code path needs.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizingIssuerSettings {
    /// Exact-match allowlist of `redirect_uri` values the wallet may use in a Pushed Authorization
    /// Request. Validated by [`AuthorizingIssuer`] at `/par`.
    pub wallet_redirect_uris: VecNonEmpty<Url>,

    #[serde(flatten)]
    pub issuer_settings: IssuerSettings,
}

impl AuthorizingIssuerSettings {
    /// Build an [`AuthorizingIssuer`] (auth-code Authorization Phase) wrapping the inner [`Issuer`]
    /// produced by [`IssuerSettings::into_issuer`], plus the PAR store and an [`AuthorizationCodeFlow`]
    /// implementation. The `flow` closure receives the same [`StoreConnection`] used for sessions
    /// + PAR so the impl can construct its own stores.
    pub async fn into_authorizing_issuer<AF, E>(
        self,
        hsm: Option<Pkcs11Hsm>,
        flow: impl FnOnce(StoreConnection) -> Result<AF, E>,
    ) -> Result<
        (
            AuthorizingIssuer<
                PrivateKeyVariant,
                PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
                SessionStoreVariant<IssuanceData>,
                ProofNonceStore,
                IssuerParStore,
                AF,
            >,
            Vec<DatabaseChecker>,
            StoreConnection,
            Settings,
        ),
        IssuerSettingsError,
    >
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let Self {
            wallet_redirect_uris,
            issuer_settings,
        } = self;

        let (issuer, database_checkers, store_connection, server_settings) = issuer_settings.into_issuer(hsm).await?;

        let par_store = IssuerParStore::new(store_connection.clone());
        let flow =
            flow(store_connection.clone()).map_err(|e| IssuerSettingsError::AuthorizationCodeFlow(Box::new(e)))?;

        let authorizing_issuer = AuthorizingIssuer::new(Arc::new(issuer), par_store, flow, wallet_redirect_uris);

        Ok((authorizing_issuer, database_checkers, store_connection, server_settings))
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct IssuerSettings {
    /// Publicly reachable URL used by the wallet during sessions, which should be a valid Credential Issuer
    /// Identifier.
    pub public_url: IssuerIdentifier,

    /// Parsed from the `credential_configurations` and `type_metadata` settings together.
    #[serde(flatten)]
    pub credential_configurations: ParsedCredentialConfigurationsSettings,

    #[debug(skip)]
    pub credential_metadata_keypair: KeyPair,

    /// Base64url-encoded WRPRC bound to the credential metadata signing WRPAC.
    #[debug(skip)]
    #[serde_as(as = "TryFromIntoRef<String>")]
    pub registration_certificate: RegistrationCertificateEnvelope,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: HashSet<String>,

    /// The maximum amount of copies of a credential that the holder can request.
    pub batch_size: NonZeroU8,

    #[serde(flatten)]
    #[debug(skip)]
    pub server_settings: Settings,

    pub status_lists: StatusListsSettings,

    /// Trust anchors for verifying the wallet attestation (Wallet Instance Attestation).
    #[debug(skip)]
    pub wia_trust_anchors: TrustAnchors,
}

#[derive(Debug, Clone, Default, AsRef)]
pub struct TypeMetadataByVct(HashMap<String, JsonFile<UncheckedTypeMetadata>>);

/// The credential configurations of an issuer, each of which is guaranteed to be described by exactly one kind of
/// metadata that is appropriate for its format.
#[derive(Debug, Clone, Deserialize, From, IntoIterator, AsRef)]
#[serde(try_from = "CredentialConfigurationsSettings")]
pub struct ParsedCredentialConfigurationsSettings(
    #[into_iterator(owned, ref)] HashMap<CredentialConfigurationId, ParsedCredentialConfigurationSettings>,
);

#[derive(Debug, Clone)]
pub struct ParsedCredentialConfigurationSettings {
    pub format: CredentialConfigurationFormat,

    #[debug(skip)]
    pub keypair: KeyPair,

    pub valid_days: u64,

    pub status_list: StatusListAttestationSettings,
}

/// The credential configurations as they appear in the settings. The SD-JWT VC Type Metadata documents are configured
/// separately from the credential configurations, so these can only be matched to each other in a second pass.
#[serde_as]
#[derive(Deserialize)]
struct CredentialConfigurationsSettings {
    credential_configurations: HashMap<CredentialConfigurationId, CredentialConfigurationSettings>,

    /// Type metadata is optional for mdocs.
    #[serde(default)]
    #[serde_as(as = "Option<TryFromInto<Vec<String>>>")]
    type_metadata: Option<TypeMetadataByVct>,
}

#[serde_as]
#[derive(Deserialize)]
struct CredentialConfigurationSettings {
    #[serde(flatten)]
    format: CredentialFormatSettings,

    #[serde(flatten)]
    keypair: KeyPair,

    valid_days: u64,

    status_list: StatusListAttestationSettings,
}

/// The credential format of a credential configuration as it appears in the settings, together with the settings that
/// depend on it.
#[serde_as]
#[derive(Deserialize)]
#[serde(tag = "format")]
enum CredentialFormatSettings {
    #[serde(rename = "mso_mdoc")]
    MsoMdoc {
        attestation_type: String,

        /// Path to the JSON file with the Credential Metadata published for this credential configuration.
        #[serde_as(as = "TryFromInto<String>")]
        credential_metadata: JsonFile<CredentialMetadata>,
    },
    #[serde(rename = "dc+sd-jwt")]
    SdJwt {
        attestation_type: String,

        /// Path to the JSON file with the Credential Metadata published for this credential configuration. Not to
        /// be combined with SD-JWT VC Type Metadata for the same attestation type.
        #[serde_as(as = "Option<TryFromInto<String>>")]
        credential_metadata: Option<JsonFile<CredentialMetadata>>,
    },
}

impl TryFrom<CredentialConfigurationsSettings> for ParsedCredentialConfigurationsSettings {
    type Error = CredentialConfigurationFormatError;

    fn try_from(value: CredentialConfigurationsSettings) -> Result<Self, Self::Error> {
        let CredentialConfigurationsSettings {
            credential_configurations,
            type_metadata,
        } = value;

        let configurations = credential_configurations
            .into_iter()
            .map(|(config_id, settings)| {
                let format =
                    resolve_credential_configuration_format(&config_id, settings.format, type_metadata.as_ref())?;

                let settings = ParsedCredentialConfigurationSettings {
                    format,
                    keypair: settings.keypair,
                    valid_days: settings.valid_days,
                    status_list: settings.status_list,
                };

                Ok((config_id, settings))
            })
            .try_collect()?;

        Ok(Self(configurations))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsonFileParseError {
    #[error("could not read \"{0}\": {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("could not deserialize \"{0}\": {1}")]
    Deserialize(PathBuf, #[source] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonFile<T> {
    contents: T,
    json: Vec<u8>,
}

impl<T> JsonFile<T> {
    pub fn contents(&self) -> &T {
        &self.contents
    }

    pub fn into_contents(self) -> T {
        self.contents
    }

    pub fn json(&self) -> &[u8] {
        &self.json
    }
}

impl<T> TryFrom<String> for JsonFile<T>
where
    T: DeserializeOwned,
{
    type Error = JsonFileParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let path = prefix_local_path(PathBuf::from(value));
        let json = fs::read(&path).map_err(|error| JsonFileParseError::Read(path.clone().into_owned(), error))?;
        let contents =
            serde_json::from_slice(&json).map_err(|error| JsonFileParseError::Deserialize(path.into_owned(), error))?;

        Ok(Self { contents, json })
    }
}

impl TryFrom<Vec<String>> for TypeMetadataByVct {
    type Error = JsonFileParseError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        // Map the contents of each JSON file by the `vct` field by decoding the JSON and extracting just that field.
        let documents = value
            .into_iter()
            .map(|path| {
                let document = JsonFile::<UncheckedTypeMetadata>::try_from(path)?;

                let vct = document.contents().vct.clone();

                Ok((vct, document))
            })
            .try_collect()?;

        Ok(Self(documents))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataDocumentsError {
    #[error("maximum chain length exceeded")]
    MaximumLengthExceeded,

    #[error("missing metadata document for vct: {0}")]
    MissingDocument(String),
}

impl TypeMetadataByVct {
    /// Collect a chain of SD-JWT VC type metadata JSON from the configured files.
    fn to_metadata_documents(&self, vct: &str) -> Result<TypeMetadataDocuments, TypeMetadataDocumentsError> {
        const MAX_CHAIN_LENGTH: usize = 100;

        let Self(metadata_by_vct) = self;

        let mut documents = Vec::with_capacity(1);
        let mut chain_length = 0;
        let mut next_vct = Some(vct);

        while let Some(vct) = next_vct {
            chain_length += 1;
            if chain_length == MAX_CHAIN_LENGTH {
                return Err(TypeMetadataDocumentsError::MaximumLengthExceeded);
            }

            let document = metadata_by_vct
                .get(vct)
                .ok_or_else(|| TypeMetadataDocumentsError::MissingDocument(vct.to_string()))?;

            documents.push(document.json().to_vec());

            next_vct = document
                .contents()
                .extends
                .as_ref()
                .map(|extends| extends.extends.as_str());
        }

        // This `.unwrap()` is guaranteed to succeed as the `while` loop above runs at least once.
        let metadata_documents = TypeMetadataDocuments::new(documents.try_into().unwrap());

        Ok(metadata_documents)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialConfigurationFormatError {
    #[error("could not compile SD-JWT VC Type Metadata chain: {0}")]
    TypeMetadataChain(#[source] TypeMetadataDocumentsError),

    #[error("could not verify SD-JWT VC Type Metadata chain for credential configuration \"{0}\": {1}")]
    TypeMetadataVerification(CredentialConfigurationId, #[source] TypeMetadataChainError),

    #[error(
        "both SD-JWT VC Type Metadata and Credential Metadata are configured for SD-JWT credential configuration \
         \"{0}\", expected exactly one"
    )]
    DuplicateSdJwtMetadata(CredentialConfigurationId),

    #[error("no metadata is configured for SD-JWT credential configuration \"{0}\"")]
    MissingSdJwtMetadata(CredentialConfigurationId),
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialConfigurationsSettingsError {
    #[error("invalid private key: {0}")]
    PrivateKey(#[source] PrivateKeySettingsError),

    #[error("could not initialize status: {0}")]
    StatusList(#[source] StatusListAttestationSettingsError),

    #[error("could not use credential configuration ID as scope: {0}")]
    Scope(#[source] ScopeInvalid),
}

/// Determine the format of a single credential configuration, including the metadata that describes it.
fn resolve_credential_configuration_format(
    config_id: &CredentialConfigurationId,
    format: CredentialFormatSettings,
    metadata_by_vct: Option<&TypeMetadataByVct>,
) -> Result<CredentialConfigurationFormat, CredentialConfigurationFormatError> {
    match format {
        CredentialFormatSettings::MsoMdoc {
            attestation_type,
            credential_metadata,
        } => Ok(CredentialConfigurationFormat::new_mdoc(
            attestation_type,
            credential_metadata.into_contents(),
        )),
        CredentialFormatSettings::SdJwt {
            attestation_type,
            credential_metadata,
        } => {
            let type_metadata =
                metadata_by_vct.filter(|metadata| metadata.as_ref().contains_key(attestation_type.as_str()));

            match (type_metadata, credential_metadata) {
                (Some(_), Some(_)) => Err(CredentialConfigurationFormatError::DuplicateSdJwtMetadata(
                    config_id.clone(),
                )),
                (None, None) => Err(CredentialConfigurationFormatError::MissingSdJwtMetadata(
                    config_id.clone(),
                )),
                (Some(metadata_by_vct), None) => {
                    let documents = metadata_by_vct
                        .to_metadata_documents(&attestation_type)
                        .map_err(CredentialConfigurationFormatError::TypeMetadataChain)?;
                    let type_metadata = CredentialConfigurationTypeMetadata::try_new(&attestation_type, documents)
                        .map_err(|error| {
                            CredentialConfigurationFormatError::TypeMetadataVerification(config_id.clone(), error)
                        })?;

                    Ok(CredentialConfigurationFormat::new_sd_jwt_type_metadata(type_metadata))
                }
                (None, Some(credential_metadata)) => Ok(CredentialConfigurationFormat::new_sd_jwt_credential_metadata(
                    attestation_type,
                    credential_metadata.into_contents(),
                )),
            }
        }
    }
}

impl ParsedCredentialConfigurationsSettings {
    pub async fn into_credential_configurations(
        self,
        status_list_connection: DatabaseConnection,
        public_url: BaseUrl,
        hsm: Option<Pkcs11Hsm>,
        status_list_settings: &StatusListsSettings,
    ) -> Result<
        HashMap<
            CredentialConfigurationId,
            CredentialConfiguration<PrivateKeyVariant, PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>>,
        >,
        CredentialConfigurationsSettingsError,
    > {
        let Self(inner) = self;

        let config_count = inner.len();
        let credential_configs = try_join_all(
            inner
                .into_iter()
                .zip_eq(std::iter::repeat_n(
                    (status_list_connection, public_url, hsm),
                    config_count,
                ))
                .map(
                    |((config_id, settings), (status_list_connection, public_url, hsm))| async move {
                        let key_pair = settings
                            .keypair
                            .parse(hsm.clone())
                            .await
                            .map_err(CredentialConfigurationsSettingsError::PrivateKey)?;

                        let status_list = settings
                            .status_list
                            .into_service(status_list_connection, public_url, hsm, status_list_settings)
                            .await
                            .map_err(CredentialConfigurationsSettingsError::StatusList)?;

                        // Use the Credential Configuration ID as the scope value.
                        let scope =
                            Scope::try_new(config_id.as_ref()).map_err(CredentialConfigurationsSettingsError::Scope)?;

                        let config = CredentialConfiguration {
                            scope,
                            format: settings.format,
                            key_pair,
                            status_list,
                            valid_days: Days::new(settings.valid_days),
                        };

                        Ok::<_, CredentialConfigurationsSettingsError>((config_id, config))
                    },
                ),
        )
        .await?
        .into_iter()
        .collect();

        Ok(credential_configs)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerSettingsValidationError {
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
    #[error("error verifying certificate: {0}")]
    CertificateVerification(#[from] CertificateVerificationError),
    #[error(
        "attestation and status list certificate subject are different {config_id}: `{attestation}` vs `{status_list}`"
    )]
    CertificatesSubjectNameMismatch {
        config_id: CredentialConfigurationId,
        attestation: CanonicalDistinguishedName,
        status_list: CanonicalDistinguishedName,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerSettingsError {
    #[error("could not initialize HSM: {0}")]
    Hsm(#[source] HsmError),

    #[error("invalid metadata private key: {0}")]
    MetadataPrivateKey(#[source] PrivateKeySettingsError),

    #[error("could not initialize credential configurations from settings: {0}")]
    CredentialConfigurationsSettings(#[source] CredentialConfigurationsSettingsError),

    #[error("could not initialize storage: {0}")]
    Storage(#[source] StoreError),

    #[error("could not connect to status list database: {0}")]
    StatusListDatabase(#[source] DbErr),

    #[error("could not parse status list settings: {0}")]
    StatusListSettings(#[source] StatusListAttestationSettingsError),

    #[error("could not initialize status list service: {0}")]
    StatusLists(#[source] StatusListServiceError),

    #[error("no database configured for status lists")]
    NoStatusListDatabase,

    #[error("could not initialize credential configurations: {0}")]
    CredentialConfigurations(#[source] CredentialConfigurationsError),

    #[error("could not initialize authorization code flow: {0}")]
    AuthorizationCodeFlow(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl IssuerSettings {
    pub fn validate(&self) -> Result<(), IssuerSettingsValidationError> {
        tracing::debug!("verifying issuer settings");

        let time = TimeGenerator;

        verify_key_pairs(
            &[("credential_metadata", &self.credential_metadata_keypair)],
            &self.server_settings.wrpac_trust_anchors,
            None,
            &time,
        )?;

        let trust_anchors = &self.server_settings.issuer_trust_anchors;

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .credential_configurations
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.as_ref(), &attestation.keypair))
            .collect();

        verify_key_pairs(&key_pairs, trust_anchors, Some(CertificateUsage::Mdl), &time)?;

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .credential_configurations
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.as_ref(), &attestation.status_list.keypair))
            .collect();

        verify_key_pairs(
            &key_pairs,
            trust_anchors,
            Some(CertificateUsage::StatusListSigning),
            &time,
        )?;

        for (config_id, attestation) in self.credential_configurations.as_ref() {
            let attestation_dn = attestation.keypair.certificate.to_canonical_distinguished_name()?;
            let status_list_dn = attestation
                .status_list
                .keypair
                .certificate
                .to_canonical_distinguished_name()?;
            if attestation_dn != status_list_dn {
                return Err(IssuerSettingsValidationError::CertificatesSubjectNameMismatch {
                    config_id: config_id.clone(),
                    attestation: attestation_dn,
                    status_list: status_list_dn,
                });
            }
        }

        Ok(())
    }

    pub async fn into_issuer(
        self,
        hsm: Option<Pkcs11Hsm>,
    ) -> Result<
        (
            Issuer<
                PrivateKeyVariant,
                PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
                SessionStoreVariant<IssuanceData>,
                ProofNonceStore,
            >,
            Vec<DatabaseChecker>,
            StoreConnection,
            Settings,
        ),
        IssuerSettingsError,
    > {
        let mut database_checkers = Vec::with_capacity(1);

        let store_connection = StoreConnection::try_new(self.server_settings.storage.url.clone())
            .await
            .map_err(IssuerSettingsError::Storage)?;

        if let StoreConnection::Postgres(connection) = &store_connection {
            let name = if self.status_lists.storage_url.is_some() {
                "db-stores"
            } else {
                "db"
            };

            database_checkers.push(DatabaseChecker::new(name, connection));
        }

        let sessions = SessionStoreVariant::new(store_connection.clone(), (&self.server_settings.storage).into());
        let proof_nonce_store = ProofNonceStore::new(store_connection.clone());

        let status_list_connection = match (&store_connection, self.status_lists.storage_url.clone()) {
            (_, Some(url)) => {
                let connection = new_connection(url)
                    .await
                    .map_err(IssuerSettingsError::StatusListDatabase)?;
                database_checkers.push(DatabaseChecker::new("db-status-list", &connection));

                connection
            }
            (StoreConnection::Postgres(connection), None) => connection.clone(),
            _ => {
                return Err(IssuerSettingsError::NoStatusListDatabase);
            }
        };

        let metadata_keypair = self
            .credential_metadata_keypair
            .parse(hsm.clone())
            .await
            .map_err(IssuerSettingsError::MetadataPrivateKey)?;

        let credential_configs = self
            .credential_configurations
            .into_credential_configurations(
                status_list_connection,
                self.public_url.as_base_url().clone(),
                hsm,
                &self.status_lists,
            )
            .await
            .map_err(IssuerSettingsError::CredentialConfigurationsSettings)?;

        let issuer = Issuer::try_new(
            self.public_url,
            metadata_keypair,
            self.registration_certificate,
            self.batch_size,
            self.wallet_client_ids,
            credential_configs,
            self.wia_trust_anchors,
            Arc::new(sessions),
            proof_nonce_store,
        )
        .map_err(IssuerSettingsError::CredentialConfigurations)?;

        Ok((issuer, database_checkers, store_connection, self.server_settings))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StatusListAttestationSettingsError {
    #[error("incorrectly configured attestation status list expiration: {0}")]
    ExpiryLessThanTtl(#[source] ExpiryLessThanTtl),

    #[error("incorrectly configured attestation status list private key or certificate: {0}")]
    PrivateKey(#[source] PrivateKeySettingsError),

    #[error("could not initialize status list database: {0}")]
    Service(#[source] DbErr),
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusListAttestationSettings {
    /// The attestation group name, which should remain the same for the same credential over time.
    pub group_name: String,

    /// Base url for the status list if different from public url of the server
    pub base_url: Option<BaseUrl>,

    /// Context path for the status list joined with base_url, also used for serving
    pub context_path: String,

    /// Path to directory for the published status list
    pub publish_dir: PublishDir,

    /// Key pair to sign status list
    #[serde(flatten)]
    #[debug(skip)]
    pub keypair: KeyPair,
}

impl StatusListAttestationSettings {
    async fn into_service(
        self,
        connection: DatabaseConnection,
        public_url: BaseUrl,
        hsm: Option<Pkcs11Hsm>,
        status_list_settings: &StatusListsSettings,
    ) -> Result<PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>, StatusListAttestationSettingsError> {
        let base_url = self.base_url.unwrap_or(public_url);
        let key_pair = self
            .keypair
            .parse(hsm)
            .await
            .map_err(StatusListAttestationSettingsError::PrivateKey)?;

        let config = status_list_settings
            .to_config(base_url, self.context_path, self.publish_dir, key_pair)
            .map_err(StatusListAttestationSettingsError::ExpiryLessThanTtl)?;

        let service = PostgresStatusListService::try_new(&self.group_name, connection, config, NoRevokeAll)
            .await
            .map_err(StatusListAttestationSettingsError::Service)?;
        service
            .initialize_lists()
            .await
            .map_err(StatusListAttestationSettingsError::Service)?;

        Ok(service)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::num::NonZeroU8;
    use std::num::NonZeroU16;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
    use attestation_data::registration_certificate::mock::MockRegistrationCertificate;
    use attestation_data::x509::CertificateTypeError;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::BorrowingCertificate;
    use crypto::x509::CertificateConfiguration;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;
    use crypto::x509::DistinguishedName;
    use crypto::x509::SubjectAltNameUri;
    use openid4vc::credential_configurations::SdJwtMetadata;
    use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
    use sd_jwt_vc_metadata::TypeMetadataChainError;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use serde::Serialize;
    use serde_json::json;
    use serde_with::base64::Base64;
    use serde_with::serde_as;
    use server_utils::settings::CertificateVerificationError;
    use server_utils::settings::Server;
    use server_utils::settings::ServerAuth;
    use server_utils::settings::Settings;
    use server_utils::settings::Storage;
    use status_lists::publish::PublishDir;
    use status_lists::settings::StatusListsSettings;
    use utils::num::NonZeroU31;
    use utils::num::Ratio;

    use super::CredentialConfigurationFormat;
    use super::CredentialFormatSettings;
    use super::CredentialMetadata;
    use super::IssuerSettings;
    use super::JsonFile;
    use super::ParsedCredentialConfigurationSettings;
    use super::StatusListAttestationSettings;
    use super::TypeMetadataByVct;
    use crate::settings::CredentialConfigurationFormatError;
    use crate::settings::IssuerSettingsValidationError;
    use crate::settings::TypeMetadataDocumentsError;
    use crate::settings::resolve_credential_configuration_format;

    fn mock_settings(wrpac_ca: &Ca, issuer_ca: &Ca) -> IssuerSettings {
        let wrpac_keypair = wrpac_ca
            .generate_wrpac_issuer_mock()
            .expect("generate metadata cert failed");
        let registration_certificate = MockRegistrationCertificate::new_issuer(
            wrpac_keypair.certificate(),
            [CredentialKind::new(Format::SdJwt, "com.example.pid".to_string())],
        );

        let issuance_keypair = generate_issuer_mock_with_registration(issuer_ca, &IssuerRegistration::new_mock())
            .expect("generate issuer cert failed")
            .into();

        let status_list_keypair = issuer_ca
            .generate_issuer_status_list_mock()
            .expect("generate tsl cert failed")
            .into();

        // Normally this is its own CA; here we just reuse the issuer_ca.
        let wia_trust_anchors = TrustAnchors::from(issuer_ca);

        IssuerSettings {
            public_url: "https://example.com".parse().unwrap(),
            credential_configurations: HashMap::from([(
                "pid_sdjwt".to_string().into(),
                ParsedCredentialConfigurationSettings {
                    format: sd_jwt_format("com.example.pid"),
                    keypair: issuance_keypair,
                    valid_days: 365,
                    status_list: StatusListAttestationSettings {
                        group_name: "pid_sdjwt".to_string(),
                        base_url: None,
                        context_path: "tsl".to_string(),
                        keypair: status_list_keypair,
                        publish_dir: PublishDir::try_new(std::env::temp_dir()).unwrap(),
                    },
                },
            )])
            .into(),
            credential_metadata_keypair: wrpac_keypair.into(),
            registration_certificate: RegistrationCertificateEnvelope::try_from(
                registration_certificate.certificate.as_slice(),
            )
            .unwrap(),
            wallet_client_ids: HashSet::from([MOCK_WALLET_CLIENT_ID.to_string()]),
            batch_size: NonZeroU8::MIN,
            server_settings: Settings {
                wallet_server: Server {
                    ip: "127.0.0.1".parse().unwrap(),
                    port: 42,
                },
                internal_server: ServerAuth::InternalEndpoint(Server {
                    ip: "127.0.0.1".parse().unwrap(),
                    port: 43,
                }),
                log_requests: false,
                structured_logging: false,
                storage: Storage {
                    url: "memory://".parse().unwrap(),
                    expiration_minutes: 10.try_into().unwrap(),
                    successful_deletion_minutes: 10.try_into().unwrap(),
                    failed_deletion_minutes: 10.try_into().unwrap(),
                },
                issuer_trust_anchors: TrustAnchors::from(issuer_ca),
                wrpac_trust_anchors: TrustAnchors::from(wrpac_ca),
                wrprc_trust_anchors: TrustAnchors::empty(),
                hsm: None,
            },
            status_lists: StatusListsSettings {
                storage_url: None,
                list_size: NonZeroU31::try_new(100_000).unwrap(),
                create_threshold_ratio: Ratio::try_new(0.1).unwrap(),
                expiry_in_hours: NonZeroU16::new(24).unwrap(),
                refresh_threshold_ratio: Ratio::try_new(0.25).unwrap(),
                ttl_in_minutes: None,
                serve: true,
            },
            wia_trust_anchors,
        }
    }

    #[test]
    fn test_deserialize_registration_certificate() {
        #[serde_as]
        #[derive(Serialize)]
        struct Certificate<'a>(#[serde_as(as = "Base64")] &'a BorrowingCertificate);

        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let keypair = ca.generate_wrpac_issuer_mock().unwrap();
        let registration_certificate = MockRegistrationCertificate::new_issuer(
            keypair.certificate(),
            [CredentialKind::new(Format::SdJwt, "com.example.pid".to_string())],
        );
        let envelope =
            RegistrationCertificateEnvelope::try_from(registration_certificate.certificate.as_slice()).unwrap();
        let encoded = String::try_from(&envelope).unwrap();
        let mut settings = json!({
            "public_url": "https://example.com",
            "credential_configurations": {},
            "credential_metadata_keypair": {
                "certificate": Certificate(keypair.certificate()),
                "private_key_type": "hsm",
                "private_key": "test-metadata-key"
            },
            "registration_certificate": encoded,
            "type_metadata": [],
            "wallet_client_ids": [MOCK_WALLET_CLIENT_ID],
            "batch_size": 1,
            "wallet_server": { "ip": "127.0.0.1", "port": 42 },
            "internal_server": { "ip": "127.0.0.1", "port": 43 },
            "log_requests": false,
            "structured_logging": false,
            "storage": {
                "url": "memory://",
                "expiration_minutes": 10,
                "successful_deletion_minutes": 10,
                "failed_deletion_minutes": 10
            },
            "issuer_trust_anchors": [],
            "wrpac_trust_anchors": [],
            "wrprc_trust_anchors": [],
            "wia_trust_anchors": [],
            "status_lists": {
                "list_size": 100_000,
                "create_threshold_ratio": 0.1,
                "expiry_in_hours": 24,
                "refresh_threshold_ratio": 0.25
            }
        });

        let parsed: IssuerSettings = serde_json::from_value(settings.clone()).unwrap();
        assert_eq!(String::try_from(&parsed.registration_certificate).unwrap(), encoded);

        settings.as_object_mut().unwrap().remove("registration_certificate");
        let error = serde_json::from_value::<IssuerSettings>(settings.clone()).unwrap_err();
        assert!(error.to_string().contains("missing field `registration_certificate`"));

        for malformed in [json!(null), json!("not base64url!"), json!("e30")] {
            settings["registration_certificate"] = malformed;
            assert!(serde_json::from_value::<IssuerSettings>(settings.clone()).is_err());
        }
    }

    #[test]
    fn test_validate() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        mock_settings(&wrpac_ca, &issuer_ca).validate().unwrap();
    }

    #[test]
    fn test_no_wrpac_trust_anchors() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&wrpac_ca, &issuer_ca);

        settings.server_settings.wrpac_trust_anchors = TrustAnchors::empty();

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsValidationError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
        );
    }

    #[test]
    fn test_no_issuer_trust_anchors() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&wrpac_ca, &issuer_ca);

        settings.server_settings.issuer_trust_anchors = TrustAnchors::empty();

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsValidationError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
        );
    }

    #[test]
    fn test_no_issuer_registration() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&wrpac_ca, &issuer_ca);

        let issuer_cert_no_registration = issuer_ca
            .generate_issuer_mock()
            .expect("generate issuer cert without issuer registration");

        let status_list_keypair = issuer_ca
            .generate_issuer_status_list_mock()
            .expect("generate tsl cert failed")
            .into();

        settings.server_settings.issuer_trust_anchors = TrustAnchors::from(&issuer_ca);
        settings.credential_configurations = HashMap::from([(
            "no_registration_sdjwt".to_string().into(),
            ParsedCredentialConfigurationSettings {
                format: sd_jwt_format("com.example.no_registration"),
                keypair: issuer_cert_no_registration.into(),
                valid_days: 365,
                status_list: StatusListAttestationSettings {
                    group_name: "no_registration_sdjwt".to_string(),
                    base_url: None,
                    context_path: "tsl".to_string(),
                    keypair: status_list_keypair,
                    publish_dir: PublishDir::try_new(std::env::temp_dir()).unwrap(),
                },
            },
        )])
        .into();

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsValidationError::CertificateVerification(
                CertificateVerificationError::NoCertificateType(CertificateTypeError::IssuerRegistrationNotFound, key)
            ) if key == "no_registration_sdjwt"
        );
    }

    #[test]
    fn test_status_list_invalid_usage() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&wrpac_ca, &issuer_ca);

        let (typ, attestation_settings) = settings.credential_configurations.as_ref().iter().next().unwrap();
        let mut attestation_settings = attestation_settings.clone();
        attestation_settings.status_list.keypair = attestation_settings.keypair.clone();
        settings.credential_configurations = HashMap::from([(typ.clone(), attestation_settings)]).into();

        let error = settings.validate().expect_err("should fail");
        assert_matches!(
            error,
            IssuerSettingsValidationError::CertificateVerification(
                CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key)
            ) if key == "pid_sdjwt"
        );
    }

    #[test]
    fn test_different_subject_field() {
        let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate wrpac CA failed");
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&wrpac_ca, &issuer_ca);

        let status_list_keypair = issuer_ca
            .generate_key_pair(
                DistinguishedName::create_legal_person_mock("different"),
                CertificateConfiguration::with_usage(CertificateUsage::StatusListSigning),
                ["https://different.example.com/".parse::<SubjectAltNameUri>().unwrap()],
            )
            .expect("generate tsl cert failed");

        let (typ, attestation_settings) = settings.credential_configurations.as_ref().iter().next().unwrap();
        let mut attestation_settings = attestation_settings.clone();
        attestation_settings.status_list.keypair = status_list_keypair.into();
        settings.credential_configurations = HashMap::from([(typ.clone(), attestation_settings)]).into();

        let error = settings.validate().expect_err("should fail");
        assert_matches!(
            error,
            IssuerSettingsValidationError::CertificatesSubjectNameMismatch { config_id, attestation, status_list }
                if config_id.as_ref() == "pid_sdjwt" &&
                    attestation == "2.5.4.3=DAtDZXJ0IGlzc3Vlcg,2.5.4.6=DAJOTA,2.5.4.10=DBBDZXJ0IGlzc3VlciBCLlYu,1.3.6.1.1.15=DA5OVFJOTC05OTMzMzY3Mw".to_string().into() &&
                    status_list == "2.5.4.3=DAlkaWZmZXJlbnQ,2.5.4.6=DAJOTA,2.5.4.10=DA5kaWZmZXJlbnQgQi5WLg,1.3.6.1.1.15=DA5OVFJOTC0xOTU3MDE4Ng".to_string().into()
        );
    }

    /// Build a [`TypeMetadataByVct`] containing exactly the given metadata documents.
    fn type_metadata_by_vct(metadata: impl IntoIterator<Item = UncheckedTypeMetadata>) -> TypeMetadataByVct {
        TypeMetadataByVct(
            metadata
                .into_iter()
                .map(|metadata| {
                    let vct = metadata.vct.clone();
                    let json = serde_json::to_vec(&metadata).unwrap();

                    (
                        vct,
                        JsonFile {
                            contents: metadata,
                            json,
                        },
                    )
                })
                .collect(),
        )
    }

    fn credential_metadata() -> CredentialMetadata {
        CredentialMetadata::new_example(&["family_name"])
    }

    fn sd_jwt_format(vct: &str) -> CredentialConfigurationFormat {
        CredentialConfigurationFormat::new_sd_jwt_credential_metadata(vct.to_string(), credential_metadata())
    }

    fn credential_metadata_file() -> JsonFile<CredentialMetadata> {
        let contents = credential_metadata();
        let json = serde_json::to_vec(&contents).unwrap();

        JsonFile { contents, json }
    }

    fn sd_jwt_settings(vct: &str, with_credential_metadata: bool) -> CredentialFormatSettings {
        CredentialFormatSettings::SdJwt {
            attestation_type: vct.to_string(),
            credential_metadata: with_credential_metadata.then(credential_metadata_file),
        }
    }

    #[test]
    fn test_resolve_credential_configuration_format_mdoc_ignores_type_metadata() {
        let metadata = type_metadata_by_vct([UncheckedTypeMetadata::empty_example_with_attestation_type(
            "com.example.a",
        )]);
        let format = CredentialFormatSettings::MsoMdoc {
            attestation_type: "com.example.a".to_string(),
            credential_metadata: credential_metadata_file(),
        };

        let resolved = resolve_credential_configuration_format(&"cfg".to_string().into(), format, Some(&metadata))
            .expect("an mdoc sharing its attestation type with an SD-JWT should resolve");

        assert!(matches!(resolved, CredentialConfigurationFormat::MsoMdoc { .. }));
    }

    #[test]
    fn test_resolve_credential_configuration_format_is_per_credential_configuration() {
        let metadata = type_metadata_by_vct([UncheckedTypeMetadata::empty_example_with_attestation_type(
            "com.example.a",
        )]);

        let with_type_metadata = resolve_credential_configuration_format(
            &"cfg_a".to_string().into(),
            sd_jwt_settings("com.example.a", false),
            Some(&metadata),
        )
        .expect("a configuration with Type Metadata should resolve");
        assert_matches!(
            with_type_metadata,
            CredentialConfigurationFormat::SdJwt(SdJwtMetadata::TypeMetadata(type_metadata))
                if type_metadata.vct() == "com.example.a"
        );

        let falls_back = resolve_credential_configuration_format(
            &"cfg_b".to_string().into(),
            sd_jwt_settings("com.example.b", true),
            Some(&metadata),
        )
        .expect("a configuration without Type Metadata should fall back to its Credential Metadata");
        assert_matches!(
            falls_back,
            CredentialConfigurationFormat::SdJwt(SdJwtMetadata::CredentialMetadata { vct, .. })
                if vct == "com.example.b"
        );
    }

    #[test]
    fn test_resolve_credential_configuration_format_error_duplicate() {
        let metadata = type_metadata_by_vct([UncheckedTypeMetadata::empty_example_with_attestation_type(
            "com.example.a",
        )]);

        let Err(error) = resolve_credential_configuration_format(
            &"cfg_a".to_string().into(),
            sd_jwt_settings("com.example.a", true),
            Some(&metadata),
        ) else {
            panic!("configuring both kinds of metadata should not be allowed")
        };

        assert_matches!(
            error,
            CredentialConfigurationFormatError::DuplicateSdJwtMetadata(config_id) if config_id.as_ref() == "cfg_a"
        );
    }

    #[test]
    fn test_resolve_credential_configuration_format_error_missing() {
        let cases = [
            None,
            Some(type_metadata_by_vct([])),
            Some(type_metadata_by_vct([
                UncheckedTypeMetadata::empty_example_with_attestation_type("com.example.other"),
            ])),
        ];

        for metadata in cases {
            let Err(error) = resolve_credential_configuration_format(
                &"cfg_a".to_string().into(),
                sd_jwt_settings("com.example.a", false),
                metadata.as_ref(),
            ) else {
                panic!("configuring neither kind of metadata should not be allowed")
            };

            assert_matches!(
                error,
                CredentialConfigurationFormatError::MissingSdJwtMetadata(config_id) if config_id.as_ref() == "cfg_a"
            );
        }
    }

    #[test]
    fn test_resolve_credential_configuration_format_error_broken_chain() {
        let mut json = serde_json::to_value(UncheckedTypeMetadata::empty_example_with_attestation_type(
            "com.example.a",
        ))
        .unwrap();
        json["extends"] = serde_json::json!("com.example.absent");
        json["extends#integrity"] = serde_json::json!("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=");
        let extending: UncheckedTypeMetadata = serde_json::from_value(json).unwrap();

        let Err(error) = resolve_credential_configuration_format(
            &"cfg_a".to_string().into(),
            sd_jwt_settings("com.example.a", false),
            Some(&type_metadata_by_vct([extending])),
        ) else {
            panic!("a chain that extends an absent document should not resolve")
        };

        assert_matches!(
            error,
            CredentialConfigurationFormatError::TypeMetadataChain(TypeMetadataDocumentsError::MissingDocument(vct))
                if vct == "com.example.absent"
        );
    }

    #[test]
    fn test_resolve_credential_configuration_format_error_type_metadata_verification() {
        // The extended document is present, but its resource integrity does not match the one in the extending
        // document.
        let mut json = serde_json::to_value(UncheckedTypeMetadata::empty_example_with_attestation_type(
            "com.example.a",
        ))
        .unwrap();
        json["extends"] = serde_json::json!("com.example.b");
        json["extends#integrity"] = serde_json::json!("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=");
        let extending: UncheckedTypeMetadata = serde_json::from_value(json).unwrap();
        let extended = UncheckedTypeMetadata::empty_example_with_attestation_type("com.example.b");

        let Err(error) = resolve_credential_configuration_format(
            &"cfg_a".to_string().into(),
            sd_jwt_settings("com.example.a", false),
            Some(&type_metadata_by_vct([extending, extended])),
        ) else {
            panic!("a chain with a mismatching resource integrity should not resolve")
        };

        assert_matches!(
            error,
            CredentialConfigurationFormatError::TypeMetadataVerification(
                config_id,
                TypeMetadataChainError::ResourceIntegrity(_)
            ) if config_id.as_ref() == "cfg_a"
        );
    }
}
