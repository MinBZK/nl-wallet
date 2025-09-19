use derive_more::Constructor;
use reqwest::ClientBuilder;
use tracing::info;
use tracing::warn;

use crypto::utils as crypto_utils;
use crypto::x509::BorrowingCertificate;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;

use crate::errors::AuthorizationErrorCode;
use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpRequestUriObject;
use crate::verifier::VerifierUrlParameters;

use super::DisclosureClient;
use super::VerifierCertificate;
use super::error::VpClientError;
use super::error::VpSessionError;
use super::error::VpVerifierError;
use super::message_client::HttpVpMessageClient;
use super::message_client::VpMessageClient;
use super::message_client::VpMessageClientError;
use super::session::VpDisclosureSession;
use super::uri_source::DisclosureUriSource;

#[derive(Debug, Constructor)]
pub struct VpDisclosureClient<H = HttpVpMessageClient> {
    client: H,
}

impl VpDisclosureClient<HttpVpMessageClient> {
    pub fn new_http(client_builder: ClientBuilder) -> Result<Self, reqwest::Error> {
        let client = Self::new(HttpVpMessageClient::new(client_builder)?);

        Ok(client)
    }
}

impl<H> VpDisclosureClient<H> {
    /// Report an error back to the RP. Note: this function only reports errors that are the RP's fault.
    async fn report_error_back(&self, url: BaseUrl, error: VpVerifierError) -> VpVerifierError
    where
        H: VpMessageClient,
    {
        match error {
            VpVerifierError::Request(VpMessageClientError::Json(_))
            | VpVerifierError::AuthRequestValidation(_)
            | VpVerifierError::IncorrectClientId { .. }
            | VpVerifierError::RpCertificate(_)
            | VpVerifierError::MissingReaderRegistration
            | VpVerifierError::RequestedAttributesValidation(_) => {
                let error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest);

                let error_response = ErrorResponse {
                    error: error_code,
                    error_description: Some(error.to_string()),
                    error_uri: None,
                };

                // If sending the error results in an error, log it but do nothing else.
                let _ = self
                    .client
                    .send_error(url, error_response)
                    .await
                    .inspect_err(|err| warn!("failed to send error to verifier: {err}"));
            }
            // don't report other errors
            _ => {}
        };

        error
    }

    /// Internal helper function for processing and checking the Authorization Request.
    fn process_auth_request(
        request_uri_client_id: &str,
        auth_request_client_id: &str,
        credential_requests: &NormalizedCredentialRequests,
        certificate: BorrowingCertificate,
    ) -> Result<VerifierCertificate, VpVerifierError> {
        // The `client_id` in the Authorization Request, which has been authenticated, has to equal
        // the `client_id` that the RP sent in the Request URI object at the start of the session.
        if auth_request_client_id != request_uri_client_id {
            return Err(VpVerifierError::IncorrectClientId {
                expected: request_uri_client_id.to_string(),
                found: auth_request_client_id.to_string(),
            })?;
        }

        // Extract `ReaderRegistration` from the certificate.
        let verifier_certificate = VerifierCertificate::try_new(certificate)
            .map_err(VpVerifierError::RpCertificate)?
            .ok_or(VpVerifierError::MissingReaderRegistration)?;

        // Verify that the requested attributes are included in the reader authentication.
        verifier_certificate
            .registration()
            .verify_requested_attributes(credential_requests.as_ref())
            .map_err(VpVerifierError::RequestedAttributesValidation)?;

        Ok(verifier_certificate)
    }
}

