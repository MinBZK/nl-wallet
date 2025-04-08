mod background_repository;
mod http_repository;
#[cfg(any(test, feature = "mock"))]
mod mock;

use background_repository::BackgroundUpdateableUpdatePolicyRepository;

use error_category::ErrorCategory;
use http_utils::http::client::TlsPinningConfig;

use crate::repository::FileStorageError;
use crate::repository::HttpClientError;

pub use self::http_repository::HttpUpdatePolicyRepository;
#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockUpdatePolicyRepository;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum UpdatePolicyError {
    #[error("could not join background task: {0}")]
    #[category(critical)]
    Join(#[from] tokio::task::JoinError),
    #[error("http client error: {0}")]
    HttpClient(#[from] HttpClientError),
    #[error("could not store or load etag file: {0}")]
    FileStorage(#[from] FileStorageError),
}

pub type UpdatePolicyRepository =
    BackgroundUpdateableUpdatePolicyRepository<HttpUpdatePolicyRepository, TlsPinningConfig>;
