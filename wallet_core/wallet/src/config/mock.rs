use base64::{engine::general_purpose::STANDARD, Engine};
use url::Url;

use wallet_common::account::jwt::EcdsaDecodingKey;

use super::{AccountServerConfiguration, Configuration, ConfigurationRepository};

pub struct MockConfigurationRepository(pub Configuration);

impl ConfigurationRepository for MockConfigurationRepository {
    fn config(&self) -> &Configuration {
        &self.0
    }
}

impl Default for MockConfigurationRepository {
    fn default() -> Self {
        MockConfigurationRepository(Configuration {
            account_server: AccountServerConfiguration {
                base_url: Url::parse("http://rijksoverheid.nl").unwrap(),
                public_key: EcdsaDecodingKey::from_sec1(&STANDARD.decode("").unwrap()),
            },
        })
    }
}
