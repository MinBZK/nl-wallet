use chrono::DateTime;
use chrono::Utc;
use derive_more::From;

use crate::DeletionCode;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("failed to revoke wallet")]
    RevocationFailed,
}

#[derive(Debug, Clone, From)]
pub struct RevocationResult {
    pub revoked_at: DateTime<Utc>,
}

#[trait_variant::make(Send)]
pub trait RevocationClient {
    async fn revoke(&self, deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError>;
}

#[derive(Debug, Clone)]
pub struct HttpRevocationClient {}

impl RevocationClient for HttpRevocationClient {
    async fn revoke(&self, _deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError> {
        // TODO: implement in PVW-5306
        Ok(Utc::now().into())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Clone, Default)]
    pub struct MockRevocationClient {
        should_fail: bool,
    }

    impl MockRevocationClient {
        pub fn new_failing() -> Self {
            Self { should_fail: true }
        }
    }

    impl RevocationClient for MockRevocationClient {
        async fn revoke(&self, _deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError> {
            if self.should_fail {
                Err(RevocationError::RevocationFailed)
            } else {
                Ok(Utc::now().into())
            }
        }
    }
}
