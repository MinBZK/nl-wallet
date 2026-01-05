mod config_file;
mod data;
mod file_repository;
mod http_repository;
#[cfg(any(test, feature = "test"))]
mod mock;
mod updating_repository;

use error_category::ErrorCategory;
use http_utils::tls::pinning::TlsPinningConfig;
use jwt::error::JwtError;

use crate::repository::FileStorageError;
use crate::repository::HttpClientError;

pub use self::data::UNIVERSAL_LINK_BASE_URL;
pub use self::data::default_config_server_config;
pub use self::data::default_wallet_config;
pub use self::data::init_universal_link_base_url;
pub use self::file_repository::FileStorageConfigurationRepository;
pub use self::http_repository::HttpConfigurationRepository;
pub use self::updating_repository::UpdatingConfigurationRepository;

pub type WalletConfigurationRepository =
    UpdatingConfigurationRepository<FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>>>;

#[cfg(any(test, feature = "test"))]
pub use self::mock::LocalConfigurationRepository;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum ConfigurationError {
    #[error("could not store or load configuration or etag file: {0}")]
    FileStorage(#[from] FileStorageError),
    #[error("could not validate JWT: {0}")]
    Jwt(#[from] JwtError),
    #[error("http client error: {0}")]
    HttpClient(#[from] HttpClientError),
}

#[cfg(test)]
pub(crate) mod test {
    use std::collections::HashMap;

    use utils::vec_at_least::NonEmptyIterator;
    use wallet_configuration::wallet_config::PidAttributePaths;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use crate::config::default_wallet_config;

    fn as_example_urn(input: &str) -> String {
        input.replace("urn:eudi:pid", "urn:example:pid")
    }

    fn to_example_attributes((key, value): (String, PidAttributePaths)) -> (String, PidAttributePaths) {
        let key = as_example_urn(&key);
        let login = value.login.nonempty_iter().map(|attr| as_example_urn(attr)).collect();
        let recovery_code = value
            .recovery_code
            .nonempty_iter()
            .map(|attr| as_example_urn(attr))
            .collect();

        (key, PidAttributePaths { login, recovery_code })
    }

    pub fn test_wallet_config() -> WalletConfiguration {
        let mut wallet_configuration: WalletConfiguration = default_wallet_config();

        let mso_mdoc = &wallet_configuration
            .pid_attributes
            .mso_mdoc
            .into_iter()
            .map(to_example_attributes)
            .collect::<HashMap<_, _>>();

        wallet_configuration.pid_attributes.mso_mdoc = mso_mdoc.clone();

        let sd_jwt = &wallet_configuration
            .pid_attributes
            .sd_jwt
            .into_iter()
            .map(to_example_attributes)
            .collect::<HashMap<_, _>>();

        wallet_configuration.pid_attributes.sd_jwt = sd_jwt.clone();

        wallet_configuration
    }
}
