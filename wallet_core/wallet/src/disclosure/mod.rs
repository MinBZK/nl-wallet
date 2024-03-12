mod uri;

use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{
        CborHttpClient, DisclosureMissingAttributes, DisclosureProposal, DisclosureResult, DisclosureSession,
        MdocDataSource, ProposedAttributes, TrustAnchor,
    },
    identifiers::AttributeIdentifier,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        x509::Certificate,
    },
    verifier::SessionType,
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

    fn rp_certificate(&self) -> &Certificate;
    fn reader_registration(&self) -> &ReaderRegistration;
    fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal>;
    fn session_type(&self) -> SessionType;

    async fn terminate(self) -> nl_wallet_mdoc::Result<()>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier];
}

pub trait MdocDisclosureProposal {
    fn return_url(&self) -> Option<&Url>;
    fn proposed_source_identifiers(&self) -> Vec<Uuid>;
    fn proposed_attributes(&self) -> ProposedAttributes;

    async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<()>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey;
}

impl<D> MdocDisclosureSession<D> for DisclosureSession<CborHttpClient, Uuid>
where
    D: MdocDataSource<MdocIdentifier = Uuid>,
{
    type MissingAttributes = DisclosureMissingAttributes<CborHttpClient>;
    type Proposal = DisclosureProposal<CborHttpClient, Uuid>;

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

    fn rp_certificate(&self) -> &Certificate {
        self.verifier_certificate()
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_state(
        &self,
    ) -> MdocDisclosureSessionState<
        &DisclosureMissingAttributes<CborHttpClient>,
        &DisclosureProposal<CborHttpClient, Uuid>,
    > {
        match self {
            DisclosureSession::MissingAttributes(session) => MdocDisclosureSessionState::MissingAttributes(session),
            DisclosureSession::Proposal(session) => MdocDisclosureSessionState::Proposal(session),
        }
    }

    async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
        self.terminate().await
    }

    fn session_type(&self) -> SessionType {
        self.session_type()
    }
}

impl MdocDisclosureMissingAttributes for DisclosureMissingAttributes<CborHttpClient> {
    fn missing_attributes(&self) -> &[AttributeIdentifier] {
        self.missing_attributes()
    }
}

impl MdocDisclosureProposal for DisclosureProposal<CborHttpClient, Uuid> {
    fn return_url(&self) -> Option<&Url> {
        self.return_url()
    }

    fn proposed_source_identifiers(&self) -> Vec<Uuid> {
        self.proposed_source_identifiers().into_iter().copied().collect()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }

    async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<()>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        self.disclose(key_factory).await
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    use once_cell::sync::Lazy;
    use parking_lot::Mutex;

    use nl_wallet_mdoc::{holder::DisclosureError, server_keys::KeyPair, verifier::SessionType};

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

    #[derive(Debug)]
    pub struct MockMdocDisclosureProposal {
        pub return_url: Option<Url>,
        pub proposed_source_identifiers: Vec<Uuid>,
        pub proposed_attributes: ProposedAttributes,
        pub disclosure_count: Arc<AtomicUsize>,
        pub next_error: Mutex<Option<nl_wallet_mdoc::Error>>,
        pub attributes_shared: bool,
        pub session_type: SessionType,
    }

    impl Default for MockMdocDisclosureProposal {
        fn default() -> Self {
            Self {
                return_url: Default::default(),
                proposed_source_identifiers: Default::default(),
                proposed_attributes: Default::default(),
                disclosure_count: Default::default(),
                next_error: Default::default(),
                attributes_shared: Default::default(),
                session_type: SessionType::SameDevice,
            }
        }
    }

    impl MdocDisclosureProposal for MockMdocDisclosureProposal {
        fn return_url(&self) -> Option<&Url> {
            self.return_url.as_ref()
        }

        fn proposed_source_identifiers(&self) -> Vec<Uuid> {
            self.proposed_source_identifiers.clone()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }

        async fn disclose<KF, K>(&self, _key_factory: &KF) -> DisclosureResult<()>
        where
            KF: KeyFactory<Key = K>,
            K: MdocEcdsaKey,
        {
            if let Some(error) = self.next_error.lock().take() {
                return Err(DisclosureError::new(self.attributes_shared, error));
            }

            self.disclosure_count
                .store(self.disclosure_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: DisclosureUriData,
        pub certificate: Certificate,
        pub reader_registration: ReaderRegistration,
        pub session_state: SessionState,
        pub was_terminated: Arc<AtomicBool>,
        pub session_type: SessionType,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(reader_registration: ReaderRegistration, session_state: SessionState) {
            NEXT_MOCK_FIELDS.lock().replace((reader_registration, session_state));
        }

        pub fn next_start_error(error: nl_wallet_mdoc::Error) {
            NEXT_START_ERROR.lock().replace(error);
        }
    }

    /// The reader key, generated once for testing.
    static READER_KEY: Lazy<KeyPair> = Lazy::new(|| {
        let reader_ca = KeyPair::generate_reader_mock_ca().unwrap();
        reader_ca
            .generate_reader_mock(ReaderRegistration::new_mock().into())
            .unwrap()
    });

    impl Default for MockMdocDisclosureSession {
        fn default() -> Self {
            Self {
                disclosure_uri: Default::default(),
                certificate: READER_KEY.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                session_state: Default::default(),
                was_terminated: Default::default(),
                session_type: SessionType::SameDevice,
            }
        }
    }

    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        type MissingAttributes = MockMdocDisclosureMissingAttributes;
        type Proposal = MockMdocDisclosureProposal;

        async fn start<'a>(
            disclosure_uri: DisclosureUriData,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> nl_wallet_mdoc::Result<Self> {
            if let Some(error) = NEXT_START_ERROR.lock().take() {
                return Err(error);
            }

            let (reader_registration, session_state) = NEXT_MOCK_FIELDS
                .lock()
                .take()
                .unwrap_or_else(|| (ReaderRegistration::new_mock(), SessionState::default()));

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_registration,
                session_state,
                ..Default::default()
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

        fn rp_certificate(&self) -> &Certificate {
            &self.certificate
        }

        fn session_type(&self) -> SessionType {
            self.session_type
        }
    }
}
