use std::collections::HashMap;

use derive_more::AsRef;
use derive_more::From;
use derive_more::IntoIterator;
use nutype::nutype;
use ring::hmac;
use serde::Deserialize;
use serde_with::hex::Hex;
use serde_with::serde_as;

use openid4vc::verifier::SessionTypeReturnUrl;
use openid4vc::verifier::UseCase;
use openid4vc::verifier::UseCases;
use wallet_common::urls::CorsOrigin;

use super::*;

const MIN_KEY_LENGTH_BYTES: usize = 16;

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct VerifierSettings {
    pub usecases: VerifierUseCases,
    #[serde_as(as = "Hex")]
    pub ephemeral_id_secret: EphemeralIdSecret,
    pub allow_origins: Option<CorsOrigin>,

    /// Reader trust anchors are used to verify the keys and certificates in the `verifier.usecases` configuration on
    /// application startup.
    #[serde_as(as = "Vec<Base64>")]
    pub reader_trust_anchors: Vec<BorrowingTrustAnchor>,

    // used by the application, SHOULD be reachable only by the application.
    // if not configured the wallet_server will be used, but an api_key is required in that case
    // if it conflicts with wallet_server, the application will crash on startup
    #[cfg(feature = "disclosure")]
    pub requester_server: RequesterAuth,

    pub universal_link_base_url: BaseUrl,

    #[serde(flatten)]
    pub server_settings: Settings,
}

#[derive(Clone, From, AsRef, IntoIterator, Deserialize)]
pub struct VerifierUseCases(HashMap<String, VerifierUseCase>);

#[nutype(validate(predicate = |v| v.len() >= MIN_KEY_LENGTH_BYTES), derive(Clone, TryFrom, AsRef, Deserialize))]
pub struct EphemeralIdSecret(Vec<u8>);

#[derive(Clone, Deserialize)]
pub struct VerifierUseCase {
    #[serde(default)]
    pub session_type_return_url: SessionTypeReturnUrl,
    #[serde(flatten)]
    pub key_pair: KeyPair,
}

impl TryFrom<VerifierUseCases> for UseCases {
    type Error = anyhow::Error;

    fn try_from(value: VerifierUseCases) -> Result<Self, Self::Error> {
        let use_cases = value
            .into_iter()
            .map(|(id, use_case)| {
                let use_case = UseCase::try_from(use_case)?;

                Ok((id, use_case))
            })
            .collect::<Result<HashMap<_, _>, Self::Error>>()?
            .into();

        Ok(use_cases)
    }
}

impl TryFrom<VerifierUseCase> for UseCase {
    type Error = anyhow::Error;

    fn try_from(value: VerifierUseCase) -> Result<Self, Self::Error> {
        let use_case = UseCase::try_new(value.key_pair.try_into()?, value.session_type_return_url)?;

        Ok(use_case)
    }
}

impl From<&EphemeralIdSecret> for hmac::Key {
    fn from(value: &EphemeralIdSecret) -> Self {
        hmac::Key::new(hmac::HMAC_SHA256, value.as_ref())
    }
}

impl ServerSettings for VerifierSettings {
    fn new_custom(config_file: &str, env_prefix: &str) -> Result<Self, ConfigError> {
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
            )?;

        let config_builder = config_builder
            .set_default("universal_link_base_url", DEFAULT_UNIVERSAL_LINK_BASE)?
            .set_default("requester_server.ip", "0.0.0.0")?
            .set_default("requester_server.port", 3002)?;

        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_source = utils::prefix_local_path(config_file.as_ref());

        let environment_parser = Environment::with_prefix(env_prefix)
            .separator("__")
            .prefix_separator("__")
            .list_separator(",");

        let environment_parser = environment_parser.with_list_parse_key("reader_trust_anchors");
        let environment_parser = environment_parser.try_parsing(true);

        let config = config_builder
            .add_source(File::from(config_source.as_ref()).required(false))
            .add_source(File::from(config_file.as_ref()).required(false))
            .add_source(environment_parser)
            .build()?
            .try_deserialize()?;

        Ok(config)
    }

    fn verify_key_pairs(&self) -> Result<(), CertificateVerificationError> {
        tracing::debug!("verifying verifier.usecases certificates");

        let time = TimeGenerator;

        let trust_anchors: Vec<TrustAnchor<'_>> = self
            .reader_trust_anchors
            .iter()
            .map(BorrowingTrustAnchor::to_owned_trust_anchor)
            .collect::<Vec<_>>();

        let key_pairs: Vec<(String, KeyPair)> = self
            .usecases
            .as_ref()
            .iter()
            .map(|(use_case_id, usecase)| (use_case_id.clone(), usecase.key_pair.clone()))
            .collect();

        verify_key_pairs(
            &key_pairs,
            &trust_anchors,
            CertificateUsage::ReaderAuth,
            &time,
            |certificate_type| matches!(certificate_type, CertificateType::ReaderAuth(Some(_))),
        )?;

        Ok(())
    }

    fn structured_logging(&self) -> bool {
        self.server_settings.structured_logging
    }
}
