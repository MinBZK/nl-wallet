use super::{Configuration, ConfigurationRepository};

#[derive(Default)]
pub struct MockConfigurationRepository(pub Configuration);

impl ConfigurationRepository for MockConfigurationRepository {
    fn config(&self) -> &Configuration {
        &self.0
    }
}
