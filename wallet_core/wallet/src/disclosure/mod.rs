mod session;

use nl_wallet_mdoc::utils::serialization::CborError;
use url::Url;

pub use session::HttpDisclosureSession;

#[derive(Debug, thiserror::Error)]
pub enum DisclosureSessionError {
    #[error("could not decode cbor: {0}")]
    Cbor(#[from] CborError),
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DisclosureSession {
    fn start(reader_engagement_bytes: &[u8], return_url: Option<Url>) -> Result<Self, DisclosureSessionError>
    where
        Self: Sized;
}
