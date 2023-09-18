use super::{Configuration, ConfigurationRepository};

// TODO: This will become HttpConfigurationRepository in the near future.
pub struct LocalConfigurationRepository {
    config: Configuration,
}

impl LocalConfigurationRepository {
    pub fn new(config: Configuration) -> Self {
        LocalConfigurationRepository { config }
    }
}

impl Default for LocalConfigurationRepository {
    fn default() -> Self {
        Self::new(Configuration::default())
    }
}

impl ConfigurationRepository for LocalConfigurationRepository {
    fn config(&self) -> &Configuration {
        &self.config
    }
}
