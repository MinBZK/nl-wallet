use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use reqwest::ClientBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::reqwest::IntoPinnedReqwestClient;
use crate::reqwest::PinnedReqwestClient;
use crate::reqwest::ReqwestTrustAnchor;
use crate::reqwest::default_reqwest_client_builder;
use crate::reqwest::tls_pinned_client_builder;
use crate::urls::BaseUrl;

#[derive(Debug, thiserror::Error)]
pub enum HttpConfigError {
    #[error("base_url must use the http scheme, not https")]
    NotHttp,
    #[error("base_url must use the https scheme")]
    NotHttps,
}

#[derive(Deserialize)]
struct InternalHttpConfigRaw {
    base_url: BaseUrl,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "InternalHttpConfigRaw")]
pub struct InternalHttpConfig {
    base_url: BaseUrl,
}

impl InternalHttpConfig {
    pub fn try_new(base_url: BaseUrl) -> Result<Self, HttpConfigError> {
        if base_url.is_https() {
            return Err(HttpConfigError::NotHttp);
        }
        Ok(Self { base_url })
    }

    pub fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }
}

impl TryFrom<InternalHttpConfigRaw> for InternalHttpConfig {
    type Error = HttpConfigError;

    fn try_from(raw: InternalHttpConfigRaw) -> Result<Self, Self::Error> {
        Self::try_new(raw.base_url)
    }
}

impl IntoPinnedReqwestClient for InternalHttpConfig {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        let client = builder_adapter(default_reqwest_client_builder()).build()?;
        let pinned_client = PinnedReqwestClient::new(client, self.base_url);

        Ok(pinned_client)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TlsPinningConfigHash(u64);

#[serde_as]
#[derive(Deserialize)]
struct TlsPinningConfigRaw {
    base_url: BaseUrl,
    #[serde_as(as = "Vec<Base64>")]
    trust_anchors: Vec<ReqwestTrustAnchor>,
}

#[serde_as]
#[derive(derive_more::Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(try_from = "TlsPinningConfigRaw")]
pub struct TlsPinningConfig {
    base_url: BaseUrl,
    #[debug(skip)]
    #[serde_as(as = "Vec<Base64>")]
    trust_anchors: Vec<ReqwestTrustAnchor>,
}

impl TlsPinningConfig {
    pub fn try_new(base_url: BaseUrl, trust_anchors: Vec<ReqwestTrustAnchor>) -> Result<Self, HttpConfigError> {
        if !base_url.is_https() {
            return Err(HttpConfigError::NotHttps);
        }
        Ok(Self {
            base_url,
            trust_anchors,
        })
    }

    pub fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }

    pub fn trust_anchors(&self) -> &[ReqwestTrustAnchor] {
        &self.trust_anchors
    }

    pub fn to_hash(&self) -> TlsPinningConfigHash {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        TlsPinningConfigHash(hasher.finish())
    }
}

impl TryFrom<TlsPinningConfigRaw> for TlsPinningConfig {
    type Error = HttpConfigError;

    fn try_from(raw: TlsPinningConfigRaw) -> Result<Self, Self::Error> {
        Self::try_new(raw.base_url, raw.trust_anchors)
    }
}

impl IntoPinnedReqwestClient for TlsPinningConfig {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        let certificates = self.trust_anchors.into_iter().map(ReqwestTrustAnchor::into_certificate);
        let client = builder_adapter(tls_pinned_client_builder(certificates)).build()?;
        let pinned_client = PinnedReqwestClient::new(client, self.base_url);

        Ok(pinned_client)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum HttpServiceConfig {
    Pinned(TlsPinningConfig),
    Internal(InternalHttpConfig),
}

impl IntoPinnedReqwestClient for HttpServiceConfig {
    fn try_into_custom_client<F>(self, builder_adapter: F) -> Result<PinnedReqwestClient, reqwest::Error>
    where
        F: FnOnce(ClientBuilder) -> ClientBuilder,
    {
        match self {
            Self::Pinned(c) => c.try_into_custom_client(builder_adapter),
            Self::Internal(c) => c.try_into_custom_client(builder_adapter),
        }
    }
}
