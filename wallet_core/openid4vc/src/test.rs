use std::str::FromStr;
use std::sync::Arc;

use assert_matches::assert_matches;
use chrono::Utc;
use derive_more::Debug;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use parking_lot::Mutex;
use rustls_pki_types::TrustAnchor;
use url::Url;

use nl_wallet_mdoc::examples::EXAMPLE_ATTRIBUTES;
use nl_wallet_mdoc::examples::EXAMPLE_DOC_TYPE;
use nl_wallet_mdoc::examples::EXAMPLE_NAMESPACE;
use nl_wallet_mdoc::holder::mock::MockMdocDataSource;
use nl_wallet_mdoc::iso::device_retrieval::ItemsRequest;
use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::verifier::ItemsRequests;
use wallet_common::jwt::Jwt;
use wallet_common::urls::BaseUrl;
use wallet_common::utils::random_string;

use crate::disclosure_session::DisclosureSession;
use crate::disclosure_session::DisclosureUriSource;
use crate::disclosure_session::VpClientError;
use crate::disclosure_session::VpMessageClient;
use crate::disclosure_session::VpMessageClientError;
use crate::jwt;
use crate::openid4vp::IsoVpAuthorizationRequest;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpRequestUriObject;
use crate::openid4vp::WalletRequest;
use crate::verifier::SessionType;
use crate::verifier::VerifierUrlParameters;
use crate::AuthorizationErrorCode;
use crate::ErrorResponse;
use crate::VpAuthorizationErrorCode;

// Constants for testing.
pub const VERIFIER_URL: &str = "http://example.com/disclosure";

/// Contains the minimum logic to respond with the correct verifier messages in a disclosure session,
/// exposing fields to its user to inspect and/or modify the behaviour.
#[derive(Debug)]
pub struct MockVerifierSession<F> {
    pub redirect_uri: Option<BaseUrl>,
    pub reader_registration: Option<ReaderRegistration>,
    pub trust_anchors: Vec<TrustAnchor<'static>>,
    pub items_requests: ItemsRequests,
    pub nonce: String,
    pub encryption_keypair: EcKeyPair,
    pub request_uri_object: VpRequestUriObject,
    pub request_uri_override: Option<String>,
    pub response_uri: BaseUrl,
    pub wallet_messages: Mutex<Vec<WalletMessage>>,

    key_pair: KeyPair,
    #[debug(skip)]
    transform_auth_request: F,
}

pub fn request_uri_object(mut request_uri: Url, session_type: SessionType, client_id: String) -> VpRequestUriObject {
    request_uri.set_query(Some(
        &serde_urlencoded::to_string(VerifierUrlParameters {
            session_type,
            ephemeral_id: vec![42],
            time: Utc::now(),
        })
        .unwrap(),
    ));

    VpRequestUriObject {
        request_uri: request_uri.try_into().unwrap(),
        request_uri_method: Some(RequestUriMethod::POST),
        client_id,
    }
}

impl<F> MockVerifierSession<F>
where
    F: Fn(VpAuthorizationRequest) -> VpAuthorizationRequest,
{
    pub fn new(
        session_type: SessionType,
        verifier_url: &BaseUrl,
        redirect_uri: Option<BaseUrl>,
        reader_registration: Option<ReaderRegistration>,
        transform_auth_request: F,
    ) -> Self {
        // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![ca.to_trust_anchor().to_owned()];
        let key_pair = ca.generate_reader_mock(reader_registration.clone()).unwrap();

        // Generate some OpenID4VP specific session material.
        let nonce = random_string(32);
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let response_uri = verifier_url.join_base_url("response_uri");
        let request_uri_object = request_uri_object(
            verifier_url.join_base_url("request_uri").into_inner(),
            session_type,
            String::from(key_pair.certificate().san_dns_name().unwrap().unwrap()),
        );
        let items_requests = vec![ItemsRequest::new_example()].into();

        MockVerifierSession {
            redirect_uri,
            trust_anchors,
            reader_registration,
            key_pair,
            items_requests,
            transform_auth_request,
            nonce,
            encryption_keypair,
            request_uri_object,
            request_uri_override: None,
            response_uri,
            wallet_messages: Mutex::new(Vec::new()),
        }
    }

    pub fn client_id(&self) -> &str {
        self.key_pair.certificate().san_dns_name().unwrap().unwrap()
    }

    pub fn request_uri_query(&self) -> String {
        self.request_uri_override
            .clone()
            .unwrap_or(serde_urlencoded::to_string(&self.request_uri_object).unwrap())
    }

    /// Generate the first protocol message of the verifier.
    async fn auth_request(&self, wallet_request: WalletRequest) -> Jwt<VpAuthorizationRequest> {
        let request = IsoVpAuthorizationRequest::new(
            &self.items_requests,
            self.key_pair.certificate(),
            self.nonce.clone(),
            self.encryption_keypair.to_jwk_public_key().try_into().unwrap(),
            self.response_uri.clone(),
            wallet_request.wallet_nonce,
        )
        .unwrap()
        .into();

        let request = (self.transform_auth_request)(request);

        jwt::sign_with_certificate(&request, &self.key_pair).await.unwrap()
    }
}

