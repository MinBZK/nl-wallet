use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::sync::Arc;

use attestation_types::credential_format::Format;
use attestation_types::credential_kind::CredentialKind;
use chrono::Days;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CanonicalDistinguishedName;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use derive_more::AsRef;
use derive_more::Debug;
use derive_more::From;
use derive_more::Into;
use derive_more::IntoIterator;
use futures::future::try_join_all;
use health_checkers::postgres::DatabaseChecker;
use hsm::service::HsmError;
use hsm::service::Pkcs11Hsm;
use http_utils::urls::BaseUrl;
use itertools::Itertools;
use oauth::issuer_identifier::IssuerIdentifier;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::credential_configurations::CredentialConfigurationParameters;
use openid4vc::credential_configurations::CredentialConfigurationsError;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use serde::Deserialize;
use serde_with::TryFromInto;
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

    pub credential_configurations: CredentialConfigurationsSettings,

    #[debug(skip)]
    pub credential_metadata_keypair: KeyPair,

    #[debug(skip)]
    #[serde_as(as = "TryFromInto<Vec<String>>")]
    pub type_metadata: TypeMetadataByVct,

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

#[derive(Debug, Clone, AsRef)]
pub struct TypeMetadataByVct(HashMap<String, (UncheckedTypeMetadata, Vec<u8>)>);

#[derive(Debug, Clone, Deserialize, From, IntoIterator, AsRef)]
pub struct CredentialConfigurationsSettings(
    #[into_iterator(owned, ref)] HashMap<CredentialConfigurationId, CredentialConfigurationSettings>,
);

#[derive(Debug, Clone, Deserialize)]
pub struct CredentialConfigurationSettings {
    #[serde(flatten)]
    pub credential_kind: CredentialKind,

    #[serde(flatten)]
    #[debug(skip)]
    pub keypair: KeyPair,

    pub valid_days: u64,

    pub status_list: StatusListAttestationSettings,

    /// Path to the JSON file with the Credential Metadata published for this credential configuration.
    #[debug(skip)]
    pub credential_metadata: Option<CredentialMetadataFile>,
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataParseError {
    #[error("could not read \"{0}\": {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("could not deserialize \"{0}\": {1}")]
    Deserialize(PathBuf, #[source] serde_json::Error),
}

impl TryFrom<Vec<String>> for TypeMetadataByVct {
    type Error = MetadataParseError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        // Map the contents of each JSON file by the `vct` field by decoding the JSON and extracting just that field.
        let documents = value
            .into_iter()
            .map(|path| {
                let path = prefix_local_path(PathBuf::from(path));
                let json =
                    fs::read(&path).map_err(|error| MetadataParseError::Read(path.clone().into_owned(), error))?;
                let metadata = serde_json::from_slice::<UncheckedTypeMetadata>(&json)
                    .map_err(|error| MetadataParseError::Deserialize(path.into_owned(), error))?;

                Ok((metadata.vct.clone(), (metadata, json)))
            })
            .try_collect()?;

        Ok(Self(documents))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Into)]
pub struct CredentialMetadataFile(CredentialMetadata);

impl<'de> Deserialize<'de> for CredentialMetadataFile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let path = String::deserialize(deserializer)?;

