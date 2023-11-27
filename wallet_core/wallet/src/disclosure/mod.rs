mod uri;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, DisclosureSession, MdocDataSource, ProposedAttributes, TrustAnchor},
    utils::reader_auth::ReaderRegistration,
    verifier::SessionType,
};

use crate::utils;

pub use self::uri::{DisclosureUriData, DisclosureUriError};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockMdocDisclosureSession;

#[async_trait]
pub trait MdocDisclosureSession<D> {
    async fn start<'a>(
        disclosure_uri: DisclosureUriData,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, nl_wallet_mdoc::Error>
    where
        Self: Sized;

    fn return_url(&self) -> Option<&Url>;
    fn reader_registration(&self) -> &ReaderRegistration;
    fn proposed_attributes(&self) -> ProposedAttributes;
}

#[async_trait]
impl<D> MdocDisclosureSession<D> for DisclosureSession<CborHttpClient>
where
    D: MdocDataSource + Sync,
{
    async fn start<'a>(
        disclosure_uri: DisclosureUriData,
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
            SessionType::SameDevice, // TODO: Distinguish between same device and cross device flows.
            mdoc_data_source,
            trust_anchors,
        )
        .await
    }

    fn return_url(&self) -> Option<&Url> {
        self.return_url.as_ref()
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        &self.reader_registration
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;

    pub static READER_REGISTRATION_NEXT_START_ERROR: Lazy<Mutex<Option<nl_wallet_mdoc::Error>>> =
        Lazy::new(|| Mutex::new(None));
    pub static READER_REGISTRATION_DISCLOSED_ATTRIBUTES: Lazy<Mutex<Option<(ReaderRegistration, ProposedAttributes)>>> =
        Lazy::new(|| Mutex::new(None));

    #[derive(Debug, Default)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUriData,
        pub reader_registration: ReaderRegistration,
        pub proposed_attributes: ProposedAttributes,
    }

    impl MockMdocDisclosureSession {
        pub fn next_reader_registration_and_proposed_attributes(
            reader_registration: ReaderRegistration,
            proposed_attributes: ProposedAttributes,
        ) {
            READER_REGISTRATION_DISCLOSED_ATTRIBUTES
                .lock()
                .unwrap()
                .replace((reader_registration, proposed_attributes));
        }

        pub fn next_start_error(error: nl_wallet_mdoc::Error) {
            READER_REGISTRATION_NEXT_START_ERROR.lock().unwrap().replace(error);
        }
    }

    #[async_trait]
    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        async fn start<'a>(
            disclosure_uri: DisclosureUriData,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> Result<Self, nl_wallet_mdoc::Error> {
            if let Some(error) = READER_REGISTRATION_NEXT_START_ERROR.lock().unwrap().take() {
                return Err(error);
            }

            let (reader_registration, proposed_attributes) = READER_REGISTRATION_DISCLOSED_ATTRIBUTES
                .lock()
                .unwrap()
                .take()
                .unwrap_or_default();

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_registration,
                proposed_attributes,
            };

            Ok(session)
        }

        fn return_url(&self) -> Option<&Url> {
            self.disclosure_uri.return_url.as_ref()
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }
    }
}
