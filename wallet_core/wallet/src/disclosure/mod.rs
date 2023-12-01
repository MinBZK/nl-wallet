mod uri;

use async_trait::async_trait;
use url::Url;

use nl_wallet_mdoc::{
    holder::{
        CborHttpClient, DisclosureMissingAttributes, DisclosureProposal, DisclosureSession, MdocDataSource,
        ProposedAttributes, TrustAnchor,
    },
    identifiers::AttributeIdentifier,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
    },
};

use crate::utils;

pub use self::uri::{DisclosureUriData, DisclosureUriError};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::{MockMdocDisclosureProposal, MockMdocDisclosureSession};

#[derive(Debug)]
pub enum MdocDisclosureSessionState<M, P> {
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
    fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal>;

    async fn terminate(self) -> nl_wallet_mdoc::Result<()>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier];
}

#[async_trait]
pub trait MdocDisclosureProposal {
    fn return_url(&self) -> Option<&Url>;
    fn proposed_attributes(&self) -> ProposedAttributes;

    async fn disclose<'a, KF, K>(&self, key_factory: &'a KF) -> nl_wallet_mdoc::Result<()>
    where
        KF: KeyFactory<'a, Key = K> + Send + Sync,
        K: MdocEcdsaKey + Send + Sync;
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
            disclosure_uri.session_type,
            mdoc_data_source,
            trust_anchors,
        )
        .await
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_state(
        &self,
    ) -> MdocDisclosureSessionState<&DisclosureMissingAttributes<CborHttpClient>, &DisclosureProposal<CborHttpClient>>
    {
        match self {
            DisclosureSession::MissingAttributes(session) => MdocDisclosureSessionState::MissingAttributes(session),
            DisclosureSession::Proposal(session) => MdocDisclosureSessionState::Proposal(session),
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

#[async_trait]
impl MdocDisclosureProposal for DisclosureProposal<CborHttpClient> {
    fn return_url(&self) -> Option<&Url> {
        self.return_url()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }

    async fn disclose<'a, KF, K>(&self, key_factory: &'a KF) -> nl_wallet_mdoc::Result<()>
    where
        KF: KeyFactory<'a, Key = K> + Send + Sync,
        K: MdocEcdsaKey + Send + Sync,
    {
        self.disclose(key_factory).await
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    };

    use nl_wallet_mdoc::verifier::SessionType;
    use once_cell::sync::Lazy;

    use super::*;

    type SessionState = MdocDisclosureSessionState<MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal>;
    type MockFields = (ReaderRegistration, SessionState);

    pub static NEXT_START_ERROR: Lazy<Mutex<Option<nl_wallet_mdoc::Error>>> = Lazy::new(|| Mutex::new(None));
    pub static NEXT_MOCK_FIELDS: Lazy<Mutex<Option<MockFields>>> = Lazy::new(|| Mutex::new(None));

    // For testing, provide a default for `DisclosureUriData`.
    impl Default for DisclosureUriData {
        fn default() -> Self {
            DisclosureUriData {
                reader_engagement_bytes: Default::default(),
                return_url: None,
                session_type: SessionType::CrossDevice,
            }
        }
    }

    // For convenience, the default `SessionState` is a proposal.
    impl Default for SessionState {
        fn default() -> Self {
            MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal::default())
        }
    }

    #[derive(Debug, Default)]
    pub struct MockMdocDisclosureProposal {
        pub return_url: Option<Url>,
        pub proposed_attributes: ProposedAttributes,
        pub disclosure_count: Arc<AtomicUsize>,
        pub next_error: Mutex<Option<nl_wallet_mdoc::Error>>,
    }

    #[async_trait]
    impl MdocDisclosureProposal for MockMdocDisclosureProposal {
        fn return_url(&self) -> Option<&Url> {
            self.return_url.as_ref()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }

        async fn disclose<'a, KF, K>(&self, _key_factory: &'a KF) -> nl_wallet_mdoc::Result<()> {
            if let Some(error) = self.next_error.lock().unwrap().take() {
                return Err(error);
            }

            self.disclosure_count
                .store(self.disclosure_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);

            Ok(())
        }
    }

    #[derive(Debug, Default)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUriData,
        pub reader_registration: ReaderRegistration,
        pub session_state: SessionState,
        pub was_terminated: Arc<AtomicBool>,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(reader_registration: ReaderRegistration, session_state: SessionState) {
            NEXT_MOCK_FIELDS
                .lock()
                .unwrap()
                .replace((reader_registration, session_state));
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

            let (reader_registration, session_state) = NEXT_MOCK_FIELDS.lock().unwrap().take().unwrap_or_default();

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_registration,
                session_state,
                was_terminated: Default::default(),
            };

            Ok(session)
        }

        fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal> {
            match self.session_state {
                MdocDisclosureSessionState::MissingAttributes(ref session) => {
                    MdocDisclosureSessionState::MissingAttributes(session)
                }
                MdocDisclosureSessionState::Proposal(ref session) => MdocDisclosureSessionState::Proposal(session),
            }
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
            self.was_terminated.store(true, Ordering::Relaxed);

            Ok(())
        }
    }
}
