use crate::config::{Configuration, ConfigurationRepository};

use super::Wallet;

pub type ConfigurationCallback = Box<dyn FnMut(&Configuration) + Send + Sync>;

impl<C, S, K, A, D, P> Wallet<C, S, K, A, D, P>
where
    C: ConfigurationRepository,
{
    pub fn set_config_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&Configuration) + Send + Sync + 'static,
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
