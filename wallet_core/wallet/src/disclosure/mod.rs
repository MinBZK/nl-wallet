mod uri;

use async_trait::async_trait;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, DisclosureSession, MdocDataSource, PropsedAttributes, TrustAnchor},
    utils::reader_auth::ReaderRegistration,
};

use crate::utils;

pub use self::uri::{DisclosureUri, DisclosureUriError};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockMdocDisclosureSession;

#[async_trait]
pub trait MdocDisclosureSession<D> {
    async fn start<'a>(
        disclosure_uri: DisclosureUri,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, nl_wallet_mdoc::Error>
    where
        Self: Sized;

    fn reader_registration(&self) -> &ReaderRegistration;
    fn proposed_attributes(&self) -> PropsedAttributes;
}

#[async_trait]
impl<D> MdocDisclosureSession<D> for DisclosureSession<CborHttpClient>
where
    D: MdocDataSource + Sync,
{
    async fn start<'a>(
        disclosure_uri: DisclosureUri,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, nl_wallet_mdoc::Error> {
        let http_client = utils::reqwest::default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        Self::start(
            CborHttpClient(http_client),
            &disclosure_uri.reader_engagement_bytes,
            disclosure_uri.return_url,
            mdoc_data_source,
            trust_anchors,
        )
        .await
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn proposed_attributes(&self) -> PropsedAttributes {
        self.proposed_attributes()
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use super::*;

    #[derive(Debug)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUri,
        pub reader_registration: ReaderRegistration,
        pub proposed_attributes: PropsedAttributes,
    }

    impl Default for MockMdocDisclosureSession {
        fn default() -> Self {
            MockMdocDisclosureSession {
                disclosure_uri: DisclosureUri {
                    reader_engagement_bytes: Default::default(),
                    return_url: Default::default(),
                },
                reader_registration: Default::default(),
                proposed_attributes: Default::default(),
            }
        }
    }

    #[async_trait]
    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        async fn start<'a>(
            disclosure_uri: DisclosureUri,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> Result<Self, nl_wallet_mdoc::Error> {
            let session = MockMdocDisclosureSession {
                disclosure_uri,
                ..Default::default()
            };

            Ok(session)
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        fn proposed_attributes(&self) -> PropsedAttributes {
            self.proposed_attributes.clone()
        }
    }
}
