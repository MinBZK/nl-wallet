use std::num::NonZeroUsize;

use uuid::Uuid;

use attestation_types::status_claim::StatusClaim;
use utils::date_time_seconds::DateTimeSeconds;
use utils::vec_at_least::VecNonEmpty;

#[trait_variant::make(Send)]
pub trait StatusListServices {
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
pub trait StatusListService {
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
    #[error("batch IDs not found: {0:?}")]
    BatchIdsNotFound(VecNonEmpty<Uuid>),

    #[error("internal error occurred: {0}")]
    InternalError(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for RevocationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RevocationError::BatchIdsNotFound(batch_ids) => {
                tracing::info!("revocation batch IDs not found: {:?}", batch_ids);
                (axum::http::StatusCode::NOT_FOUND, axum::Json(batch_ids)).into_response()
            }
            _ => {
                tracing::error!("revocation error: {:?}", self);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[trait_variant::make(Send)]
pub trait StatusListRevocationService {
    async fn revoke_attestation_batches(&self, batch_ids: Vec<Uuid>) -> Result<(), RevocationError>;
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    use dashmap::DashMap;
    use url::Url;

    use attestation_types::status_claim::StatusListClaim;
    use http_utils::urls::BaseUrl;

    use super::*;

    // Note: since checking the status is not part of the trait, keeping track of that is not implemented here.
    pub struct MockStatusListServices {
        base_url: BaseUrl,
        index_map: DashMap<String, u32>,
    }

    impl MockStatusListServices {
        pub fn from_base_url(base_url: BaseUrl) -> Self {
            Self {
                base_url,
                index_map: Default::default(),
            }
        }
    }

    impl Default for MockStatusListServices {
        fn default() -> Self {
            MockStatusListServices::from_base_url("https://example.com".parse().unwrap())
        }
    }

    impl StatusListServices for MockStatusListServices {
        type Error = Infallible;

        async fn obtain_status_claims(
            &self,
            attestation_type: &str,
            _batch_id: Uuid,
            _expires: Option<DateTimeSeconds>,
            copies: NonZeroUsize,
        ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
            let copies = copies.get() as u32;
            let url = self.base_url.join(attestation_type.replace(':', "-").as_str());
            let mut entry = self.index_map.entry(attestation_type.to_string()).or_insert(0);
            let start = *entry + 1;
            *entry += copies;
            let claims = (start..=*entry)
                .map(|idx| StatusClaim::StatusList(StatusListClaim { idx, uri: url.clone() }))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Ok(claims)
        }
    }

    impl StatusListRevocationService for MockStatusListServices {
        async fn revoke_attestation_batches(&self, _batch_ids: Vec<Uuid>) -> Result<(), RevocationError> {
            Ok(())
        }
    }

    pub struct MockStatusListService {
        url: Url,
        index: AtomicU32,
    }

    impl MockStatusListService {
        pub fn from_base_url(url: Url) -> Self {
            Self {
                url,
                index: AtomicU32::default(),
            }
        }
    }

    impl Default for MockStatusListService {
        fn default() -> Self {
            MockStatusListService::from_base_url("https://example.com".parse().unwrap())
        }
    }

    impl StatusListService for MockStatusListService {
        type Error = Infallible;

        async fn obtain_status_claims(
            &self,
            _batch_id: Uuid,
            _expires: Option<DateTimeSeconds>,
            copies: NonZeroUsize,
        ) -> Result<VecNonEmpty<StatusClaim>, Self::Error> {
            let copies = copies.get() as u32;
            let start = self.index.fetch_add(copies, Ordering::Relaxed);
            let claims = (0..copies)
                .map(|k| {
                    StatusClaim::StatusList(StatusListClaim {
                        idx: start + k,
                        uri: self.url.clone(),
                    })
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Ok(claims)
        }
    }
}
