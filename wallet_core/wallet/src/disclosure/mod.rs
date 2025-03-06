mod uri;

use rustls_pki_types::TrustAnchor;
use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::holder::MdocDataSource;
use nl_wallet_mdoc::holder::ProposedAttributes;
use nl_wallet_mdoc::identifiers::AttributeIdentifier;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use openid4vc::disclosure_session::DisclosureError;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::disclosure_session::VpClientError;
use openid4vc::verifier::SessionType;
use poa::factory::PoaFactory;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::reqwest::default_reqwest_client_builder;

pub use openid4vc::disclosure_session::DisclosureUriSource;

pub use self::uri::DisclosureUriError;
pub use self::uri::VpDisclosureUriData;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockMdocDisclosureProposal;
#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockMdocDisclosureSession;

pub type DisclosureResult<T, E> = std::result::Result<T, DisclosureError<E>>;

#[derive(Debug)]
pub enum MdocDisclosureSessionState<M, P> {
    MissingAttributes(M),
    Proposal(P),
}

#[derive(thiserror::Error, Debug)]
pub enum MdocDisclosureError {
    #[error("error in OpenID4VP disclosure session: {0}")]
    Vp(#[from] VpClientError),
}

pub trait MdocDisclosureSession<D> {
    type MissingAttributes: MdocDisclosureMissingAttributes;
    type Proposal: MdocDisclosureProposal;
    type DisclosureUriData;

    fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError>;

    async fn start(
        disclosure_uri: Self::DisclosureUriData,
        disclosure_uri_source: DisclosureUriSource,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, MdocDisclosureError>
    where
        Self: Sized;

    fn rp_certificate(&self) -> &BorrowingCertificate;
    fn reader_registration(&self) -> &ReaderRegistration;
    fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal>;
    fn session_type(&self) -> SessionType;

    async fn terminate(self) -> Result<Option<Url>, MdocDisclosureError>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier];
}

pub trait MdocDisclosureProposal {
    fn proposed_source_identifiers(&self) -> Vec<Uuid>;
    fn proposed_attributes(&self) -> ProposedAttributes;

    async fn disclose<K, KF>(&self, key_factory: &KF) -> DisclosureResult<Option<Url>, MdocDisclosureError>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
        KF: PoaFactory<Key = K>;
}

type VpDisclosureSession = openid4vc::disclosure_session::DisclosureSession<HttpVpMessageClient, Uuid>;
type VpDisclosureMissingAttributes = openid4vc::disclosure_session::DisclosureMissingAttributes<HttpVpMessageClient>;
type VpDisclosureProposal = openid4vc::disclosure_session::DisclosureProposal<HttpVpMessageClient, Uuid>;

impl<D> MdocDisclosureSession<D> for VpDisclosureSession
where
    D: MdocDataSource<MdocIdentifier = Uuid>,
{
    type MissingAttributes = VpDisclosureMissingAttributes;
    type Proposal = VpDisclosureProposal;
    type DisclosureUriData = VpDisclosureUriData;

    fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError> {
        VpDisclosureUriData::parse_from_uri(uri, base_uri)
    }

    async fn start(
        disclosure_uri: VpDisclosureUriData,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, MdocDisclosureError>
    where
        Self: Sized,
    {
        let client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client")
            .into();
        let session = Self::start(
            client,
            &disclosure_uri.query,
            uri_source,
            mdoc_data_source,
            trust_anchors,
        )
        .await?;

        Ok(session)
    }

    fn rp_certificate(&self) -> &BorrowingCertificate {
        self.verifier_certificate()
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_state(&self) -> MdocDisclosureSessionState<&VpDisclosureMissingAttributes, &VpDisclosureProposal> {
        match self {
            Self::MissingAttributes(session) => MdocDisclosureSessionState::MissingAttributes(session),
            Self::Proposal(session) => MdocDisclosureSessionState::Proposal(session),
        }
    }

    fn session_type(&self) -> SessionType {
        self.session_type()
    }

    async fn terminate(self) -> Result<Option<Url>, MdocDisclosureError> {
        let return_url = self.terminate().await?.map(|url| url.into_inner());

        Ok(return_url)
    }
}

impl MdocDisclosureMissingAttributes for VpDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier] {
        self.missing_attributes()
    }
}

impl MdocDisclosureProposal for VpDisclosureProposal {
    fn proposed_source_identifiers(&self) -> Vec<Uuid> {
        self.proposed_source_identifiers().into_iter().copied().collect()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }

