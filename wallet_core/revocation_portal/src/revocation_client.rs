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
