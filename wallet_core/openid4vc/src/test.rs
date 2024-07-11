use std::{
    fmt,
    str::FromStr,
    sync::{Arc, Mutex},
};

use assert_matches::assert_matches;
use chrono::Utc;
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};

use nl_wallet_mdoc::{
    examples::{EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
    holder::{
        test::{
            example_items_request, MdocIdentifier, MockMdocDataSource, ReaderCertificateKind, EXAMPLE_ATTRIBUTES,
            VERIFIER_URL,
        },
        DisclosureUriSource, TrustAnchor,
    },
    server_keys::KeyPair,
    utils::reader_auth::ReaderRegistration,
    verifier::{ItemsRequests, SessionType},
};
use url::Url;
use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt, trust_anchor::DerTrustAnchor, utils::random_string};

use crate::{
    disclosure_session::{DisclosureSession, VpClientError, VpMessageClient, VpMessageClientError},
    jwt,
    openid4vp::{
        IsoVpAuthorizationRequest, RequestUriMethod, VpAuthorizationRequest, VpRequestUriObject, WalletRequest,
    },
    verifier::VerifierUrlParameters,
    AuthorizationErrorCode, ErrorResponse, VpAuthorizationErrorCode,
};

/// Contains the minimum logic to respond with the correct verifier messages in a disclosure session,
/// exposing fields to its user to inspect and/or modify the behaviour.
pub struct MockVerifierSession<F> {
    pub session_type: SessionType,
    pub redirect_uri: Option<BaseUrl>,
    pub reader_registration: Option<ReaderRegistration>,
    pub trust_anchors: Vec<DerTrustAnchor>,
    pub items_requests: ItemsRequests,
    pub nonce: String,
    pub encryption_keypair: EcKeyPair,
    pub request_uri_object: VpRequestUriObject,
    pub request_uri_override: Option<String>,
    pub response_uri: BaseUrl,
    pub wallet_request: Mutex<Option<WalletRequest>>,
    pub disclosure_jwe: Mutex<Option<String>>,
    pub wallet_error: Mutex<Option<ErrorResponse<VpAuthorizationErrorCode>>>,

    key_pair: KeyPair,
    transform_auth_request: F,
}

impl<F> fmt::Debug for MockVerifierSession<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockVerifierSession")
            .field("session_type", &self.session_type)
            .field("redirect_uri", &self.redirect_uri)
            .field("reader_registration", &self.reader_registration)
            .field("trust_anchors", &self.trust_anchors)
            .field("items_requests", &self.items_requests)
            .field("nonce", &self.nonce)
            .field("encryption_keypair", &self.encryption_keypair)
            .field("request_uri_object", &self.request_uri_object)
            .field("request_uri_override", &self.request_uri_override)
            .field("response_uri", &self.response_uri)
            .field("wallet_request", &self.wallet_request)
            .field("disclosure_jwe", &self.disclosure_jwe)
            .field("key_pair", &self.key_pair)
            .finish_non_exhaustive()
    }
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
        verifier_url: BaseUrl,
        redirect_uri: Option<BaseUrl>,
        reader_registration: Option<ReaderRegistration>,
        transform_auth_request: F,
    ) -> Self {
        // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
        let ca = KeyPair::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![DerTrustAnchor::from_der(ca.certificate().as_bytes().to_vec()).unwrap()];
        let key_pair = ca.generate_reader_mock(reader_registration.clone()).unwrap();

        // Generate some OpenID4VP specific session material.
        let nonce = random_string(32);
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let response_uri = verifier_url.join_base_url("response_uri");
        let request_uri_object = request_uri_object(
            verifier_url.join_base_url("request_uri").into_inner(),
            session_type,
            key_pair.certificate().san_dns_name().unwrap().unwrap(),
        );
        let items_requests = vec![example_items_request()].into();

        MockVerifierSession {
            session_type,
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
            wallet_request: Mutex::new(None),
            disclosure_jwe: Mutex::new(None),
            wallet_error: Mutex::new(None),
        }
    }

    pub fn client_id(&self) -> String {
        self.key_pair.certificate().san_dns_name().unwrap().unwrap()
    }

    fn trust_anchors(&self) -> Vec<TrustAnchor> {
        self.trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect()
    }

    pub fn request_uri_query(&self) -> String {
        self.request_uri_override
            .as_ref()
            .cloned()
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
            wallet_request.wallet_nonce.clone(),
        )
        .unwrap()
        .into();

        let request = (self.transform_auth_request)(request);

        jwt::sign_with_certificate(&request, &self.key_pair).await.unwrap()
    }
}

