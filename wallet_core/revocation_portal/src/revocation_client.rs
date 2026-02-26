use chrono::DateTime;
use chrono::Utc;
use derive_more::From;
use serde::Deserialize;

use http_utils::client::HttpServiceConfig;
use http_utils::reqwest::IntoPinnedReqwestClient;
use http_utils::reqwest::PinnedReqwestClient;
use http_utils::reqwest::ReqwestClientUrl;

use crate::DeletionCode;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("failed to revoke wallet")]
    RevocationFailed,

    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
}

#[derive(Debug, Clone, From, Deserialize)]
pub struct RevocationResult {
    pub revoked_at: DateTime<Utc>,
}

#[trait_variant::make(Send)]
pub trait RevocationClient {
    async fn revoke(&self, deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError>;
}

#[derive(Debug, Clone)]
pub struct HttpRevocationClient {
    http_client: PinnedReqwestClient,
}

impl HttpRevocationClient {
    pub fn new(http_config: HttpServiceConfig) -> Result<Self, reqwest::Error> {
        let http_client = http_config.try_into_json_client()?;

        Ok(Self { http_client })
    }
}

impl RevocationClient for HttpRevocationClient {
    async fn revoke(&self, deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError> {
        let response = self
            .http_client
            .send_custom_post(
                ReqwestClientUrl::Relative("/revoke-wallet-by-revocation-code/"),
                |request| request.json(&deletion_code),
            )
            .await?;

        let result = response.error_for_status()?.json().await?;

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Clone, Default)]
    pub struct MockRevocationClient {
        should_fail: bool,
        revoked_at: Option<DateTime<Utc>>,
    }

    impl MockRevocationClient {
        pub fn new_failing() -> Self {
            Self {
                should_fail: true,
                revoked_at: None,
            }
        }

        pub fn new_with_fixed_revoked_at(revoked_at: DateTime<Utc>) -> Self {
            Self {
                should_fail: false,
                revoked_at: Some(revoked_at),
            }
        }
    }

    impl RevocationClient for MockRevocationClient {
        async fn revoke(&self, _deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError> {
            if self.should_fail {
                Err(RevocationError::RevocationFailed)
            } else if let Some(revoked_at) = self.revoked_at {
                Ok(revoked_at.into())
            } else {
                Ok(Utc::now().into())
            }
        }
    }
}
