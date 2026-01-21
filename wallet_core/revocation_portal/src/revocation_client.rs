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
