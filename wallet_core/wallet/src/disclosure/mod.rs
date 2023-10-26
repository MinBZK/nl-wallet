mod uri;

use async_trait::async_trait;

use nl_wallet_mdoc::holder::{CborHttpClient, DisclosureSession, TrustAnchor};

use crate::utils;

pub use self::uri::{DisclosureUri, DisclosureUriError};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait MdocDisclosureSession {
    async fn start<'a>(
        disclosure_uri: DisclosureUri,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, nl_wallet_mdoc::Error>
    where
        Self: Sized;
}

#[async_trait]
impl MdocDisclosureSession for DisclosureSession<CborHttpClient> {
    async fn start<'a>(
        disclosure_uri: DisclosureUri,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, nl_wallet_mdoc::Error> {
        let http_client = utils::reqwest::default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        Self::start(
            CborHttpClient(http_client),
            &disclosure_uri.reader_engagement_bytes,
            disclosure_uri.return_url,
            trust_anchors,
        )
        .await
    }
}
