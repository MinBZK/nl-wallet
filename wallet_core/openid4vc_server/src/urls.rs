use serde::Deserialize;
use wallet_common::urls::BaseUrl;

#[derive(Clone, Deserialize)]
pub struct Urls {
    /// Publically reachable URL used by the wallet during sessions
    pub public_url: BaseUrl,

    #[cfg(feature = "disclosure")]
    pub universal_link_base_url: BaseUrl,
}
