use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::ConfigurationRepository;

use super::Wallet;

pub type ConfigurationCallback = Box<dyn FnMut(&WalletConfiguration) + Send + Sync>;

impl<CR, S, PEK, APC, DGS, PIC, MDS> Wallet<CR, S, PEK, APC, DGS, PIC, MDS>
where
    CR: ConfigurationRepository,
{
    pub fn set_config_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&WalletConfiguration) + Send + Sync + 'static,
    {
        callback(self.config_repository.config());
        // TODO: Once configuration fetching from the Wallet Provider is implemented,
        //       this callback should be called every time the config updates.
        self.config_callback.replace(Box::new(callback));
    }

    pub fn clear_config_callback(&mut self) {
        self.config_callback.take();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::config::default_configuration;

    use super::{super::tests::WalletWithMocks, *};

    // Tests both setting and clearing the configuration callback.
    #[test]
    fn test_wallet_set_clear_config_callback() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::default();

        // Wrap a `Vec<Configuration>` in both a `Mutex` and `Arc`,
        // so we can write to it from the closure.
        let configs = Arc::new(Mutex::new(Vec::<WalletConfiguration>::with_capacity(1)));
        let callback_configs = Arc::clone(&configs);

        // Set the configuration callback on the `Wallet`,
        // which should immediately be called exactly once.
        wallet.set_config_callback(move |config| callback_configs.lock().unwrap().push(config.clone()));

        // Infer that the closure is still alive by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&configs), 2);

        // Test the contents of the `Vec<Configuration>`.
        {
            let configs = configs.lock().unwrap();

            assert_eq!(configs.len(), 1);
            assert_eq!(
                configs.first().unwrap().account_server.base_url,
                default_configuration().account_server.base_url
            );
        }

        // Clear the configuration callback on the `Wallet.`
        wallet.clear_config_callback();

        // Infer that the closure is now dropped by counting the `Arc` references.
        assert_eq!(Arc::strong_count(&configs), 1);
    }
}
