use crate::DeletionCode;

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("failed to revoke wallet")]
    RevocationFailed,
}

#[trait_variant::make(Send)]
pub trait RevocationClient {
    async fn revoke(&self, deletion_code: DeletionCode) -> Result<(), RevocationError>;
}

#[derive(Debug, Clone)]
pub struct HttpRevocationClient {}

impl RevocationClient for HttpRevocationClient {
    async fn revoke(&self, _deletion_code: DeletionCode) -> Result<(), RevocationError> {
        // TODO: implement in PVW-5306
        Ok(())
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
        async fn revoke(&self, _deletion_code: DeletionCode) -> Result<(), RevocationError> {
            if self.should_fail {
                Err(RevocationError::RevocationFailed)
            } else {
                Ok(())
            }
        }
    }
}
