mod uri;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::{
    holder::{CborHttpClient, DisclosureSession, MdocDataSource, ProposedAttributes, TrustAnchor},
    identifiers::AttributeIdentifier,
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
    fn has_missing_attributes(&self) -> bool;
    fn missing_attributes(&self) -> Vec<AttributeIdentifier>;
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

    fn has_missing_attributes(&self) -> bool {
        self.has_missing_attributes()
    }

    fn missing_attributes(&self) -> Vec<AttributeIdentifier> {
        self.missing_attributes()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::Mutex;

    use nl_wallet_mdoc::identifiers::AttributeIdentifier;
    use once_cell::sync::Lazy;

    use super::*;

    type MockFields = (ReaderRegistration, Vec<AttributeIdentifier>, ProposedAttributes);

    pub static NEXT_START_ERROR: Lazy<Mutex<Option<nl_wallet_mdoc::Error>>> = Lazy::new(|| Mutex::new(None));
    pub static NEXT_MOCK_FIELDS: Lazy<Mutex<Option<MockFields>>> = Lazy::new(|| Mutex::new(None));

    #[derive(Debug, Default)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUriData,
        pub reader_registration: ReaderRegistration,
        pub missing_attributes: Vec<AttributeIdentifier>,
        pub proposed_attributes: ProposedAttributes,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(
            reader_registration: ReaderRegistration,
            missing_attributes: Vec<AttributeIdentifier>,
            proposed_attributes: ProposedAttributes,
        ) {
            NEXT_MOCK_FIELDS
                .lock()
                .unwrap()
                .replace((reader_registration, missing_attributes, proposed_attributes));
        }

        pub fn next_start_error(error: nl_wallet_mdoc::Error) {
            NEXT_START_ERROR.lock().unwrap().replace(error);
        }
    }

    #[async_trait]
    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        async fn start<'a>(
            disclosure_uri: DisclosureUriData,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> Result<Self, nl_wallet_mdoc::Error> {
            if let Some(error) = NEXT_START_ERROR.lock().unwrap().take() {
                return Err(error);
            }

            let (reader_registration, missing_attributes, proposed_attributes) =
                NEXT_MOCK_FIELDS.lock().unwrap().take().unwrap_or_default();

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_registration,
                missing_attributes,
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

        fn has_missing_attributes(&self) -> bool {
            !self.missing_attributes.is_empty()
        }

        fn missing_attributes(&self) -> Vec<AttributeIdentifier> {
            self.missing_attributes.clone()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }
    }
}
