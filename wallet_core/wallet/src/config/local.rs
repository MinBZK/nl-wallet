use super::{Configuration, ConfigurationRepository};

// TODO: This will become RemoteConfigurationRepository in the near future.
pub struct LocalConfigurationRepository {
    config: Configuration,
}

impl LocalConfigurationRepository {
    pub fn new_with_initial<F>(f: F) -> Self
    where
        F: FnOnce() -> Configuration,
    {
        LocalConfigurationRepository { config: f() }
    }
}

impl ConfigurationRepository for LocalConfigurationRepository {
    fn config(&self) -> &Configuration {
        &self.config
    }
}
