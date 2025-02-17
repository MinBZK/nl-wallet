use std::env;
use std::path::PathBuf;

// re-export for convenience
pub use async_dropper::AsyncDropper;

use async_dropper::AsyncDrop;
use async_trait::async_trait;
use config::Config;
use config::ConfigError;
use config::File;
use serde::Deserialize;
use serde_with::serde_as;

use wallet_common::utils::random_string;

use crate::model::Hsm;
use crate::service::Pkcs11Hsm;
use crate::settings;

#[serde_as]
#[derive(Clone, Deserialize)]
struct TestSettings {
    pub(crate) hsm: settings::Hsm,
}

// Default is needed for AsyncDrop
pub struct TestCase<H> {
    identifier: String,
    hsm: Option<H>,
}

// Default is needed for AsyncDrop
impl<H> Default for TestCase<H> {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            hsm: None,
        }
    }
}

impl<H> TestCase<H> {
    pub fn test_params(&self) -> (&H, &str) {
        (self.hsm.as_ref().unwrap(), &self.identifier)
    }
}

#[cfg(feature = "mock")]
mod mock {
    use crate::model::mock::MockPkcs11Client;
    use crate::service::HsmError;

    use super::TestCase;

    impl TestCase<MockPkcs11Client<HsmError>> {
        pub fn mock(identifier_prefix: &str) -> Self {
            Self {
                identifier: identifier_prefix.to_string(),
                hsm: Some(MockPkcs11Client::default()),
            }
        }
    }
}

impl TestCase<Pkcs11Hsm> {
    pub fn new(config_file: &str, identifier_prefix: &str) -> Self {
        // let (hsm, settings) = setup_hsm();
        let settings = TestSettings::new(config_file).unwrap();
        let hsm = Pkcs11Hsm::from_settings(settings.hsm.clone()).unwrap();
        Self {
            identifier: format!("{}-{}", identifier_prefix, random_string(8)),
            hsm: Some(hsm),
        }
    }
}

#[async_trait]
impl AsyncDrop for TestCase<Pkcs11Hsm> {
    async fn async_drop(&mut self) -> () {
        let (hsm, identifier) = self.test_params();
        let _ = Hsm::delete_key(hsm, identifier).await;
    }
}

impl TestSettings {
    fn new(config_file: &str) -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .add_source(File::from(config_path.join(config_file)).required(true))
            .build()?
            .try_deserialize()
    }
}
