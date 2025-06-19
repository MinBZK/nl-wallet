#[cfg(test)]
pub mod mock {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use parking_lot::Mutex;
    use rustls_pki_types::TrustAnchor;
    use uuid::Uuid;

    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::KeyPair;
    use crypto::x509::BorrowingCertificate;
    use http_utils::urls::BaseUrl;
    use mdoc::holder::ProposedAttributes;
    use mdoc::identifiers::AttributeIdentifier;
    use openid4vc::disclosure_session::DisclosureError;
    use openid4vc::disclosure_session::DisclosureMissingAttributes;
    use openid4vc::disclosure_session::DisclosureProposal;
    use openid4vc::disclosure_session::DisclosureSession;
    use openid4vc::disclosure_session::DisclosureSessionState;
    use openid4vc::disclosure_session::DisclosureUriSource;
    use openid4vc::disclosure_session::VpSessionError;
    use openid4vc::verifier::SessionType;

    type SessionState = DisclosureSessionState<MockDisclosureMissingAttributes, MockDisclosureProposal>;
    type MockFields = (ReaderRegistration, SessionState, Option<BaseUrl>);

    pub static NEXT_START_ERROR: Mutex<Option<VpSessionError>> = Mutex::new(None);
    pub static NEXT_MOCK_FIELDS: Mutex<Option<MockFields>> = Mutex::new(None);

    mockall::mock! {
        #[derive(Debug)]
        pub DisclosureMissingAttributes {
            pub fn missing_attributes(&self) -> &[AttributeIdentifier];
        }
    }

    impl DisclosureMissingAttributes for MockDisclosureMissingAttributes {
        fn missing_attributes(&self) -> impl Iterator<Item = &AttributeIdentifier> {
            Self::missing_attributes(self).iter()
        }
    }

    #[derive(Debug)]
    pub struct MockDisclosureProposal {
        pub disclose_return_url: Option<BaseUrl>,
        pub proposed_source_identifiers: Vec<Uuid>,
        pub proposed_attributes: ProposedAttributes,
        pub disclosure_count: Arc<AtomicUsize>,
        pub next_error: Mutex<Option<VpSessionError>>,
        pub attributes_shared: bool,
        pub session_type: SessionType,
    }

    impl Default for MockDisclosureProposal {
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

    impl DisclosureProposal<Uuid> for MockDisclosureProposal {
        fn proposed_source_identifiers<'a>(&'a self) -> impl Iterator<Item = &'a Uuid>
        where
            Uuid: 'a,
        {
            self.proposed_source_identifiers.iter()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }

        async fn disclose<K, KF>(&self, _key_factory: &KF) -> Result<Option<BaseUrl>, DisclosureError<VpSessionError>> {
            if let Some(error) = self.next_error.lock().take() {
                return Err(DisclosureError::new(self.attributes_shared, error));
            }

            self.disclosure_count
                .store(self.disclosure_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);

            Ok(self.disclose_return_url.clone())
        }
    }

    #[derive(Debug)]
    pub struct MockDisclosureSession {
        pub uri_source: DisclosureUriSource,
        pub certificate: BorrowingCertificate,
        pub reader_registration: ReaderRegistration,
        pub session_state: SessionState,
        pub was_terminated: Arc<AtomicBool>,
        pub session_type: SessionType,
        pub terminate_return_url: Option<BaseUrl>,
    }

    impl MockDisclosureSession {
        pub fn next_fields(
            reader_registration: ReaderRegistration,
            session_state: SessionState,
            terminate_return_url: Option<BaseUrl>,
        ) {
            NEXT_MOCK_FIELDS
                .lock()
                .replace((reader_registration, session_state, terminate_return_url));
        }

        pub fn next_start_error(error: VpSessionError) {
            NEXT_START_ERROR.lock().replace(error);
        }
    }

    /// The reader key, generated once for testing.
    static READER_KEY: LazyLock<KeyPair> = LazyLock::new(|| {
        let reader_ca = Ca::generate_reader_mock_ca().unwrap();

        generate_reader_mock(&reader_ca, ReaderRegistration::new_mock().into()).unwrap()
    });

    impl Default for MockDisclosureSession {
        fn default() -> Self {
            Self {
                uri_source: DisclosureUriSource::Link,
                certificate: READER_KEY.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                // For convenience, the default `SessionState` is a proposal.
                session_state: DisclosureSessionState::Proposal(MockDisclosureProposal::default()),
                was_terminated: Default::default(),
                session_type: SessionType::SameDevice,
                terminate_return_url: Default::default(),
            }
        }
    }

    impl<H> DisclosureSession<Uuid, H> for MockDisclosureSession {
        type MissingAttributes = MockDisclosureMissingAttributes;
        type Proposal = MockDisclosureProposal;

        async fn start<S>(
            _client: H,
            _request_uri_query: &str,
            uri_source: DisclosureUriSource,
            _mdoc_data_source: &S,
            _trust_anchors: &[TrustAnchor<'_>],
        ) -> Result<Self, VpSessionError> {
            if let Some(error) = NEXT_START_ERROR.lock().take() {
                Err(error)?;
            }

            let (reader_registration, session_state, terminate_return_url) =
                NEXT_MOCK_FIELDS.lock().take().unwrap_or_else(|| {
                    (
                        ReaderRegistration::new_mock(),
                        DisclosureSessionState::Proposal(MockDisclosureProposal::default()),
                        None,
                    )
                });

            let session = MockDisclosureSession {
                uri_source,
                reader_registration,
                session_state,
                terminate_return_url,
                ..Default::default()
            };

            Ok(session)
        }

        fn session_state(&self) -> DisclosureSessionState<&Self::MissingAttributes, &Self::Proposal> {
            match self.session_state {
                DisclosureSessionState::MissingAttributes(ref session) => {
                    DisclosureSessionState::MissingAttributes(session)
                }
                DisclosureSessionState::Proposal(ref session) => DisclosureSessionState::Proposal(session),
            }
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
            self.was_terminated.store(true, Ordering::Relaxed);

            Ok(self.terminate_return_url)
        }

        fn verifier_certificate(&self) -> &BorrowingCertificate {
            &self.certificate
        }

        fn session_type(&self) -> SessionType {
            self.session_type
        }
    }
}
