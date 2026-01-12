use std::collections::HashMap;
use std::time::Duration;

use derive_more::AsRef;
use derive_more::From;
use derive_more::IntoIterator;

use crypto::server_keys::KeyPair;
use http_utils::urls::BaseUrl;
use utils::num::NonZeroU31;
use utils::num::U31;

use crate::publish::PublishDir;

#[derive(Debug, Clone)]
pub struct StatusListConfig<K> {
    pub list_size: NonZeroU31,
    pub create_threshold: U31,
    pub expiry: Duration,
    pub refresh_threshold: Duration,
    pub ttl: Option<Duration>,

    pub base_url: BaseUrl,
    pub publish_dir: PublishDir,
    pub key_pair: KeyPair<K>,
}

#[derive(Debug, Clone, From, IntoIterator, AsRef)]
pub struct StatusListConfigs<K>(HashMap<String, StatusListConfig<K>>);

impl<K> StatusListConfigs<K> {
    pub fn types(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}
