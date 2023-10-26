mod uri;

use nl_wallet_mdoc::holder::DisclosureSession;

pub use self::uri::{DisclosureUri, DisclosureUriError};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureSession {
    fn start(disclosure_uri: DisclosureUri) -> Result<Self, nl_wallet_mdoc::Error>
    where
        Self: Sized;
}

impl MdocDisclosureSession for DisclosureSession {
    fn start(disclosure_uri: DisclosureUri) -> Result<Self, nl_wallet_mdoc::Error> {
        Self::start(&disclosure_uri.reader_engagement_bytes, disclosure_uri.return_url)
    }
}
