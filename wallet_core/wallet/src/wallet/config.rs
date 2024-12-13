use std::sync::Arc;

use wallet_common::config::wallet_config::WalletConfiguration;
use wallet_common::update_policy::VersionState;

use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;

use super::Wallet;

impl<CR, S, PEK, APC, DS, IS, MDS, WIC, UR> Wallet<CR, S, PEK, APC, DS, IS, MDS, WIC, UR>
where
    UR: Repository<VersionState>,
{
    pub fn is_blocked(&self) -> bool {
        self.update_policy_repository.get() == VersionState::Block
    }
}

impl<CR, S, PEK, APC, DS, IS, MDS, WIC, UR> Wallet<CR, S, PEK, APC, DS, IS, MDS, WIC, UR>
where
    CR: ObservableRepository<Arc<WalletConfiguration>>,
    UR: ObservableRepository<VersionState>,
{
    pub fn set_config_callback(
        &self,
        mut callback: RepositoryCallback<Arc<WalletConfiguration>>,
    ) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        callback(self.config_repository.get());
        self.config_repository.register_callback_on_update(callback)
    }

    pub fn clear_config_callback(&self) -> Option<RepositoryCallback<Arc<WalletConfiguration>>> {
        self.config_repository.clear_callback()
    }

    pub fn set_version_state_callback(
        &self,
        mut callback: RepositoryCallback<VersionState>,
    ) -> Option<RepositoryCallback<VersionState>> {
        callback(self.update_policy_repository.get());
        self.update_policy_repository.register_callback_on_update(callback)
    }

    pub fn clear_version_state_callback(&self) -> Option<RepositoryCallback<VersionState>> {
        self.update_policy_repository.clear_callback()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;
    use tokio::sync::Notify;

    use wallet_common::config::wallet_config::WalletConfiguration;

    use crate::config::default_wallet_config;

    use super::super::test::WalletWithMocks;

    // Tests both setting and clearing the configuration callback.
    #[tokio::test]
    async fn test_wallet_set_clear_config_callback() {
        // Prepare an unregistered wallet.
        let wallet = WalletWithMocks::new_unregistered().await;

        // Wrap a `Vec<Configuration>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let configs = Arc::new(Mutex::new(Vec::<Arc<WalletConfiguration>>::with_capacity(1)));
        let callback_configs = Arc::clone(&configs);

        assert_eq!(Arc::strong_count(&configs), 2);

        let notifier = Arc::new(Notify::new());
        let callback_notifier = notifier.clone();

        // Set the configuration callback on the `Wallet`,
        // which should immediately be called exactly once.
        wallet.set_config_callback(Box::new(move |config| {
            callback_configs.lock().push(config);
            callback_notifier.notify_one();
        }));

        // Wait for the callback to be completed.
        notifier.notified().await;

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&configs), 2);

        // Test the contents of the `Vec<Configuration>`.
        {
            let configs = configs.lock();

            assert_eq!(configs.len(), 1);
            assert_eq!(
                configs.first().unwrap().account_server.http_config.base_url,
                default_wallet_config().account_server.http_config.base_url
            );
        }

        // Clear the configuration callback on the `Wallet.`
        wallet.clear_config_callback();

        assert_eq!(Arc::strong_count(&configs), 1);
    }
}
