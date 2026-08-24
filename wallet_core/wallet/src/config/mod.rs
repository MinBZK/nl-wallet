mod config_file;
mod data;
mod file_repository;
mod http_repository;
#[cfg(any(test, feature = "test"))]
mod mock;
mod updating_repository;

use std::sync::LazyLock;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use error_category::ErrorCategory;
use http_utils::client::TlsPinningConfig;
use jwt::Algorithm;
use jwt::JwtValidation;
use jwt::UnverifiedJwt;
use jwt::ValidationWrapper;
use jwt::error::JwtParseError;
use jwt::error::JwtVerifyError;
use wallet_configuration::wallet_config::WalletConfiguration;

pub use self::data::UNIVERSAL_LINK_BASE_URL;
pub use self::data::default_config_server_config;
pub use self::data::default_wallet_config;
pub use self::data::init_universal_link_base_url;
pub use self::file_repository::FileStorageConfigurationRepository;
pub use self::http_repository::HttpConfigurationRepository;
pub use self::updating_repository::UpdatingConfigurationRepository;
use crate::repository::FileStorageError;
use crate::repository::HttpClientError;

pub type WalletConfigJwt = UnverifiedJwt<WalletConfiguration>;

/// Leeway allowed when evaluating the `exp` of a [`WalletConfiguration`], in order to handle clock skew between the
/// wallet device and the configuration server.
pub const CONFIG_EXPIRY_LEEWAY: Duration = Duration::from_secs(60);

pub static WALLET_CONFIG_VALIDATION: LazyLock<ValidationWrapper> = LazyLock::new(|| {
    let mut validation = JwtValidation::default_with_algorithms([Algorithm::ES256]);
    validation.require_exp();
    validation.set_leeway(CONFIG_EXPIRY_LEEWAY);

    validation
        .try_into_validation()
        .expect("should succeed because only one algorithm family is used to create this validation")
});

/// Checks whether the given configuration is expired at `now`, using the same leeway as [`WALLET_CONFIG_VALIDATION`].
///
/// Note that this necessarily trusts the wall clock of the device, which the user controls. It therefore protects a
/// wallet that is honestly offline from continuing to trust retired key material, but is not a defence against a user
/// who deliberately sets back the clock of their own device.
pub fn is_expired(config: &WalletConfiguration, now: DateTime<Utc>) -> bool {
    let expires: DateTime<Utc> = config.expires.into();

    expires + CONFIG_EXPIRY_LEEWAY < now
}

pub type WalletConfigurationRepository =
    UpdatingConfigurationRepository<FileStorageConfigurationRepository<HttpConfigurationRepository<TlsPinningConfig>>>;

#[cfg(any(test, feature = "test"))]
pub use self::mock::LocalConfigurationRepository;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum ConfigurationError {
    #[error("could not store or load configuration or etag file: {0}")]
    FileStorage(#[from] FileStorageError),
    #[error("could not parse JWT: {0}")]
    JwtParse(#[from] JwtParseError),
    #[error("could not validate JWT: {0}")]
    JwtVerify(#[from] JwtVerifyError),
    #[error("http client error: {0}")]
    HttpClient(#[from] HttpClientError),
}

#[cfg(test)]
pub(crate) mod test {
    use chrono::TimeDelta;
    use chrono::Timelike;
    use chrono::Utc;
    use rstest::rstest;
    use wallet_configuration::wallet_config::WalletConfiguration;

    use super::CONFIG_EXPIRY_LEEWAY;
    use super::is_expired;

    const TEST_WALLET_CONFIG_JSON: &str = include_str!("../../test-wallet-config.json");

    pub fn test_wallet_config() -> WalletConfiguration {
        // The JSON has already been parsed in build.rs, so unwrap is safe here
        serde_json::from_str(TEST_WALLET_CONFIG_JSON).unwrap()
    }

    fn leeway() -> TimeDelta {
        TimeDelta::from_std(CONFIG_EXPIRY_LEEWAY).unwrap()
    }

    #[rstest]
    #[case(TimeDelta::hours(1), false)]
    #[case(TimeDelta::seconds(1), false)]
    // Expiry within the leeway should not yet be considered expired, matching what JWT verification accepts.
    #[case(TimeDelta::zero(), false)]
    #[case(-leeway(), false)]
    // Beyond the leeway the configuration is expired.
    #[case(-leeway() - TimeDelta::seconds(1), true)]
    #[case(-TimeDelta::hours(1), true)]
    fn test_is_expired(#[case] expires_in: TimeDelta, #[case] expected_expired: bool) {
        // `exp` has seconds precision, so use a whole-second `now`.
        let now = Utc::now().with_nanosecond(0).unwrap();

        let config = WalletConfiguration {
            expires: (now + expires_in).into(),
            ..test_wallet_config()
        };

        assert_eq!(is_expired(&config, now), expected_expired);
    }
}
