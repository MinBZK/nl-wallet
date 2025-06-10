use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use reqwest::ClientBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::reqwest::tls_pinned_client_builder;
use crate::reqwest::IntoPinnedReqwestClient;
use crate::reqwest::PinnedReqwestClient;
use crate::reqwest::ReqwestTrustAnchor;
use crate::urls::BaseUrl;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TlsPinningConfigHash(u64);

#[serde_as]
#[derive(derive_more::Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TlsPinningConfig {
    pub base_url: BaseUrl,
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    pub trust_anchors: Vec<ReqwestTrustAnchor>,
}

impl TlsPinningConfig {
    pub fn to_hash(&self) -> TlsPinningConfigHash {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        TlsPinningConfigHash(hasher.finish())
    }
}

impl IntoPinnedReqwestClient for TlsPinningConfig {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        let certificates = self
            .trust_anchors
            .into_iter()
            .map(ReqwestTrustAnchor::into_certificate)
            .collect();
        let client = builder_adapter(tls_pinned_client_builder(certificates)).build()?;
        let pinned_client = PinnedReqwestClient::new(client, self.base_url);

        Ok(pinned_client)
    }
}
