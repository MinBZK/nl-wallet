use std::sync::Arc;

use configuration::wallet_config::WalletConfiguration;

use crate::config::data::default_wallet_config;
use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::ConfigurationError;

pub struct LocalConfigurationRepository {
    config: Arc<WalletConfiguration>,
}

impl LocalConfigurationRepository {
    pub fn new(config: WalletConfiguration) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Default for LocalConfigurationRepository {
    fn default() -> Self {
        Self::new(default_wallet_config())
    }
}

impl Repository<Arc<WalletConfiguration>> for LocalConfigurationRepository {
    fn get(&self) -> Arc<WalletConfiguration> {
        Arc::clone(&self.config)
    }
}

impl<B> UpdateableRepository<Arc<WalletConfiguration>, B> for LocalConfigurationRepository
where
    B: Send + Sync,
{
    type Error = ConfigurationError;

    async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<Arc<WalletConfiguration>>, Self::Error> {
        Ok(RepositoryUpdateState::Updated {
            from: self.get(),
            to: self.get(),
        })
    }
}

impl ObservableRepository<Arc<WalletConfiguration>> for LocalConfigurationRepository {
    fn register_callback_on_update(
        &self,
        _callback: RepositoryCallback<Arc<WalletConfiguration>>,
    ) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        None
    }

    fn clear_callback(&self) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        None
    }
}