impl<H> DisclosureClient for VpDisclosureClient<H>
where
    H: VpMessageClient + Clone,
{
    type Session = VpDisclosureSession<H>;

    async fn start(
        &self,
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        trust_anchors: &[rustls_pki_types::TrustAnchor<'_>],
    ) -> Result<Self::Session, VpSessionError> {
        info!("start disclosure session");

        let request_uri_object: VpRequestUriObject =
            serde_urlencoded::from_str(request_uri_query).map_err(VpClientError::RequestUri)?;

        // Parse the `SessionType` from the verifier URL.
        let VerifierUrlParameters { session_type, .. } = serde_urlencoded::from_str(
            request_uri_object
                .request_uri
                .as_ref()
                .query()
                .ok_or(VpVerifierError::MissingSessionType)?,
        )
        .map_err(VpVerifierError::MalformedSessionType)?;

        // Check the `SessionType` that was contained in the verifier URL against the source of the URI.
        // A same-device session is expected to come from a Universal Link,
        // while a cross-device session should come from a scanned QR code.
        if uri_source.session_type() != session_type {
            return Err(VpClientError::DisclosureUriSourceMismatch(session_type, uri_source).into());
        }

        // If the server supports it, require it to include a nonce in the Authorization Request JWT
        let method = request_uri_object.request_uri_method.unwrap_or_default();
        let request_nonce = match method {
            RequestUriMethod::GET => None,
            RequestUriMethod::POST => Some(crypto_utils::random_string(32)),
        };

        let jws = self
            .client
            .get_authorization_request(request_uri_object.request_uri, request_nonce.clone())
            .await?;

        let (vp_auth_request, certificate) = VpAuthorizationRequest::try_new(&jws, trust_anchors)?;
        let response_uri = vp_auth_request.response_uri.clone();

        let auth_request_result = vp_auth_request
            .validate(&certificate, request_nonce.as_deref())
            .map_err(VpVerifierError::AuthRequestValidation);
        let auth_request = match (auth_request_result, response_uri) {
            (Err(error), Some(response_uri)) => {
                return Err(self.report_error_back(response_uri, error).await)?;
            }
            (result, _) => result?,
        };

        let process_request_result = Self::process_auth_request(
            &request_uri_object.client_id,
            &auth_request.client_id,
            &auth_request.credential_requests,
            certificate,
        );
        let verifier_certificate = match process_request_result {
            Ok(value) => value,
            Err(error) => return Err(self.report_error_back(auth_request.response_uri, error).await)?,
        };

        let session = VpDisclosureSession::new(self.client.clone(), session_type, verifier_certificate, auth_request);

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use http::StatusCode;
    use itertools::Itertools;
    use rstest::rstest;
    use serde::de::Error;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::auth::reader_auth::ValidationError;
    use attestation_data::disclosure::DisclosedAttributes;
    use attestation_types::claim_path::ClaimPath;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::x509::BorrowingCertificateExtension;
    use crypto::x509::CertificateUsage;
    use dcql::CredentialFormat;
    use dcql::normalized::NormalizedCredentialRequests;
    use http_utils::urls::BaseUrl;
    use mdoc::holder::disclosure::PartialMdoc;
    use mdoc::test::data::PID;
    use sd_jwt::sd_jwt::SdJwt;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::mock_remote::MOCK_WALLET_CLIENT_ID;
    use wscd::mock_remote::MockRemoteWscd;

    use crate::errors::AuthorizationErrorCode;
    use crate::errors::VpAuthorizationErrorCode;
    use crate::openid4vp::AuthRequestValidationError;
    use crate::openid4vp::RequestUriMethod;
    use crate::openid4vp::VpAuthorizationResponse;
    use crate::openid4vp::VpRequestUriObject;
    use crate::openid4vp::WalletRequest;
    use crate::verifier::SessionType;

    use super::super::DisclosableAttestations;
    use super::super::DisclosureClient;
    use super::super::DisclosureSession;
    use super::super::DisclosureUriSource;
    use super::super::client::VpMessageClientError;
    use super::super::error::VpClientError;
    use super::super::error::VpSessionError;
    use super::super::error::VpVerifierError;
    use super::super::message_client::mock::MockErrorFactoryVpMessageClient;
    use super::super::message_client::mock::MockVerifierSession;
    use super::super::message_client::mock::MockVerifierVpMessageClient;
    use super::super::message_client::mock::WalletMessage;
    use super::super::message_client::mock::request_uri_object;
    use super::super::session::VpDisclosureSession;
    use super::VpDisclosureClient;

    static VERIFIER_URL: LazyLock<BaseUrl> = LazyLock::new(|| "http://example.com/disclosure".parse().unwrap());

    type StartDisclosureResult = Result<
        (
            VpDisclosureSession<MockVerifierVpMessageClient>,
            Arc<MockVerifierSession>,
        ),
        (Box<VpSessionError>, Arc<MockVerifierSession>),
    >;

    #[expect(clippy::too_many_arguments)]
    fn start_disclosure_session<SF>(
        session_type: SessionType,
        uri_source: DisclosureUriSource,
        request_uri_method: RequestUriMethod,
        redirect_uri: Option<BaseUrl>,
        credential_format: CredentialFormat,
        reader_registration_pid_attributes: &[&str],
        transform_verifier_session: SF,
    ) -> StartDisclosureResult
    where
        SF: FnOnce(MockVerifierSession) -> MockVerifierSession,
    {
        // If the list of PID attributes is empty, do not generate a `ReaderRegistration`.
        let reader_registration = (!reader_registration_pid_attributes.is_empty()).then(|| {
            let authorized_attributes = match credential_format {
                CredentialFormat::MsoMdoc => ReaderRegistration::create_mdoc_attributes(
                    PID,
                    PID,
                    reader_registration_pid_attributes.iter().copied(),
                ),
                CredentialFormat::SdJwt => ReaderRegistration::create_attributes(
                    PID,
                    reader_registration_pid_attributes
                        .iter()
                        .map(|attribute| std::iter::once(*attribute)),
                ),
            };

            ReaderRegistration {
                authorized_attributes,
                ..ReaderRegistration::new_mock()
            }
        });

        let credential_requests = match credential_format {
            CredentialFormat::MsoMdoc => NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
            CredentialFormat::SdJwt => NormalizedCredentialRequests::new_mock_sd_jwt_pid_example(),
        };

        // Prepare a mock `VpMessageClient` implementation that embeds everything we need for a disclosure session.
        let verifier_session = MockVerifierSession::new(
            &VERIFIER_URL,
            session_type,
            request_uri_method,
            redirect_uri,
            reader_registration,
            credential_requests,
        );
        let verifier_session = Arc::new(transform_verifier_session(verifier_session));
        let mock_client = MockVerifierVpMessageClient::new(Arc::clone(&verifier_session));

        // Create a new `VpDisclosureClient` and start a disclosure session.
        let client = VpDisclosureClient::new(mock_client);

        let disclosure_session_result = client
            .start(
                &verifier_session.request_uri_query(),
                uri_source,
                &verifier_session.trust_anchors,
            )
            .now_or_never()
            .unwrap();

        match disclosure_session_result {
            Ok(disclosure_session) => Ok((disclosure_session, verifier_session)),
            Err(err) => Err((Box::new(err), verifier_session)),
        }
    }

    /// This tests the full happy path for both `VpDisclosureClient` and `VpDisclosureSession`.
    #[rstest]
    #[case(SessionType::SameDevice, DisclosureUriSource::Link)]
    #[case(SessionType::CrossDevice, DisclosureUriSource::QrCode)]
    fn test_vp_disclosure_client_full(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] credential_format: CredentialFormat,
        #[case] session_type: SessionType,
        #[case] uri_source: DisclosureUriSource,
        #[values(RequestUriMethod::GET, RequestUriMethod::POST)] request_uri_method: RequestUriMethod,
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<BaseUrl>,
    ) {
        let (disclosure_session, verifier_session) = start_disclosure_session(
            session_type,
            uri_source,
            request_uri_method,
            redirect_uri.clone(),
            credential_format,
            &["bsn", "given_name", "family_name"],
            std::convert::identity,
        )
        .expect("starting a new disclosure session on VpDisclosureClient should succeed");

        let wallet_nonce = {
            // The verifier should now have recieved a message from the client,
            // which may include a wallet nonce based on the request URI method.
            let wallet_messages = verifier_session.wallet_messages.lock();

            assert_eq!(wallet_messages.len(), 1);
            let message = wallet_messages.last().unwrap();

            let WalletMessage::Request(WalletRequest { wallet_nonce }) = message else {
                panic!("wallet should have sent initial request");
            };

            match request_uri_method {
                RequestUriMethod::GET => assert!(wallet_nonce.is_none()),
                RequestUriMethod::POST => assert!(wallet_nonce.is_some()),
            }

            // Extract the wallet nonce for response validation later on.
            wallet_nonce.as_ref().cloned()
        };

        // Check all of the data the new `VpDisclosureSession` exposes.
        assert_eq!(disclosure_session.session_type(), session_type);

        assert_eq!(
            *disclosure_session.credential_requests(),
            match credential_format {
                CredentialFormat::MsoMdoc => NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
                CredentialFormat::SdJwt => NormalizedCredentialRequests::new_mock_sd_jwt_pid_example(),
            }
        );

        assert_eq!(
            disclosure_session.verifier_certificate().certificate(),
            verifier_session.key_pair.certificate()
        );
        assert_eq!(
            *disclosure_session.verifier_certificate().registration(),
            ReaderRegistration::from_certificate(disclosure_session.verifier_certificate().certificate())
                .unwrap()
                .unwrap(),
        );
        assert_eq!(
            disclosure_session.verifier_certificate().registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        // Create an attestation and disclose it.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let attestation_key = MockRemoteEcdsaKey::new_random("attestation_key".to_string());
        let wscd = MockRemoteWscd::new(vec![attestation_key.clone()]);

        let attestations = match credential_format {
            CredentialFormat::MsoMdoc => {
                let partial_mdoc = PartialMdoc::new_mock_with_ca_and_key(&ca, &attestation_key);

                DisclosableAttestations::MsoMdoc(HashMap::from([(
                    "mdoc_pid_example".try_into().unwrap(),
                    vec_nonempty![partial_mdoc],
                )]))
                .try_into()
                .unwrap()
            }
            CredentialFormat::SdJwt => {
                let issuer_key_pair = ca
                    .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
                    .unwrap();
                let sd_jwt = SdJwt::example_pid_sd_jwt(&issuer_key_pair, attestation_key.verifying_key());
                let unsigned_sd_jwt_presentation = sd_jwt
                    .into_presentation_builder()
                    .disclose(&vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())])
                    .unwrap()
                    .disclose(&vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())])
                    .unwrap()
                    .disclose(&vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())])
                    .unwrap()
                    .finish();

                DisclosableAttestations::SdJwt(HashMap::from([(
                    "sd_jwt_pid_example".try_into().unwrap(),
                    vec_nonempty![(unsigned_sd_jwt_presentation, "attestation_key".to_string())],
                )]))
                .try_into()
                .unwrap()
            }
        };

        let disclosure_redirect_uri = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect("disclosing mdoc using VpDisclosureSession should succeed");

        // We should have recieved the correct redirect URI from the verifier
        // and the verifier should have received the auth response.
        assert_eq!(disclosure_redirect_uri, redirect_uri);

        let wallet_messages = verifier_session.wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 2);
        let message = wallet_messages.last().unwrap();

        let WalletMessage::Disclosure(jwe) = message else {
            panic!("verifier should have received authentiation response from holder");
        };

        // Decrypt and verify the response that was sent by `VpDisclosureSession`.
        let disclosed_attestations = VpAuthorizationResponse::decrypt_and_verify(
            jwe,
            &verifier_session.encryption_keypair,
            &verifier_session.normalized_auth_request(wallet_nonce),
            &[MOCK_WALLET_CLIENT_ID.to_string()],
            &MockTimeGenerator::default(),
            &[ca.to_trust_anchor()],
        )
        .expect("decrypting and verifying VPDisclosureSession authorization response should succeed");

        // Finally, check that the disclosed attestations match exactly those provided.
        let disclosed_attributes = disclosed_attestations
            .iter()
            .exactly_one()
            .ok()
            .and_then(|attestations| attestations.attestations.iter().exactly_one().ok())
            .and_then(|attestation| (attestation.attestation_type.as_str() == PID).then_some(attestation))
            .and_then(|attestation| match &attestation.attributes {
                DisclosedAttributes::MsoMdoc(attributes) => attributes
                    .iter()
                    .exactly_one()
                    .ok()
                    .and_then(|(namespaces, attributes)| (namespaces == PID).then_some(attributes))
                    .map(|attributes| {
                        attributes
                            .iter()
                            .filter_map(|(key, value)| {
                                match value {
                                    AttributeValue::Text(text) => Some(text),
                                    _ => None,
                                }
                                .map(|text| (key.as_str(), text.as_str()))
                            })
                            .collect::<HashSet<_>>()
                    }),
                DisclosedAttributes::SdJwt(attributes) => Some(
                    attributes
                        .flattened()
                        .into_iter()
                        .flat_map(|(path, value)| match (path.iter().exactly_one().ok(), value) {
                            (Some(path), AttributeValue::Text(text)) => Some((*path, text.as_str())),
                            _ => None,
                        })
                        .collect(),
                ),
            })
            .unwrap_or_default();

        assert!(
            HashSet::from([
                ("bsn", "999999999"),
                ("given_name", "Willeke Liselotte"),
                ("family_name", "De Bruijn"),
            ])
            .is_subset(&disclosed_attributes)
        );
    }

    #[test]
    fn test_vp_disclosure_client_start_error_request_uri() {
        // Calling `VpDisclosureClient::start()` with an invalid request URI object should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            |mut verifier_session| {
                verifier_session.request_uri_override = Some(String::new());

                verifier_session
            },
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(*error, VpSessionError::Client(VpClientError::RequestUri(_)));
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[test]
    fn test_vp_disclosure_client_start_error_missing_session_type() {
        // Calling `VpDisclosureClient::start()` with a request URI object
        // that does not contain a request URI should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            |mut verifier_session| {
                let mut request_uri = verifier_session.request_uri_object.request_uri.clone().into_inner();
                request_uri.set_query(None);

                verifier_session.request_uri_object.request_uri = request_uri.try_into().unwrap();

                verifier_session
            },
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(*error, VpSessionError::Verifier(VpVerifierError::MissingSessionType));
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[test]
    fn test_vp_disclosure_client_start_error_malformed_session_type() {
        // Calling `VpDisclosureClient::start()` with a request URI object that contains
        // a request URI with an invalid session_type should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            |mut verifier_session| {
                let mut request_uri_object: VpRequestUriObject =
                    serde_urlencoded::from_str(&verifier_session.request_uri_query()).unwrap();
                request_uri_object.request_uri = format!("{}?session_type=invalid", LazyLock::force(&VERIFIER_URL))
                    .parse()
                    .unwrap();

                verifier_session.request_uri_object = request_uri_object;

                verifier_session
            },
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(
            *error,
            VpSessionError::Verifier(VpVerifierError::MalformedSessionType(_))
        );
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[rstest]
    #[case(SessionType::SameDevice, DisclosureUriSource::QrCode)]
    #[case(SessionType::CrossDevice, DisclosureUriSource::Link)]
    fn test_vp_disclosure_client_start_error_disclosure_uri_source_mismatch(
        #[case] session_type: SessionType,
        #[case] uri_source: DisclosureUriSource,
    ) {
        // Calling `VpDisclosureClient::start()` with a request URI object that contains a
        // `SessionType` that is incompatible with its source should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            session_type,
            uri_source,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            std::convert::identity,
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(
            *error,
            VpSessionError::Client(VpClientError::DisclosureUriSourceMismatch(
                typ,
                source
            )) if typ == session_type && source == uri_source);
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[test]
    fn test_vp_disclosure_client_start_error_auth_request_validation() {
        // Calling `VpDisclosureClient::start()` without trust anchors should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            |mut verifier_session| {
                verifier_session.trust_anchors.clear();

                verifier_session
            },
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(
            *error,
            VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(
                AuthRequestValidationError::JwtVerification(_)
            ))
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.first().unwrap(), WalletMessage::Request(_));
    }

    fn start_disclosure_session_http_error<F>(
        error_factory: F,
        error_has_error: bool,
    ) -> (VpSessionError, Vec<WalletMessage>)
    where
        F: Fn() -> VpMessageClientError + Clone,
    {
        let error_client = MockErrorFactoryVpMessageClient::new(error_factory, error_has_error);
        let wallet_messages = Arc::clone(&error_client.wallet_messages);

        let request_query = serde_urlencoded::to_string(request_uri_object(
            VERIFIER_URL.join_base_url("redirect_uri").into_inner(),
            SessionType::SameDevice,
            RequestUriMethod::POST,
            "client_id".to_string(),
        ))
        .unwrap();

        let client = VpDisclosureClient::new(error_client);
        let error = client
            .start(&request_query, DisclosureUriSource::Link, &[])
            .now_or_never()
            .unwrap()
            .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        // Collect the messages sent through the `VpMessageClient`.
        let wallet_messages = wallet_messages.lock();

        (error, wallet_messages.clone())
    }

    #[rstest]
    fn test_vp_disclosure_client_start_error_verifier_request(#[values(false, true)] error_has_error: bool) {
        let (error, wallet_messages) =
            start_disclosure_session_http_error(|| serde_json::Error::custom("").into(), error_has_error);

        // Trying to start a session in which the transport gives a
        // JSON error should result in the error being forwarded.
        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.first().unwrap(), WalletMessage::Request(_));
    }

    #[rstest]
    fn test_vp_disclosure_client_start_error_client_request(#[values(false, true)] error_has_error: bool) {
        let (error, wallet_messages) = start_disclosure_session_http_error(
            || {
                let response = http::Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body("")
                    .unwrap();

                reqwest::Response::from(response).error_for_status().unwrap_err().into()
            },
            error_has_error,
        );

        // Trying to start a session in which the transport gives a
        // HTTP error should result in the error being forwarded.
        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::Request(VpMessageClientError::Http(_)))
        );
        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.first().unwrap(), WalletMessage::Request(_));
    }

    #[test]
    fn test_vp_disclosure_client_start_error_incorrect_client_id() {
        // Calling `VpDisclosureClient::start()` with a request URI object in which the `client_id`
        // does not match the one from the RP's certificate should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &["bsn", "given_name", "family_name"],
            |mut verifier_session| {
                verifier_session.request_uri_object.client_id = "other_client_id".to_string();

                verifier_session
            },
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(
            *error,
            VpSessionError::Verifier(VpVerifierError::IncorrectClientId {
                expected,
                ..
            }) if expected == *"other_client_id"
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        assert_matches!(&wallet_messages[0], WalletMessage::Request(_));
        // This error should be reported back to the verifier.
        let expected_error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest);
        assert_matches!(&wallet_messages[1], WalletMessage::Error(response) if response.error == expected_error_code);
    }

    #[test]
    fn test_vp_disclosure_client_start_error_missing_reader_registration() {
        // Calling `VpDisclosureClient::start()` with an Authorization Request JWT that contains
        // a valid reader certificate but no `ReaderRegistration` should result in an error.
        // Note that the test for `VpVerifierError::RpCertificate` is missing,
        // which is too convoluted of an error condition to simulate.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            CredentialFormat::MsoMdoc,
            &[],
            std::convert::identity,
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        assert_matches!(
            *error,
            VpSessionError::Verifier(VpVerifierError::MissingReaderRegistration)
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        assert_matches!(&wallet_messages[0], WalletMessage::Request(_));
        // This error should be reported back to the verifier.
        let expected_error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest);
        assert_matches!(&wallet_messages[1], WalletMessage::Error(response) if response.error == expected_error_code);
    }

    #[rstest]
    fn test_vp_disclosure_client_start_error_requested_attributes_validation(
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] credential_format: CredentialFormat,
    ) {
        // Calling `VpDisclosureClient::start()` where the Authorization Request contains
        // an attribute that is not in the `ReaderRegistration` should result in an error.
        let (error, verifier_session) = start_disclosure_session(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            RequestUriMethod::POST,
            None,
            credential_format,
            &["given_name", "family_name"],
            std::convert::identity,
        )
        .expect_err("starting a new disclosure session on VpDisclosureClient should not succeed");

        let unregistered_attributes = match credential_format {
            CredentialFormat::MsoMdoc => HashMap::from([(
                PID.to_string(),
                HashSet::from([vec![
                    ClaimPath::SelectByKey(PID.to_string()),
                    ClaimPath::SelectByKey("bsn".to_string()),
                ]
                .try_into()
                .unwrap()]),
            )]),
            CredentialFormat::SdJwt => HashMap::from([(
                PID.to_string(),
                HashSet::from([vec![ClaimPath::SelectByKey("bsn".to_string())].try_into().unwrap()]),
            )]),
        };
        assert_matches!(*error, VpSessionError::Verifier(VpVerifierError::RequestedAttributesValidation(
            ValidationError::UnregisteredAttributes(unregistered)
        )) if unregistered == unregistered_attributes);

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        assert_matches!(&wallet_messages[0], WalletMessage::Request(_));
        // This error should be reported back to the verifier.
        let expected_error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest);
        assert_matches!(&wallet_messages[1], WalletMessage::Error(response) if response.error == expected_error_code);
    }
}