        Self::try_from(path).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for CredentialMetadataFile {
    type Error = MetadataParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let path = prefix_local_path(PathBuf::from(value));
        let json = fs::read(&path).map_err(|error| MetadataParseError::Read(path.clone().into_owned(), error))?;
        let metadata =
            serde_json::from_slice(&json).map_err(|error| MetadataParseError::Deserialize(path.into_owned(), error))?;

        Ok(Self(metadata))
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

            let (metadata_document, metadata_json) = metadata_by_vct
                .get(vct)
                .ok_or_else(|| TypeMetadataDocumentsError::MissingDocument(vct.to_string()))?;

            documents.push(metadata_json.clone());

            next_vct = metadata_document
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
pub enum CredentialConfigurationsSettingsError {
    #[error("invalid private key: {0}")]
    PrivateKey(#[source] PrivateKeySettingsError),

    #[error("could not compile SD-JWT VC Type Metadata chain: {0}")]
    TypeMetadataChain(#[source] TypeMetadataDocumentsError),

    #[error("could not initialize status: {0}")]
    StatusList(#[source] StatusListAttestationSettingsError),
}

impl CredentialConfigurationsSettings {
    pub async fn into_params(
        self,
        status_list_connection: DatabaseConnection,
        public_url: BaseUrl,
        hsm: Option<Pkcs11Hsm>,
        status_list_settings: &StatusListsSettings,
        metadata_by_vct: &TypeMetadataByVct,
    ) -> Result<
        HashMap<
            CredentialConfigurationId,
            CredentialConfigurationParameters<
                PrivateKeyVariant,
                PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
            >,
        >,
        CredentialConfigurationsSettingsError,
    > {
        let Self(inner) = self;

        let config_count = inner.len();
        let config_params = try_join_all(
            inner
                .into_iter()
                .zip_eq(std::iter::repeat_n(
                    (status_list_connection, public_url, hsm),
                    config_count,
                ))
                .map(|((config_id, settings), (status_list_connection, public_url, hsm))| {
                    async move {
                        // An mdoc is described by its CredentialMetadata, so no chain needs to be configured for it.
                        let type_metadata = match settings.credential_kind.format {
                            Format::SdJwt => Some(
                                metadata_by_vct
                                    .to_metadata_documents(&settings.credential_kind.attestation_type)
                                    .map_err(CredentialConfigurationsSettingsError::TypeMetadataChain)?,
                            ),
                            Format::MsoMdoc => None,
                        };

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

                        let params = CredentialConfigurationParameters {
                            credential_kind: settings.credential_kind,
                            key_pair,
                            status_list,
                            valid_days: Days::new(settings.valid_days),
                            type_metadata,
                            credential_metadata: settings.credential_metadata.map(CredentialMetadata::from),
                        };

                        Ok::<_, CredentialConfigurationsSettingsError>((config_id, params))
                    }
                }),
        )
        .await?
        .into_iter()
        .collect();

        Ok(config_params)
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

    #[error("could not initialize credential configuration parameters: {0}")]
    CredentialConfigurationParameters(#[source] CredentialConfigurationsSettingsError),

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

        let config_params = self
            .credential_configurations
            .into_params(
                status_list_connection,
                self.public_url.as_base_url().clone(),
                hsm,
                &self.status_lists,
                &self.type_metadata,
            )
            .await
            .map_err(IssuerSettingsError::CredentialConfigurationParameters)?;

        let issuer = Issuer::try_new(
            self.public_url,
            metadata_keypair,
            self.batch_size,
            self.wallet_client_ids,
            config_params,
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
    use attestation_data::x509::CertificateTypeError;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::credential_format::Format;
    use attestation_types::credential_kind::CredentialKind;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CertificateConfiguration;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;
    use crypto::x509::DistinguishedName;
    use crypto::x509::SubjectAltNameUri;
    use openid4vc::mock::MOCK_WALLET_CLIENT_ID;
    use sd_jwt_vc_metadata::TypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use server_utils::settings::CertificateVerificationError;
    use server_utils::settings::Server;
    use server_utils::settings::ServerAuth;
    use server_utils::settings::Settings;
    use server_utils::settings::Storage;
    use status_lists::publish::PublishDir;
    use status_lists::settings::StatusListsSettings;
    use utils::num::NonZeroU31;
    use utils::num::Ratio;

    use super::CredentialConfigurationSettings;
    use super::IssuerSettings;
    use super::StatusListAttestationSettings;
    use super::TypeMetadataByVct;
    use crate::settings::IssuerSettingsValidationError;

    fn mock_settings(wrpac_ca: &Ca, issuer_ca: &Ca) -> IssuerSettings {
        let wrpac_keypair = wrpac_ca
            .generate_wrpac_issuer_mock()
            .expect("generate metadata cert failed")
            .into();

        let issuance_keypair = generate_issuer_mock_with_registration(issuer_ca, &IssuerRegistration::new_mock())
            .expect("generate issuer cert failed")
            .into();

        let status_list_keypair = issuer_ca
            .generate_issuer_status_list_mock()
            .expect("generate tsl cert failed")
            .into();

        // Normally this is its own CA; here we just reuse the issuer_ca.
        let wia_trust_anchors = vec![issuer_ca.to_borrowing_trust_anchor()].try_into().unwrap();

        IssuerSettings {
            public_url: "https://example.com".parse().unwrap(),
            credential_configurations: HashMap::from([(
                "pid_sdjwt".to_string().into(),
                CredentialConfigurationSettings {
                    credential_kind: CredentialKind::new(Format::SdJwt, "com.example.pid".to_string()),
                    credential_metadata: None,
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
            credential_metadata_keypair: wrpac_keypair,
            type_metadata: TypeMetadataByVct(HashMap::from([{
                let metadata = UncheckedTypeMetadata::pid_example();
                let vct = metadata.vct.clone();
                let metadata_bytes = serde_json::to_vec(&metadata).unwrap();
                (vct, (metadata, metadata_bytes))
            }])),
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
            CredentialConfigurationSettings {
                credential_kind: CredentialKind::new(Format::SdJwt, "com.example.no_registration".to_string()),
                credential_metadata: None,
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

        let no_registration_metadata = UncheckedTypeMetadata {
            vct: "com.example.no_registration".to_string(),
            ..UncheckedTypeMetadata::empty_example()
        };
        let no_registration_metadata_serialized = serde_json::to_vec(&no_registration_metadata).unwrap();
        let pid_metadata = TypeMetadata::pid_example().into_inner();
        let pid_metadata_serialized = serde_json::to_vec(&pid_metadata).unwrap();

        settings.type_metadata = TypeMetadataByVct(HashMap::from([
            (
                no_registration_metadata.vct.clone(),
                (no_registration_metadata, no_registration_metadata_serialized),
            ),
            (pid_metadata.vct.clone(), (pid_metadata, pid_metadata_serialized)),
        ]));

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
}