/// Implements [`VpMessageClient`] by simply forwarding the requests to an instance of [`MockVerifierSession<F>`].
#[derive(Debug)]
pub struct MockVerifierVpMessageClient<F> {
    session: Arc<MockVerifierSession<F>>,
}

impl<F> VpMessageClient for MockVerifierVpMessageClient<F>
where
    F: Fn(VpAuthorizationRequest) -> VpAuthorizationRequest,
{
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        // The URL has to match the one in the session.
        assert_eq!(url, self.session.request_uri_object.request_uri);

        let wallet_request = WalletRequest { wallet_nonce };
        self.session
            .wallet_messages
            .lock()
            .push(WalletMessage::Request(wallet_request.clone()));

        Ok(self.session.auth_request(wallet_request).await)
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.session.wallet_messages.lock().push(WalletMessage::Disclosure(jwe));

        Ok(self.session.redirect_uri.clone())
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.session.wallet_messages.lock().push(WalletMessage::Error(error));

        Ok(self.session.redirect_uri.clone())
    }
}

pub enum ReaderCertificateKind {
    NoReaderRegistration,
    WithReaderRegistration,
}

/// Perform a [`DisclosureSession`] start with test defaults.
/// This function takes several closures for modifying these
/// defaults just before they are actually used.
pub async fn disclosure_session_start<FS, FM, FD>(
    session_type: SessionType,
    disclosure_uri_source: DisclosureUriSource,
    certificate_kind: ReaderCertificateKind,
    transform_verfier_session: FS,
    transform_mdoc: FM,
    transform_device_request: FD,
) -> Result<
    (
        DisclosureSession<MockVerifierVpMessageClient<FD>, String>,
        Arc<MockVerifierSession<FD>>,
    ),
    (VpClientError, Arc<MockVerifierSession<FD>>),
>
where
    FS: FnOnce(MockVerifierSession<FD>) -> MockVerifierSession<FD>,
    FM: FnOnce(MockMdocDataSource) -> MockMdocDataSource,
    FD: Fn(VpAuthorizationRequest) -> VpAuthorizationRequest,
{
    // Create a reader registration with all of the example attributes,
    // if we should have a reader registration at all.
    let reader_registration = match certificate_kind {
        ReaderCertificateKind::NoReaderRegistration => None,
        ReaderCertificateKind::WithReaderRegistration => ReaderRegistration {
            attributes: ReaderRegistration::create_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            ),
            ..ReaderRegistration::new_mock()
        }
        .into(),
    };

    // Create a mock session and call the transform callback.
    let verifier_session = MockVerifierSession::<FD>::new(
        session_type,
        &VERIFIER_URL.parse().unwrap(),
        Some(BaseUrl::from_str(VERIFIER_URL).unwrap().join_base_url("redirect_uri")),
        reader_registration,
        transform_device_request,
    );
    let verifier_session = Arc::new(transform_verfier_session(verifier_session));

    let client = MockVerifierVpMessageClient {
        session: Arc::clone(&verifier_session),
    };

    // Set up the mock data source.
    let mdoc_data_source = transform_mdoc(MockMdocDataSource::new_example());

    // Starting disclosure and return the result.
    let result = DisclosureSession::start(
        client,
        &verifier_session.request_uri_query(),
        disclosure_uri_source,
        &mdoc_data_source,
        &verifier_session.trust_anchors,
    )
    .await;

    match result {
        Ok(disclosure_session) => Ok((disclosure_session, Arc::clone(&verifier_session))),
        Err(err) => Err((err, verifier_session)),
    }
}

/// An implementation of [`VpMessageClient`] that sends an error made by the response factory,
/// allowing inspection of the messages that were sent.
#[derive(Debug)]
pub struct MockErrorFactoryVpMessageClient<F> {
    #[debug(skip)]
    pub response_factory: F,
    pub wallet_messages: Arc<Mutex<Vec<WalletMessage>>>,
}

