use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::Mutex;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;

#[derive(Debug)]
struct CachedClientState {
    http_client: Arc<PinnedReqwestClient>,
    source_hash: u64,
}

#[derive(Debug)]
pub struct CachedReqwestClient<C> {
    client_state: Mutex<Option<CachedClientState>>,
    _config_type: PhantomData<C>,
}

impl<C> CachedReqwestClient<C> {
    pub fn new() -> Self {
        Self {
            client_state: Mutex::new(None),
            _config_type: PhantomData,
        }
    }

    pub fn get_or_try_init<F>(
        &self,
        http_config: &C,
        build_client: F,
    ) -> Result<Arc<PinnedReqwestClient>, reqwest::Error>
    where
        C: IntoPinnedReqwestClient + Clone + Hash,
        F: FnOnce(C) -> Result<PinnedReqwestClient, reqwest::Error>,
    {
        let http_config_hash = {
            let mut hasher = DefaultHasher::new();
            http_config.hash(&mut hasher);
            hasher.finish()
        };

        let mut client_state = self.client_state.lock();

        let pinned_client = match client_state.as_ref() {
            Some(client_with_hash) if client_with_hash.source_hash == http_config_hash => {
                Arc::clone(&client_with_hash.http_client)
            }
            _ => {
                let http_client = Arc::new(build_client(http_config.clone())?);

                client_state.replace(CachedClientState {
                    http_client: Arc::clone(&http_client),
                    source_hash: http_config_hash,
                });

                http_client
            }
        };

        Ok(pinned_client)
    }
}

impl<C> Default for CachedReqwestClient<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use http_utils::reqwest::IntoPinnedReqwestClient;
    use http_utils::tls::insecure::InsecureHttpConfig;
    use http_utils::urls::BaseUrl;

    use super::CachedReqwestClient;

    #[test]
    fn test_cached_reqwest_client() {
        let base_url1 = BaseUrl::from_str("http://example.com").unwrap();
        let base_url2 = BaseUrl::from_str("http://example2.com").unwrap();

        let cached_client = CachedReqwestClient::new();

        // Calling get_or_try_init() on a new InsecureHttpConfig should initialize a new client.
        let mut builder_called = false;
        let client1a = cached_client
            .get_or_try_init(&InsecureHttpConfig::new(base_url1.clone()), |config| {
                assert_eq!(config.base_url, base_url1);
                builder_called = true;

                config.try_into_client()
            })
            .unwrap();

        assert!(builder_called);

        // Calling get_or_try_init() again with the same BaseUrl should have it re-use the existing client.
        let mut builder_called = false;
        let client1b = cached_client
            .get_or_try_init(&InsecureHttpConfig::new(base_url1.clone()), |config| {
                builder_called = true;

                config.try_into_client()
            })
            .unwrap();

        assert!(!builder_called);
        assert!(Arc::ptr_eq(&client1a, &client1b));

        // Calling get_or_try_init() with a different BaseUrl should cause the client to be re-initialized.
        let mut builder_called = false;
        let client2 = cached_client
            .get_or_try_init(&InsecureHttpConfig::new(base_url2.clone()), |config| {
                assert_eq!(config.base_url, base_url2);
                builder_called = true;

                config.try_into_client()
            })
            .unwrap();

        assert!(builder_called);
        assert!(!Arc::ptr_eq(&client2, &client1a));
        assert!(!Arc::ptr_eq(&client2, &client1b));
    }
}
