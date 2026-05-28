use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::sync::Arc;

use attestation_types::qualification::AttestationQualification;
use chrono::Days;
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
use http_utils::urls::HttpsUri;
use itertools::Itertools;
use openid4vc::Format;
use openid4vc::credential_configurations::CredentialConfigurationParameters;
use openid4vc::credential_configurations::CredentialConfigurationsError;
use openid4vc::issuer::IssuanceData;
use openid4vc::issuer::Issuer;
use openid4vc::issuer::WiaConfig;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::issuer_metadata::CredentialConfigurationId;
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
use utils::generator::TimeGenerator;
use utils::path::prefix_local_path;

use crate::nonce_store::ProofNonceStore;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct IssuerSettings {
    /// Publicly reachable URL used by the wallet during sessions, which should be a valid Credential Issuer
    /// Identifier.
    pub public_url: IssuerIdentifier,

    pub credential_configurations: CredentialConfigurationsSettings,

    #[debug(skip)]
    #[serde_as(as = "TryFromInto<Vec<String>>")]
    pub metadata: TypeMetadataByVct,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: Vec<String>,

    /// The maximum amount of copies of a credential that the holder can request.
    pub batch_size: NonZeroU8,

    #[serde(flatten)]
    #[debug(skip)]
    pub server_settings: Settings,

    pub status_lists: StatusListsSettings,
}

#[derive(Debug, Clone, AsRef)]
pub struct TypeMetadataByVct(HashMap<String, (UncheckedTypeMetadata, Vec<u8>)>);

#[derive(Debug, Clone, Deserialize, From, IntoIterator, AsRef)]
pub struct CredentialConfigurationsSettings(
    #[into_iterator(owned, ref)] HashMap<CredentialConfigurationId, CredentialConfigurationSettings>,
);

#[derive(Debug, Clone, Deserialize)]
pub struct CredentialConfigurationSettings {
    pub format: Format,
    pub attestation_type: String,

    #[serde(flatten)]
    #[debug(skip)]
    pub keypair: KeyPair,

    pub valid_days: u64,

    pub status_list: StatusListAttestationSettings,

    #[serde(default)]
    pub attestation_qualification: AttestationQualification,