impl<F> MockErrorFactoryVpMessageClient<F> {
    pub fn new(response_factory: F, wallet_messages: Arc<Mutex<Vec<WalletMessage>>>) -> Self {
        Self {
            response_factory,
            wallet_messages,
        }
    }
}

/// Message that the wallet sends to the verifier through one of the mock [`VpMessageClient`] implementations.
#[derive(Debug, Clone)]
pub enum WalletMessage {
    Request(WalletRequest),
    Disclosure(String),
    Error(ErrorResponse<VpAuthorizationErrorCode>),
}

impl WalletMessage {
    pub fn request(&self) -> &WalletRequest {
        match self {
            WalletMessage::Request(request) => request,
            _ => panic!("not a request"),
        }
    }

    pub fn disclosure(&self) -> &str {
        match self {
            WalletMessage::Disclosure(jwe) => jwe,
            _ => panic!("not a disclosure"),
        }
    }

    pub fn error(&self) -> &ErrorResponse<VpAuthorizationErrorCode> {
        match self {
            WalletMessage::Error(error) => error,
            _ => panic!("not a error"),
        }
    }
}

impl<F> VpMessageClient for MockErrorFactoryVpMessageClient<F>
where
    F: Fn() -> Option<VpMessageClientError>,
{
    async fn get_authorization_request(
        &self,
        _url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        self.wallet_messages
            .lock()
            .push(WalletMessage::Request(WalletRequest { wallet_nonce }));
        Err((self.response_factory)().unwrap())
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.wallet_messages.lock().push(WalletMessage::Disclosure(jwe));
        Err((self.response_factory)().unwrap())
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.wallet_messages.lock().push(WalletMessage::Error(error));

        match (self.response_factory)() {
            Some(err) => Err(err),
            None => Ok(None),
        }
    }
}

pub async fn test_disclosure_session_start_error_http_client<F>(error_factory: F) -> (VpClientError, Vec<WalletMessage>)
where
    F: Fn() -> Option<VpMessageClientError>,
{
    let wallet_messages = Arc::new(Mutex::new(Vec::new()));
    let client = MockErrorFactoryVpMessageClient::new(error_factory, Arc::clone(&wallet_messages));

    let request_query = serde_urlencoded::to_string(request_uri_object(
        BaseUrl::from_str(VERIFIER_URL)
            .unwrap()
            .join_base_url("redirect_uri")
            .into_inner(),
        SessionType::SameDevice,
        "client_id".to_string(),
    ))
    .unwrap();

    // This mdoc data source is not actually consulted.
    let mdoc_data_source = MockMdocDataSource::default();

    let error = DisclosureSession::start(
        client,
        &request_query,
        DisclosureUriSource::Link,
        &mdoc_data_source,
        &[],
    )
    .await
    .expect_err("Starting disclosure session should have resulted in an error");

    // Collect the messages sent through the `VpMessageClient`.
    let wallet_messages = wallet_messages.lock();
    (error, wallet_messages.clone())
}

pub fn iso_auth_request() -> IsoVpAuthorizationRequest {
    let ca = Ca::generate_reader_mock_ca().unwrap();
    let key_pair = ca
        .generate_reader_mock(Some(ReaderRegistration {
            attributes: ReaderRegistration::create_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            ),
            ..ReaderRegistration::new_mock()
        }))
        .unwrap();

    IsoVpAuthorizationRequest::new(
        &vec![ItemsRequest::new_example()].into(),
        key_pair.certificate(),
        random_string(32),
        EcKeyPair::generate(EcCurve::P256)
            .unwrap()
            .to_jwk_public_key()
            .try_into()
            .unwrap(),
        VERIFIER_URL.parse().unwrap(),
        Some(random_string(32)),
    )
    .unwrap()
}

pub async fn test_disclosure_session_terminate<H>(
    session: DisclosureSession<H, String>,
    wallet_messages: Arc<Mutex<Vec<WalletMessage>>>,
) -> Result<Option<BaseUrl>, VpClientError>
where
    H: VpMessageClient,
{
    let result = session.terminate().await;

    let wallet_messages = wallet_messages.lock();
    let WalletMessage::Error(error) = wallet_messages.last().unwrap() else {
        panic!("wallet should have sent an error");
    };
    assert_matches!(
        error.error,
        VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied)
    );

    result
}
