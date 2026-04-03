use std::num::NonZeroUsize;

use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

#[trait_variant::make(Send)]
pub trait StatusListServices: StatusListRevocationService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn obtain_status_claims(
        &self,
        attestation_type: &str,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error>;
}

#[trait_variant::make(Send)]
pub trait StatusListService: StatusListRevocationService {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn obtain_status_claims(
        &self,
        batch_id: Uuid,
        expires: Option<DateTimeSeconds>,
        copies: NonZeroUsize,
    ) -> Result<VecNonEmpty<StatusClaim>, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum RevocationError {
    #[error("batch ID not found: {0}")]
    BatchIdNotFound(Uuid),

    #[error("internal error occurred: {0}")]
    InternalError(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for RevocationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RevocationError::BatchIdNotFound(batch_id) => {
                tracing::info!("revocation batch ID not found: {}", batch_id);
                (axum::http::StatusCode::NOT_FOUND, axum::Json(batch_id)).into_response()
            }
            _ => {
                tracing::error!("revocation error: {:?}", self);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[cfg_attr(feature = "axum", derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema))]
pub struct BatchIsRevoked {
    pub batch_id: Uuid,
    pub is_revoked: bool,
}

#[trait_variant::make(Send)]
pub trait StatusListRevocationService {
    async fn republish_all(&self, force: bool) -> Result<(), RevocationError>;
    async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError>;

    async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError>;
    async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use mockall::mock;
    use url::Url;
    use uuid::Uuid;

    use attestation_types::status_claim::StatusListClaim;

    use super::*;

    pub fn generate_status_claims(uri: &Url, copies: NonZeroUsize) -> VecNonEmpty<StatusClaim> {
        (0..copies.get())
            .map(|k| {
                StatusClaim::StatusList(StatusListClaim {
                    idx: k as u32,
                    uri: uri.clone(),
                })
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    mock! {
        pub StatusListService {}

        impl StatusListService for StatusListService {
            type Error = Infallible;

            async fn obtain_status_claims(
                &self,
                batch_id: Uuid,
                expires: Option<DateTimeSeconds>,
                copies: NonZeroUsize,
            ) -> Result<VecNonEmpty<StatusClaim>, Infallible>;
        }

        impl StatusListRevocationService for StatusListService {
            async fn republish_all(&self, force: bool) -> Result<(), RevocationError>;
            async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError>;

            async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError>;
            async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError>;
        }
    }

    mock! {
        pub StatusListServices {}

        impl StatusListServices for StatusListServices {
            type Error = Infallible;

            async fn obtain_status_claims(
                &self,
                attestation_type: &str,
                batch_id: Uuid,
                expires: Option<DateTimeSeconds>,
                copies: NonZeroUsize,
            ) -> Result<VecNonEmpty<StatusClaim>, Infallible>;
        }

        impl StatusListRevocationService for StatusListServices {
            async fn republish_all(&self, force: bool) -> Result<(), RevocationError>;
            async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError>;

            async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError>;
            async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError>;
        }
    }
}