    /// Which of the SAN fields in the issuer certificate to use as the `issuer_uri`/`iss` field in the mdoc/SD-JWT.
    /// If the certificate contains exactly one SAN, then this may be left blank.
    pub certificate_san: Option<HttpsUri>,
}

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataParseError {
    #[error("could not read \"{0}\": {1}")]
    Read(PathBuf, #[source] std::io::Error),

    #[error("could not deserialize \"{0}\": {1}")]
    Deserialize(PathBuf, #[source] serde_json::Error),
}

impl TryFrom<Vec<String>> for TypeMetadataByVct {
    type Error = TypeMetadataParseError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        // Map the contents of each JSON file by the `vct` field by decoding the JSON and extracting just that field.
        let documents = value
            .into_iter()
            .map(|path| {
                let path = prefix_local_path(PathBuf::from(path));
                let json =
                    fs::read(&path).map_err(|error| TypeMetadataParseError::Read(path.clone().into_owned(), error))?;
                let metadata = serde_json::from_slice::<UncheckedTypeMetadata>(&json)
                    .map_err(|error| TypeMetadataParseError::Deserialize(path.into_owned(), error))?;

                Ok((metadata.vct.clone(), (metadata, json)))
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
    #[error("invalid certificate: {0}")]
    CertificateSanDns(#[source] CertificateError),

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
                .zip_eq(itertools::repeat_n(
                    (status_list_connection, public_url, hsm),
                    config_count,
                ))
                .map(|((config_id, settings), (status_list_connection, public_url, hsm))| {
                    async move {
                        // Take the SAN from the settings if specified, or otherwise take the first SAN from the
                        // certificate. NB: the settings validation function will have verified before
                        // this that the certificate contains just one SAN.
                        let issuer_uri = match settings.certificate_san {
                            Some(san) => san,
                            None => {
                                let san_dns_name_or_uris = settings
                                    .keypair
                                    .certificate
                                    .san_dns_name_or_uris()
                                    .map_err(CredentialConfigurationsSettingsError::CertificateSanDns)?;

                                san_dns_name_or_uris.first().clone()
                            }
                        };

                        let metadata_documents = metadata_by_vct
                            .to_metadata_documents(&settings.attestation_type)
                            .map_err(CredentialConfigurationsSettingsError::TypeMetadataChain)?;

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
                            format: settings.format,
                            attestation_type: settings.attestation_type,
                            key_pair,
                            status_list,
                            valid_days: Days::new(settings.valid_days),
                            issuer_uri,
                            attestation_qualification: settings.attestation_qualification,
                            metadata_documents,
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
    #[error("certificate for {config_id} missing SAN {san}")]
    CertificateMissingSan {
        config_id: CredentialConfigurationId,
        san: HttpsUri,
    },
    #[error("multiple SANs in issuer certificate for {config_id}: which one to use was not specified")]
    CertificateSanUnspecified { config_id: CredentialConfigurationId },
    #[error(
        "attestation and status list certificate subject are different {config_id}: `{attestation}` vs `{status_list}`"
    )]
    CertificatesSubjectNameMismatch {
        config_id: CredentialConfigurationId,
        attestation: String,
        status_list: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerSettingsError {
    #[error("could not initialize HSM: {0}")]
    Hsm(#[source] HsmError),

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
}

impl IssuerSettings {
    pub fn validate(&self) -> Result<(), IssuerSettingsValidationError> {
        tracing::debug!("verifying issuer settings");

        for (config_id, attestation) in self.credential_configurations.as_ref() {
            if let Some(certificate_san) = attestation.certificate_san.as_ref() {
                // If the certificate SAN to be used has been specified, then it has to be present in the certificate.
                if !attestation
                    .keypair
                    .certificate
                    .san_dns_name_or_uris()?
                    .as_ref()
                    .contains(certificate_san)
                {
                    return Err(IssuerSettingsValidationError::CertificateMissingSan {
                        config_id: config_id.clone(),
                        san: certificate_san.clone(),
                    });
                }
            } else {
                // If not, then there must be only one SAN in the certificate so there is no disambiguation.
                if attestation.keypair.certificate.san_dns_name_or_uris()?.len().get() > 1 {
                    return Err(IssuerSettingsValidationError::CertificateSanUnspecified {
                        config_id: config_id.clone(),
                    });
                }
            }
        }

        let time = TimeGenerator;

        let trust_anchors = &self.server_settings.issuer_trust_anchors;

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .credential_configurations
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.as_ref(), &attestation.keypair))
            .collect();

        verify_key_pairs(&key_pairs, trust_anchors, CertificateUsage::Mdl, &time)?;

        let key_pairs: Vec<(&str, &KeyPair)> = self
            .credential_configurations
            .as_ref()
            .iter()
            .map(|(typ, attestation)| (typ.as_ref(), &attestation.status_list.keypair))
            .collect();

        verify_key_pairs(&key_pairs, trust_anchors, CertificateUsage::OAuthStatusSigning, &time)?;

        for (config_id, attestation) in self.credential_configurations.as_ref() {
            let attestation_dn = attestation.keypair.certificate.distinguished_name()?;
            let status_list_dn = attestation.status_list.keypair.certificate.distinguished_name()?;
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

    #[expect(clippy::too_many_arguments)]
    pub async fn into_issuer<A, PAS, PKS, UAA>(
        self,
        hsm: Option<Pkcs11Hsm>,
        wia_config: Option<WiaConfig>,
        attr_service: A,
        par_store: impl FnOnce(StoreConnection) -> PAS,
        pkce_flow_store: impl FnOnce(StoreConnection) -> PKS,
        upstream_authorization_adapter: Option<UAA>,
    ) -> Result<
        (
            Issuer<
                A,
                PrivateKeyVariant,
                PostgresStatusListService<PrivateKeyVariant, NoRevokeAll>,
                SessionStoreVariant<IssuanceData>,
                ProofNonceStore,
                PAS,
                PKS,
                UAA,
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

        let config_params = self
            .credential_configurations
            .into_params(
                status_list_connection,
                self.public_url.as_base_url().clone(),
                hsm,
                &self.status_lists,
                &self.metadata,
            )
            .await
            .map_err(IssuerSettingsError::CredentialConfigurationParameters)?;

        let par_store = par_store(store_connection.clone());
        let pkce_flow_store = pkce_flow_store(store_connection.clone());

        let issuer = Issuer::try_new(
            self.public_url,
            self.batch_size,
            self.wallet_client_ids,
            config_params,
            wia_config,
            attr_service,
            Arc::new(sessions),
            proof_nonce_store,
            Arc::new(par_store),
            Arc::new(pkce_flow_store),
            upstream_authorization_adapter,
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

        Ok(service)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::num::NonZeroU8;
    use std::num::NonZeroU16;

    use attestation_data::auth::issuer_auth::IssuerRegistration;
    use attestation_data::x509::CertificateTypeError;
    use attestation_data::x509::generate::mock::generate_issuer_mock_with_registration;
    use attestation_types::qualification::AttestationQualification;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;
    use http_utils::urls::HttpsUri;
    use openid4vc::Format;
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

    fn mock_settings(issuer_ca: &Ca) -> IssuerSettings {
        let keypair = generate_issuer_mock_with_registration(issuer_ca, IssuerRegistration::new_mock())
            .expect("generate issuer cert failed")
            .into();

        let status_list_keypair = issuer_ca
            .generate_status_list_mock()
            .expect("generate tsl cert failed")
            .into();

        IssuerSettings {
            public_url: "https://example.com".parse().unwrap(),
            credential_configurations: HashMap::from([(
                "pid_sdjwt".to_string().into(),
                CredentialConfigurationSettings {
                    attestation_type: "com.example.pid".to_string(),
                    format: Format::SdJwt,
                    keypair,
                    valid_days: 365,
                    status_list: StatusListAttestationSettings {
                        group_name: "pid_sdjwt".to_string(),
                        base_url: None,
                        context_path: "tsl".to_string(),
                        keypair: status_list_keypair,
                        publish_dir: PublishDir::try_new(std::env::temp_dir()).unwrap(),
                    },
                    attestation_qualification: AttestationQualification::PubEAA,
                    certificate_san: Some(("https://".to_string() + ISSUANCE_CERT_CN).parse().unwrap()),
                },
            )])
            .into(),
            metadata: TypeMetadataByVct(HashMap::from([{
                let metadata = UncheckedTypeMetadata::pid_example();
                let vct = metadata.vct.clone();
                let metadata_bytes = serde_json::to_vec(&metadata).unwrap();
                (vct, (metadata, metadata_bytes))
            }])),
            wallet_client_ids: vec![MOCK_WALLET_CLIENT_ID.to_string()],
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
        }
    }

    #[test]
    fn test_validate() {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        mock_settings(&issuer_ca).validate().unwrap();
    }

    #[test]
    fn test_no_issuer_trust_anchors() {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&issuer_ca);

        settings.server_settings.issuer_trust_anchors = TrustAnchors::empty();

        assert_matches!(
            settings.validate().expect_err("should fail"),
            IssuerSettingsValidationError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
        );
    }

    #[test]
    fn test_no_issuer_registration() {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&issuer_ca);

        let issuer_cert_no_registration = issuer_ca
            .generate_issuer_mock()
            .expect("generate issuer cert without issuer registration");

        let status_list_keypair = issuer_ca
            .generate_status_list_mock()
            .expect("generate tsl cert failed")
            .into();

        settings.server_settings.issuer_trust_anchors = TrustAnchors::from(&issuer_ca);
        settings.credential_configurations = HashMap::from([(
            "no_registration_sdjwt".to_string().into(),
            CredentialConfigurationSettings {
                attestation_type: "com.example.no_registration".to_string(),
                format: Format::SdJwt,
                keypair: issuer_cert_no_registration.into(),
                valid_days: 365,
                status_list: StatusListAttestationSettings {
                    group_name: "no_registration_sdjwt".to_string(),
                    base_url: None,
                    context_path: "tsl".to_string(),
                    keypair: status_list_keypair,
                    publish_dir: PublishDir::try_new(std::env::temp_dir()).unwrap(),
                },
                attestation_qualification: Default::default(),
                certificate_san: None,
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

        settings.metadata = TypeMetadataByVct(HashMap::from([
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
    fn test_wrong_san_field() {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&issuer_ca);

        let wrong_san: HttpsUri = "https://wrong.san.example.com".parse().unwrap();

        let (typ, attestation_settings) = settings.credential_configurations.as_ref().iter().next().unwrap();
        let mut attestation_settings = attestation_settings.clone();
        attestation_settings.certificate_san = Some(wrong_san.clone());
        settings.credential_configurations = HashMap::from([(typ.clone(), attestation_settings)]).into();

        let error = settings.validate().expect_err("should fail");
        assert_matches!(error, IssuerSettingsValidationError::CertificateMissingSan { san, .. } if san == wrong_san);
    }

    #[test]
    fn test_status_list_invalid_usage() {
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&issuer_ca);

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
        let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA failed");
        let mut settings = mock_settings(&issuer_ca);

        let status_list_keypair = issuer_ca
            .generate_key_pair(
                "different.example.com",
                CertificateUsage::OAuthStatusSigning,
                Default::default(),
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
                    attestation == "CN=cert.issuer.example.com" &&
                    status_list == "CN=different.example.com"
        );
    }
}
