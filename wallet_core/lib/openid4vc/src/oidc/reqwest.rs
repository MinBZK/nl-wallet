use derive_more::AsRef;

use http_utils::reqwest::IntoReqwestClient;
use http_utils::reqwest::ReqwestClient;

#[derive(Debug, Clone, AsRef)]
pub struct OidcReqwestClient(ReqwestClient);

impl OidcReqwestClient {
    pub fn try_new<C>(client_source: C) -> Result<Self, reqwest::Error>
    where
        C: IntoReqwestClient,
    {
        let client = client_source.try_into_json_client()?;

        Ok(Self(client))
    }
}
