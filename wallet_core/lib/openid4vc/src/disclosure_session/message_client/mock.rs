use std::sync::Arc;

use chrono::Utc;
use derive_more::Constructor;
use derive_more::Debug;
use futures::FutureExt;
use josekit::jwk::alg::ec::EcCurve;
use josekit::jwk::alg::ec::EcKeyPair;
use parking_lot::Mutex;
use rustls_pki_types::TrustAnchor;
use url::Url;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::x509::generate::mock::generate_reader_mock;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::KeyPair;
use crypto::utils;
use http_utils::urls::BaseUrl;
use jwt::Jwt;
use mdoc::verifier::ItemsRequests;
use mdoc::SessionTranscript;

use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::IsoVpAuthorizationRequest;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpRequestUriObject;
use crate::openid4vp::WalletRequest;
use crate::verifier::EphemeralIdParameters;
use crate::verifier::SessionType;
use crate::verifier::VerifierUrlParameters;

use super::VpMessageClient;
use super::VpMessageClientError;

/// Message that the wallet sends to the verifier through one of the mock [`VpMessageClient`] implementations.
#[derive(Debug, Clone)]
pub enum WalletMessage {
    Request(WalletRequest),
    Disclosure(String),
    Error(ErrorResponse<VpAuthorizationErrorCode>),
}

/// An implementation of [`VpMessageClient`] that sends an error made by the response factory,
/// allowing inspection of the messages that were sent.
#[derive(Debug, Clone)]
pub struct MockErrorFactoryVpMessageClient<F> {
    pub wallet_messages: Arc<Mutex<Vec<WalletMessage>>>,
    #[debug(skip)]
    pub response_factory: F,
    pub error_has_error: bool,
}

impl<F> MockErrorFactoryVpMessageClient<F> {
    pub fn new(response_factory: F, error_has_error: bool) -> Self {
        Self {
            wallet_messages: Arc::new(Mutex::new(Vec::new())),
            response_factory,
            error_has_error,
        }
    }
}

impl<F> VpMessageClient for MockErrorFactoryVpMessageClient<F>
where
    F: Fn() -> VpMessageClientError,
{
    async fn get_authorization_request(
        &self,
        _url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        self.wallet_messages
            .lock()
            .push(WalletMessage::Request(WalletRequest { wallet_nonce }));
        let error = (self.response_factory)();

        Err(error)
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.wallet_messages.lock().push(WalletMessage::Disclosure(jwe));
        let error = (self.response_factory)();

        Err(error)
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.wallet_messages.lock().push(WalletMessage::Error(error));

        if self.error_has_error {
            let error = (self.response_factory)();

            Err(error)
        } else {
            Ok(None)
        }
    }
}

pub fn request_uri_object(
    mut request_uri: Url,
    session_type: SessionType,
    request_uri_method: RequestUriMethod,
    client_id: String,
) -> VpRequestUriObject {
    request_uri.set_query(Some(
        &serde_urlencoded::to_string(VerifierUrlParameters {
            session_type,
            ephemeral_id_params: Some(EphemeralIdParameters {
                ephemeral_id: vec![42],
                time: Utc::now(),
            }),
        })
        .unwrap(),
    ));

    VpRequestUriObject {
        request_uri: request_uri.try_into().unwrap(),
        request_uri_method: Some(request_uri_method),
        client_id,
    }
}

/// Contains the minimum logic to respond with the correct verifier messages in a disclosure session,
/// exposing fields to its user to inspect and/or modify the behaviour.
#[derive(Debug)]
pub struct MockVerifierSession {
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
    pub key_pair: KeyPair,
}

impl MockVerifierSession {
    pub fn new(
        verifier_url: &BaseUrl,
        session_type: SessionType,
        request_uri_method: RequestUriMethod,
        redirect_uri: Option<BaseUrl>,
        reader_registration: Option<ReaderRegistration>,
    ) -> Self {
        // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![ca.to_trust_anchor().to_owned()];
        let key_pair = generate_reader_mock(&ca, reader_registration.clone()).unwrap();

        // Generate some OpenID4VP specific session material.
        let nonce = utils::random_string(32);
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let response_uri = verifier_url.join_base_url("response_uri");
        let request_uri_object = request_uri_object(
            verifier_url.join_base_url("request_uri").into_inner(),
            session_type,
            request_uri_method,
            String::from(key_pair.certificate().san_dns_name().unwrap().unwrap()),
        );
        let items_requests = ItemsRequests::new_pid_example();

        MockVerifierSession {
            redirect_uri,
            trust_anchors,
            reader_registration,
            key_pair,
            items_requests,
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

    pub fn session_transcript(&self, mdoc_nonce: &str) -> SessionTranscript {
        SessionTranscript::new_oid4vp(&self.response_uri, self.client_id(), self.nonce.clone(), mdoc_nonce)
    }

    pub fn request_uri_query(&self) -> String {
        self.request_uri_override
            .clone()
            .unwrap_or(serde_urlencoded::to_string(&self.request_uri_object).unwrap())
    }

    pub fn iso_auth_request(&self, wallet_nonce: Option<String>) -> IsoVpAuthorizationRequest {
        IsoVpAuthorizationRequest::new(
            &self.items_requests,
            self.key_pair.certificate(),
            self.nonce.clone(),
            self.encryption_keypair.to_jwk_public_key().try_into().unwrap(),
            self.response_uri.clone(),
            wallet_nonce,
        )
        .unwrap()
    }

    /// Generate the first protocol message of the verifier.
    fn signed_auth_request(&self, wallet_request: WalletRequest) -> Jwt<VpAuthorizationRequest> {
        let request = self.iso_auth_request(wallet_request.wallet_nonce).into();

        Jwt::sign_with_certificate(&request, &self.key_pair)
            .now_or_never()
            .unwrap()
            .unwrap()
    }
}

/// Implements [`VpMessageClient`] by simply forwarding the requests to an instance of [`MockVerifierSession`].
#[derive(Debug, Clone, Constructor)]
pub struct MockVerifierVpMessageClient {
    session: Arc<MockVerifierSession>,
}

impl VpMessageClient for MockVerifierVpMessageClient {
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
        let auth_request = self.session.signed_auth_request(wallet_request);

        Ok(auth_request)
    }

    async fn send_authorization_response(
        &self,
        _url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.session.wallet_messages.lock().push(WalletMessage::Disclosure(jwe));
        let redirect_uri = self.session.redirect_uri.clone();

        Ok(redirect_uri)
    }

    async fn send_error(
        &self,
        _url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.session.wallet_messages.lock().push(WalletMessage::Error(error));
        let redirect_uri = self.session.redirect_uri.clone();

        Ok(redirect_uri)
    }
}
