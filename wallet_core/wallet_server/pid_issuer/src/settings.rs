use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU8;
use std::num::NonZeroUsize;

use chrono::Days;
use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use futures::future::join_all;
use indexmap::IndexMap;
use rustls_pki_types::TrustAnchor;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;
use serde_with::base64::Base64;
use serde_with::serde_as;

use hsm::service::Pkcs11Hsm;
use nl_wallet_mdoc::server_keys::KeyPair as ParsedKeyPair;
use nl_wallet_mdoc::utils::x509::CertificateType;
use nl_wallet_mdoc::utils::x509::CertificateUsage;
use openid4vc::server_state::SessionStoreTimeouts;
use sd_jwt::metadata::TypeMetadata;
use server_utils::keys::PrivateKeySettingsError;
use server_utils::keys::PrivateKeyVariant;
use server_utils::settings::verify_key_pairs;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::settings::TryFromKeySettings;
use wallet_common::config::http::TlsPinningConfig;
use wallet_common::generator::TimeGenerator;
use wallet_common::p256_der::DerVerifyingKey;
use wallet_common::trust_anchor::BorrowingTrustAnchor;
use wallet_common::urls::BaseUrl;
use wallet_common::urls::HttpsUri;
use wallet_common::utils;

use crate::pid::attributes::BrpPidAttributeService;
use crate::pid::attributes::Error as BrpError;
use crate::pid::brp::client::HttpBrpClient;
use crate::server::IssuerKeyRing;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct IssuerSettings {
    /// Issuer private keys index per doctype
    pub private_keys: HashMap<String, KeyPair>,

    #[serde(deserialize_with = "deserialize_type_metadata")]
    pub metadata: Vec<TypeMetadata>,

    /// `client_id` values that this server accepts, identifying the wallet implementation (not individual instances,
    /// i.e., the `client_id` value of a wallet implementation will be constant across all wallets of that
    /// implementation).
    /// The wallet sends this value in the authorization request and as the `iss` claim of its Proof of Possession
    /// JWTs.
    pub wallet_client_ids: Vec<String>,

    pub digid: Digid,

    pub brp_server: BaseUrl,

    #[serde_as(as = "Base64")]
    pub wte_issuer_pubkey: DerVerifyingKey,

    pub valid_days: u64,

    pub copy_count: NonZeroU8,

    #[serde(flatten)]
    pub server_settings: Settings,
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

#[derive(Clone, Deserialize)]
pub struct Digid {
    pub bsn_privkey: String,
    pub http_config: TlsPinningConfig,
}

impl IssuerSettings {
    pub fn issuer_uris(&self) -> Result<IndexMap<String, HttpsUri>, BrpError> {
        self.private_keys
            .iter()
            .map(|(doctype, key_pair)| {
                let issuer_san_dns_name_or_uris = key_pair.certificate.san_dns_name_or_uris()?;
                let issuer_uri = match issuer_san_dns_name_or_uris.len() {
                    NonZeroUsize::MIN => Ok(issuer_san_dns_name_or_uris.into_first()),
                    n => Err(BrpError::UnexpectedIssuerSanDnsNameOrUrisCount(n)),
                }?;

                Ok((doctype.to_owned(), issuer_uri))
            })
            .collect::<Result<IndexMap<_, _>, _>>()
    }

    pub fn metadata(&self) -> IndexMap<String, TypeMetadata> {
        self.metadata
            .iter()
            .map(|type_metadata| (type_metadata.vct.clone(), type_metadata.clone()))
            .collect()
    }
}

impl TryFrom<&IssuerSettings> for BrpPidAttributeService {
    type Error = BrpError;

    fn try_from(issuer: &IssuerSettings) -> Result<Self, Self::Error> {
        BrpPidAttributeService::new(
            HttpBrpClient::new(issuer.brp_server.clone()),
            &issuer.digid.bsn_privkey,
            issuer.digid.http_config.clone(),
            issuer.issuer_uris()?,
            issuer.metadata(),
            Days::new(issuer.valid_days),
            issuer.copy_count,
        )
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

impl ServerSettings for IssuerSettings {
    type ValidationError = CertificateVerificationError;

    fn new(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
        let default_store_timeouts = SessionStoreTimeouts::default();

        let config_builder = Config::builder()
            .set_default("wallet_server.ip", "0.0.0.0")?
            .set_default("wallet_server.port", 3001)?
            .set_default("public_url", "http://localhost:3001/")?
            .set_default("log_requests", false)?
            .set_default("structured_logging", false)?
            .set_default("storage.url", "memory://")?
            .set_default(
                "storage.expiration_minutes",
                default_store_timeouts.expiration.as_secs() / 60,
            )?
            .set_default(
                "storage.successful_deletion_minutes",
                default_store_timeouts.successful_deletion.as_secs() / 60,
            )?
            .set_default(
                "storage.failed_deletion_minutes",
                default_store_timeouts.failed_deletion.as_secs() / 60,
            )?
            .set_default(
                "wallet_client_ids",
                vec![wallet_common::jwt::NL_WALLET_CLIENT_ID.to_string()],
            )?
            .set_default("brp_server", "http://localhost:3007/")?
            .set_default("valid_days", 365)?
            .set_default("copy_count", 4)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = utils::prefix_local_path(config_file.as_ref());

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",")
            .with_list_parse_key("issuer_trust_anchors")
            .with_list_parse_key("digid.http_config.trust_anchors")
            .with_list_parse_key("metadata")
            .try_parsing(true);

        let config = config_builder
            .add_source(File::from(config_source.as_ref()).required(false))
            .add_source(File::from(config_file.as_ref()).required(false))
            .add_source(environment_parser)
            .build()?
            .try_deserialize()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), CertificateVerificationError> {
        tracing::debug!("verifying issuer.private_keys certificates");

        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'_>> = self
            .server_settings
            .issuer_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(String, KeyPair)> = self
            .private_keys
            .iter()
            .map(|(id, keypair)| (id.clone(), keypair.clone()))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::Mdl,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::Mdl(Some(_))),
        )
    }

    fn server_settings(&self) -> &Settings {
        &self.server_settings
    }
}
