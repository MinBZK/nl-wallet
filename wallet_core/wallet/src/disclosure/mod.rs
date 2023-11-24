mod uri;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::{
    holder::{
        CborHttpClient, DisclosureMissingAttributes, DisclosureProposal, DisclosureSession, MdocDataSource,
        ProposedAttributes, TrustAnchor,
    },
    identifiers::AttributeIdentifier,
    utils::reader_auth::ReaderRegistration,
    verifier::SessionType,
};

use crate::utils;

pub use self::uri::{DisclosureUriData, DisclosureUriError};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockMdocDisclosureSession;

#[derive(Debug)]
pub enum MdocDisclosureSessionType<M, P> {
    MissingAttributes(M),
    Proposal(P),
}

#[async_trait]
pub trait MdocDisclosureSession<D> {
    type MissingAttributes: MdocDisclosureMissingAttributes;
    type Proposal: MdocDisclosureProposal;

    async fn start<'a>(
        disclosure_uri: DisclosureUriData,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> nl_wallet_mdoc::Result<Self>
    where
        Self: Sized;

    fn reader_registration(&self) -> &ReaderRegistration;
    fn session_type(&self) -> MdocDisclosureSessionType<&Self::MissingAttributes, &Self::Proposal>;

    async fn terminate(self) -> nl_wallet_mdoc::Result<()>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier];
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureProposal {
    #[allow(clippy::needless_lifetimes)]
    fn return_url<'a>(&'a self) -> Option<&'a Url>;
    fn proposed_attributes(&self) -> ProposedAttributes;
}

#[async_trait]
impl<D> MdocDisclosureSession<D> for DisclosureSession<CborHttpClient>
where
    D: MdocDataSource + Sync,
{
    type MissingAttributes = DisclosureMissingAttributes<CborHttpClient>;
    type Proposal = DisclosureProposal<CborHttpClient>;

    async fn start<'a>(
        disclosure_uri: DisclosureUriData,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> nl_wallet_mdoc::Result<Self> {
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

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_type(
        &self,
    ) -> MdocDisclosureSessionType<&DisclosureMissingAttributes<CborHttpClient>, &DisclosureProposal<CborHttpClient>>
    {
        match self {
            DisclosureSession::MissingAttributes(session) => MdocDisclosureSessionType::MissingAttributes(session),
            DisclosureSession::Proposal(session) => MdocDisclosureSessionType::Proposal(session),
        }
    }

    async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
        self.terminate().await
    }
}

impl MdocDisclosureMissingAttributes for DisclosureMissingAttributes<CborHttpClient> {
    fn missing_attributes(&self) -> &[AttributeIdentifier] {
        self.missing_attributes()
    }
}

impl MdocDisclosureProposal for DisclosureProposal<CborHttpClient> {
    fn return_url(&self) -> Option<&Url> {
        self.return_url()
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

    type SessionType = MdocDisclosureSessionType<MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal>;
    type MockFields = (ReaderRegistration, SessionType);

    pub static NEXT_START_ERROR: Lazy<Mutex<Option<nl_wallet_mdoc::Error>>> = Lazy::new(|| Mutex::new(None));
    pub static NEXT_MOCK_FIELDS: Lazy<Mutex<Option<MockFields>>> = Lazy::new(|| Mutex::new(None));

    // For convenience, the default `SessionType` is a proposal.
    impl Default for SessionType {
        fn default() -> Self {
            MdocDisclosureSessionType::Proposal(MockMdocDisclosureProposal::default())
        }
    }

    #[derive(Debug, Default)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUriData,
        pub reader_registration: ReaderRegistration,
        pub session_type: SessionType,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(reader_registration: ReaderRegistration, session_type: SessionType) {
            NEXT_MOCK_FIELDS
                .lock()
                .unwrap()
                .replace((reader_registration, session_type));
        }

        pub fn next_start_error(error: nl_wallet_mdoc::Error) {
            NEXT_START_ERROR.lock().unwrap().replace(error);
        }
    }

    #[async_trait]
    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        type MissingAttributes = MockMdocDisclosureMissingAttributes;
        type Proposal = MockMdocDisclosureProposal;

        async fn start<'a>(
            disclosure_uri: DisclosureUriData,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> nl_wallet_mdoc::Result<Self> {
            if let Some(error) = NEXT_START_ERROR.lock().unwrap().take() {
                return Err(error);
            }

            let (reader_registration, session_type) = NEXT_MOCK_FIELDS.lock().unwrap().take().unwrap_or_default();

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_registration,
                session_type,
            };

            Ok(session)
        }

        fn session_type(&self) -> MdocDisclosureSessionType<&Self::MissingAttributes, &Self::Proposal> {
            match self.session_type {
                MdocDisclosureSessionType::MissingAttributes(ref session) => {
                    MdocDisclosureSessionType::MissingAttributes(session)
                }
                MdocDisclosureSessionType::Proposal(ref session) => MdocDisclosureSessionType::Proposal(session),
            }
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
            Ok(())
        }
    }
}
