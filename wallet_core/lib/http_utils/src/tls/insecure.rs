use derive_more::Constructor;
use reqwest::ClientBuilder;

use crate::reqwest::IntoPinnedReqwestClient;
use crate::reqwest::PinnedReqwestClient;
use crate::reqwest::default_reqwest_client_builder;
use crate::urls::BaseUrl;

#[derive(Debug, Clone, Hash, Constructor)]
pub struct InsecureHttpConfig {
    pub base_url: BaseUrl,
}

impl IntoPinnedReqwestClient for InsecureHttpConfig {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        let client = builder_adapter(default_reqwest_client_builder()).build()?;
        let pinned_client = PinnedReqwestClient::new(client, self.base_url);

        Ok(pinned_client)
    }
}
