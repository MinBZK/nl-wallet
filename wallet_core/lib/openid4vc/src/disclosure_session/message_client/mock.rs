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
use attestation_data::x509::generate::mock::generate_reader_mock_with_registration;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::utils as crypto_utils;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::headers::HeaderWithX5c;
use utils::vec_nonempty;

use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::DcSdJwtAlgValues;
use crate::openid4vp::FormatAlg;
use crate::openid4vp::FormatAlgCose;
use crate::openid4vp::MsoMdocAlgValues;
use crate::openid4vp::NormalizedVpAuthorizationRequest;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpClientMetadata;
use crate::openid4vp::VpFormatsSupported;
use crate::openid4vp::VpRequestUri;
use crate::openid4vp::VpRequestUriMethod;
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
    ) -> Result<UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>, VpMessageClientError> {
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

pub fn request_uri_with_verifier_params(mut request_uri: Url, session_type: SessionType) -> BaseUrl {
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

    request_uri.try_into().unwrap()
}

pub fn request_uri(
    request_uri: Url,
    session_type: SessionType,
    request_uri_method: VpRequestUriMethod,
    client_id: &str,
) -> VpRequestUri {
    VpRequestUri {
        client_id: client_id.into(),
        object: VpRequestUriObject::AsReference {
            request_uri: request_uri_with_verifier_params(request_uri, session_type),
            request_uri_method: Some(request_uri_method),
        },
    }
}

/// Contains the minimum logic to respond with the correct verifier messages in a disclosure session,
/// exposing fields to its user to inspect and/or modify the behaviour.
#[derive(Debug)]
pub struct MockVerifierSession {
    pub redirect_uri: Option<BaseUrl>,
    pub reader_registration: Option<ReaderRegistration>,
    pub trust_anchors: Vec<TrustAnchor<'static>>,
    pub credential_requests: NormalizedCredentialRequests,
    pub nonce: String,
    pub encryption_keypair: EcKeyPair,
    pub client_id: String,
    pub request_uri: BaseUrl,
    pub request_uri_method: Option<VpRequestUriMethod>,
    pub response_uri: BaseUrl,
    pub wallet_messages: Mutex<Vec<WalletMessage>>,
    pub key_pair: KeyPair,
    pub vp_formats_supported: VpFormatsSupported,
}

impl MockVerifierSession {
    pub fn new(
        verifier_url: &BaseUrl,
        session_type: SessionType,
        request_uri_method: VpRequestUriMethod,
        redirect_uri: Option<BaseUrl>,
        reader_registration: Option<ReaderRegistration>,
        credential_requests: NormalizedCredentialRequests,
    ) -> Self {
        // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let trust_anchors = vec![ca.to_trust_anchor().to_owned()];
        let key_pair = match &reader_registration {
            Some(reader_registration) => {
                generate_reader_mock_with_registration(&ca, reader_registration.clone()).unwrap()
            }
            None => ca.generate_reader_mock().unwrap(),
        };

        // Generate some OpenID4VP specific session material.
        let nonce = crypto_utils::random_string(32);
        let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let response_uri = verifier_url.join_base_url("response_uri");
        let client_id = format!(
            "x509_san_dns:{}",
            key_pair.certificate().san_dns_name().unwrap().unwrap()
        );
        let request_uri =
            request_uri_with_verifier_params(verifier_url.join_base_url("request_uri").into_inner(), session_type);

        MockVerifierSession {
            redirect_uri,
            trust_anchors,
            reader_registration,
            key_pair,
            credential_requests,
            nonce,
            encryption_keypair,
            client_id,
            request_uri,
            request_uri_method: Some(request_uri_method),
            response_uri,
            wallet_messages: Mutex::new(Vec::new()),
            vp_formats_supported: VpFormatsSupported {
                mso_mdoc: Some(MsoMdocAlgValues {
                    issuerauth_alg_values: vec_nonempty![FormatAlgCose::ESP256].into(),
                    deviceauth_alg_values: vec_nonempty![FormatAlgCose::ESP256].into(),
                }),
                sd_jwt: Some(DcSdJwtAlgValues {
                    sd_jwt_alg_values: vec_nonempty![FormatAlg::ES256].into(),
                    kb_jwt_alg_values: vec_nonempty![FormatAlg::ES256].into(),
                }),
            },
        }
    }

    pub fn request_uri_query(&self) -> String {
        serde_urlencoded::to_string(&VpRequestUri {
            client_id: self.client_id.as_str().into(),
            object: VpRequestUriObject::AsReference {
                request_uri: self.request_uri.clone(),
                request_uri_method: self.request_uri_method,
            },
        })
        .unwrap()
    }

    pub fn normalized_auth_request(&self, wallet_nonce: Option<String>) -> NormalizedVpAuthorizationRequest {
        let mut encryption_jwk = self.encryption_keypair.to_jwk_public_key();
        encryption_jwk.set_algorithm("ECDH-ES");
        encryption_jwk.set_key_id(crypto_utils::random_string(32));

        let mut auth_request = NormalizedVpAuthorizationRequest::new_from_certificate(
            self.credential_requests.clone(),
            self.key_pair.certificate(),
            self.nonce.clone(),
            encryption_jwk.try_into().unwrap(),
            self.response_uri.clone(),
            wallet_nonce,
        );
        auth_request.client_metadata = VpClientMetadata {
            vp_formats_supported: self.vp_formats_supported.clone(),
            ..auth_request.client_metadata
        };
        auth_request
    }

    /// Generate the first protocol message of the verifier.
    fn signed_auth_request(&self, wallet_request: WalletRequest) -> SignedJwt<VpAuthorizationRequest, HeaderWithX5c> {
        let request = self.normalized_auth_request(wallet_request.wallet_nonce).into();

        SignedJwt::sign_with_certificate(&request, &self.key_pair)
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
    ) -> Result<UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>, VpMessageClientError> {
        // The URL has to match the one in the session.
        assert_eq!(url, self.session.request_uri);

        let wallet_request = WalletRequest { wallet_nonce };
        self.session
            .wallet_messages
            .lock()
            .push(WalletMessage::Request(wallet_request.clone()));
        let auth_request = self.session.signed_auth_request(wallet_request);

        Ok(auth_request.into())
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
