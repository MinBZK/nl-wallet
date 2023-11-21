use std::sync::atomic::{AtomicBool, Ordering};
use wallet_common::config::wallet_config::WalletConfiguration;

use crate::config::ConfigurationRepository;

pub struct PreloadConfigurationRepository {
    loaded: AtomicBool,
    preload: Box<dyn ConfigurationRepository + Send + Sync>,
    primary: Box<dyn ConfigurationRepository + Send + Sync>,
}

impl PreloadConfigurationRepository {
    pub fn new(
        primary: impl ConfigurationRepository + Send + Sync + 'static,
        preload: impl ConfigurationRepository + Send + Sync + 'static,
    ) -> Self {
        Self {
            loaded: AtomicBool::new(false),
            preload: Box::new(preload),
            primary: Box::new(primary),
        }
    }
}

impl ConfigurationRepository for PreloadConfigurationRepository {
    fn config(&self) -> &WalletConfiguration {
        if self.loaded.load(Ordering::Relaxed) {
            self.primary.config()
        } else {
            self.loaded.store(true, Ordering::Relaxed);
            self.preload.config()
        }
    }
}
