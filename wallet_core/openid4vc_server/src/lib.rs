use serde::Deserialize;
use wallet_common::urls::BaseUrl;

#[cfg(any(feature = "issuance", feature = "disclosure"))]
pub mod log_requests;

#[cfg(any(feature = "issuance", feature = "disclosure", feature = "postgres"))]
pub mod store;

#[cfg(feature = "issuance")]
pub mod issuer;

#[cfg(feature = "disclosure")]
pub mod verifier;

#[cfg(feature = "postgres")]
pub mod entity;

// TODO move
#[derive(Clone, Deserialize)]
pub struct Urls {
    // used by the wallet
    pub public_url: BaseUrl,

    #[cfg(feature = "disclosure")]
    pub universal_link_base_url: BaseUrl,
}