    async fn disclose<K, KF>(&self, key_factory: &KF) -> DisclosureResult<Option<Url>, MdocDisclosureError>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
        KF: PoaFactory<Key = K>,
    {
        let redirect_uri = self
            .disclose(key_factory)
            .await
            .map_err(|err| DisclosureError::new(err.data_shared, err.error.into()))?;
        Ok(redirect_uri.map(|u| u.into_inner()))
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use parking_lot::Mutex;

    use nl_wallet_mdoc::server_keys::generate::Ca;
    use nl_wallet_mdoc::server_keys::KeyPair;

    use super::*;

    type SessionState = MdocDisclosureSessionState<MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal>;
    type MockFields = (ReaderRegistration, SessionState, Option<Url>);

    pub static NEXT_START_ERROR: Mutex<Option<MdocDisclosureError>> = Mutex::new(None);
    pub static NEXT_MOCK_FIELDS: Mutex<Option<MockFields>> = Mutex::new(None);

    // For convenience, the default `SessionState` is a proposal.
    impl Default for SessionState {
        fn default() -> Self {
            MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal::default())
        }
    }

    #[derive(Debug)]
    pub struct MockMdocDisclosureProposal {
        pub disclose_return_url: Option<Url>,
        pub proposed_source_identifiers: Vec<Uuid>,
        pub proposed_attributes: ProposedAttributes,
        pub disclosure_count: Arc<AtomicUsize>,
        pub next_error: Mutex<Option<MdocDisclosureError>>,
        pub attributes_shared: bool,
        pub session_type: SessionType,
    }

    impl Default for MockMdocDisclosureProposal {
        fn default() -> Self {
            Self {
                disclose_return_url: Default::default(),
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
        fn proposed_source_identifiers(&self) -> Vec<Uuid> {
            self.proposed_source_identifiers.clone()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }

        async fn disclose<K, KF>(&self, _key_factory: &KF) -> DisclosureResult<Option<Url>, MdocDisclosureError> {
            if let Some(error) = self.next_error.lock().take() {
                return Err(DisclosureError::new(self.attributes_shared, error));
            }

            self.disclosure_count
                .store(self.disclosure_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);

            Ok(self.disclose_return_url.clone())
        }
    }

    #[derive(Debug)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri_source: DisclosureUriSource,
        pub certificate: BorrowingCertificate,
        pub reader_registration: ReaderRegistration,
        pub session_state: SessionState,
        pub was_terminated: Arc<AtomicBool>,
        pub session_type: SessionType,
        pub terminate_return_url: Option<Url>,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(
            reader_registration: ReaderRegistration,
            session_state: SessionState,
            terminate_return_url: Option<Url>,
        ) {
            NEXT_MOCK_FIELDS
                .lock()
                .replace((reader_registration, session_state, terminate_return_url));
        }

        pub fn next_start_error(error: MdocDisclosureError) {
            NEXT_START_ERROR.lock().replace(error);
        }
    }

    /// The reader key, generated once for testing.
    static READER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let reader_ca = Ca::generate_reader_mock_ca().unwrap();
        reader_ca
            .generate_reader_mock(ReaderRegistration::new_mock().into())
            .unwrap()
    });

    impl Default for MockMdocDisclosureSession {
        fn default() -> Self {
            Self {
                disclosure_uri_source: DisclosureUriSource::Link,
                certificate: READER_KEY.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                session_state: Default::default(),
                was_terminated: Default::default(),
                session_type: SessionType::SameDevice,
                terminate_return_url: Default::default(),
            }
        }
    }

    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        type MissingAttributes = MockMdocDisclosureMissingAttributes;
        type Proposal = MockMdocDisclosureProposal;
        type DisclosureUriData = ();

        fn parse_url(uri: &Url, _base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError> {
            if uri.query_pairs().any(|(param, _)| param == "invalid") {
                Err(DisclosureUriError::Malformed(uri.clone()))
            } else {
                Ok(())
            }
        }

        async fn start(
            _disclosure_uri: Self::DisclosureUriData,
            disclosure_uri_source: DisclosureUriSource,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'_>],
        ) -> Result<Self, MdocDisclosureError> {
            if let Some(error) = NEXT_START_ERROR.lock().take() {
                Err(error)?;
            }

            let (reader_registration, session_state, terminate_return_url) = NEXT_MOCK_FIELDS
                .lock()
                .take()
                .unwrap_or_else(|| (ReaderRegistration::new_mock(), SessionState::default(), None));

            let session = MockMdocDisclosureSession {
                disclosure_uri_source,
                reader_registration,
                session_state,
                terminate_return_url,
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

        async fn terminate(self) -> Result<Option<Url>, MdocDisclosureError> {
            self.was_terminated.store(true, Ordering::Relaxed);

            Ok(self.terminate_return_url)
        }

        fn rp_certificate(&self) -> &BorrowingCertificate {
            &self.certificate
        }

        fn session_type(&self) -> SessionType {
            self.session_type
        }
    }
}