/// Implements [`VpMessageClient`] by simply forwarding the requests to an instance of [`MockVerifierSession<F>`].
pub struct MockVerifierMessageClient<F> {
    session: Arc<MockVerifierSession<F>>,
}

impl<F> fmt::Debug for MockVerifierMessageClient<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockVerifierSessionClient")
            .field("session", &self.session)
            .finish()
    }
}

impl<F> VpMessageClient for MockVerifierMessageClient<F>
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
        *self.session.wallet_request.lock().unwrap() = Some(wallet_request.clone());

        Ok(self.session.auth_request(wallet_request).await)
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        *self.session.disclosure_jwe.lock().unwrap() = Some(jwe);

        Ok(self.session.redirect_uri.clone())
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        *self.session.wallet_error.lock().unwrap() = Some(error);

        Ok(self.session.redirect_uri.clone())
    }
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
        DisclosureSession<MockVerifierMessageClient<FD>, MdocIdentifier>,
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
        VERIFIER_URL.parse().unwrap(),
        Some(BaseUrl::from_str(VERIFIER_URL).unwrap().join_base_url("redirect_uri")),
        reader_registration,
        transform_device_request,
    );
    let verifier_session = Arc::new(transform_verfier_session(verifier_session));

    let client = MockVerifierMessageClient {
        session: Arc::clone(&verifier_session),
    };

    // Set up the mock data source.
    let mdoc_data_source = transform_mdoc(MockMdocDataSource::default());

    // Starting disclosure and return the result.
    let result = DisclosureSession::start(
        client,
        &verifier_session.request_uri_query(),
        disclosure_uri_source,
        &mdoc_data_source,
        &verifier_session.trust_anchors(),
    )
    .await;

    result
        .map(|disclosure_session| (disclosure_session, Arc::clone(&verifier_session)))
        .map_err(|err| (err, verifier_session))
}

/// An implementation of [`VpMessageClient`] that sends a response made by the factory,
/// allowing inspection of the messages that were sent.
pub struct MockVpMessageClient<F> {
    pub response_factory: F,
    pub wallet_messages: WalletMessages,
}

impl<F> fmt::Debug for MockVpMessageClient<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockHttpClient")
            .field("wallet_messages", &self.wallet_messages)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct WalletMessages {
    pub request: Arc<Mutex<Option<WalletRequest>>>,
    pub disclosure: Arc<Mutex<Option<String>>>,
    pub error: Arc<Mutex<Option<ErrorResponse<VpAuthorizationErrorCode>>>>,
}

impl WalletMessages {
    pub fn new() -> Self {
        Self {
            request: Arc::new(Mutex::new(None)),
            disclosure: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
        }
    }
}

impl<F> VpMessageClient for MockVpMessageClient<F>
where
    F: Fn() -> Option<VpMessageClientError>,
{
    async fn get_authorization_request(
        &self,
        _url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        *self.wallet_messages.request.lock().unwrap() = Some(WalletRequest { wallet_nonce });
        Err((self.response_factory)().unwrap())
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        *self.wallet_messages.disclosure.lock().unwrap() = Some(jwe);
        Err((self.response_factory)().unwrap())
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        *self.wallet_messages.error.lock().unwrap() = Some(error);

        match (self.response_factory)() {
            Some(err) => Err(err),
            None => Ok(None),
        }
    }
}

pub async fn test_disclosure_session_start_error_http_client<F>(
    error_factory: F,
) -> (
    VpClientError,
    Option<WalletRequest>,
    Option<ErrorResponse<VpAuthorizationErrorCode>>,
)
where
    F: Fn() -> Option<VpMessageClientError>,
{
    let wallet_messages = WalletMessages::new();
    let client = MockVpMessageClient {
        response_factory: error_factory,
        wallet_messages: wallet_messages.clone(),
    };

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
    let wallet_request = wallet_messages.request.lock().unwrap().clone();
    let wallet_error = wallet_messages.error.lock().unwrap().clone();
    (error, wallet_request, wallet_error)
}

pub fn iso_auth_request() -> IsoVpAuthorizationRequest {
    let ca = KeyPair::generate_reader_mock_ca().unwrap();
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
        &vec![example_items_request()].into(),
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
    session: DisclosureSession<H, MdocIdentifier>,
    wallet_error: Arc<Mutex<Option<ErrorResponse<VpAuthorizationErrorCode>>>>,
) -> Result<(), VpClientError>
where
    H: VpMessageClient,
{
    let result = session.terminate().await;

    let wallet_error = wallet_error.lock().unwrap();
    let wallet_error = wallet_error.as_ref().expect("wallet should have sent an error");
    assert_matches!(
        wallet_error.error,
        VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied)
    );

    result
}
