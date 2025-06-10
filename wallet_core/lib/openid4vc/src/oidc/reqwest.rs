use derive_more::AsRef;

use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;

#[derive(Debug, Clone, AsRef)]
pub struct OidcReqwestClient(PinnedReqwestClient);

impl OidcReqwestClient {
    pub fn try_new<C>(client_source: C) -> Result<Self, reqwest::Error>
    where
        C: IntoPinnedReqwestClient,
    {
        let client = client_source.try_into_json_client()?;

        Ok(Self(client))
    }
}
