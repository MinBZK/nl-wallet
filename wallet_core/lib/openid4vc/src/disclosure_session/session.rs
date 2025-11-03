use std::collections::VecDeque;
use std::hash::Hash;

use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use tracing::info;
use tracing::warn;

use crypto::CredentialEcdsaKey;
use crypto::utils::random_string;
use crypto::wscd::DisclosureWscd;
use dcql::normalized::NormalizedCredentialRequests;
use http_utils::urls::BaseUrl;
use mdoc::iso::disclosure::DeviceResponse;
use mdoc::iso::engagement::SessionTranscript;
use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use utils::generator::Generator;
use wscd::Poa;
use wscd::wscd::JwtPoaInput;

use crate::openid4vp::NormalizedVpAuthorizationRequest;
use crate::openid4vp::VerifiablePresentation;
use crate::openid4vp::VpAuthorizationResponse;
use crate::verifier::SessionType;

use super::DisclosableAttestations;
use super::DisclosureSession;
use super::NonEmptyDisclosableAttestations;
use super::VerifierCertificate;
use super::error::DisclosureError;
use super::error::VpClientError;
use super::error::VpSessionError;
use super::message_client::VpMessageClient;

#[derive(Debug)]
pub struct VpDisclosureSession<H> {
    client: H,
    session_type: SessionType,
    verifier_certificate: VerifierCertificate,
    auth_request: NormalizedVpAuthorizationRequest,
}

impl<H> VpDisclosureSession<H> {
    pub(super) fn new(
        client: H,
        session_type: SessionType,
        verifier_certificate: VerifierCertificate,
        auth_request: NormalizedVpAuthorizationRequest,
    ) -> Self {
        Self {
            client,
            session_type,
            verifier_certificate,
            auth_request,
        }
    }
}

impl<H> DisclosureSession for VpDisclosureSession<H>
where
    H: VpMessageClient,
{
    fn session_type(&self) -> SessionType {
        self.session_type
    }

    fn credential_requests(&self) -> &NormalizedCredentialRequests {
        &self.auth_request.credential_requests
    }

    fn verifier_certificate(&self) -> &VerifierCertificate {
        &self.verifier_certificate
    }

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        let return_url = self.client.terminate(self.auth_request.response_uri).await?;

        Ok(return_url)
    }

    async fn disclose<K, W>(
        self,
        attestations: NonEmptyDisclosableAttestations,
        wscd: &W,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K, Poa = Poa>,
    {
        info!("disclose mdoc documents");
        // TODO (PVW-4780): This method assumes that the attestations passed to it only contain those attributes that
        //                  were requested to be disclosed. As the `Wallet` already has to perform this operation in
        //                  order to show the disclosure to the user, we decided to have it pass those reduced versions
        //                  of the attestations to `VpDisclosureSession`. However, this responsibility would be more
        //                  appropriately housed in the disclosure session itself. This could be resolved by introducing
        //                  a new type that this method takes which encapsulates a full source attestation and a list
        //                  of the attributes to be disclosed. This type then provides the canonical method of creating
        //                  intermediate types of the attestations that contain a subset of the attributes.

        let encryption_nonce = random_string(32);
        let poa_input = JwtPoaInput::new(
            Some(self.auth_request.nonce.clone()),
            self.auth_request.client_id.clone(),
        );

        let (vp_token, poa) = match attestations.into_inner() {
            DisclosableAttestations::MsoMdoc(partial_mdocs_map) => {
                info!("signing disclosed mdoc documents");

                // Lay out all of the partial mdocs in a linear `Vec`, but remember the keys and
                // attestation counts of the `HashMap`, as we will need to reconstruct this later.
                let mut id_and_counts = Vec::with_capacity(partial_mdocs_map.len());
                let partial_mdocs = partial_mdocs_map
                    .into_iter()
                    .flat_map(|(id, partial_mdocs)| {
                        id_and_counts.push((id, partial_mdocs.len()));

                        partial_mdocs
                    })
                    .collect_vec()
                    .try_into()
                    // This unwrap to `VecNonEmpty` is safe, as the `NonEmptyDisclosableAttestations`
                    // type guarantees that it contains at least one attestation.
                    .unwrap();

                let session_transcript = SessionTranscript::new_oid4vp(
                    &self.auth_request.response_uri,
                    &self.auth_request.client_id,
                    self.auth_request.nonce.clone(),
                    &encryption_nonce,
                );

                // Have the WSCD sign all of the partial mdocs in one operation,
                // producing a PoA if multiple unique keys are used for this.
                let result = DeviceResponse::sign_multiple_from_partial_mdocs(
                    partial_mdocs,
                    &session_transcript,
                    wscd,
                    poa_input,
                )
                .await;
                let (received_device_responses, poa) = match result {
                    Ok(value) => value,
                    Err(error) => {
                        return Err((
                            self,
                            DisclosureError::before_sharing(VpClientError::DeviceResponse(error).into()),
                        ));
                    }
                };

                // Reconstruct a `HashMap` from the identifier and `DeviceResponse`s.
                let mut received_device_responses = VecDeque::from(received_device_responses.into_inner());
                let vp_token = id_and_counts
                    .into_iter()
                    .map(|(id, count)| {
                        // Note that:
                        // * The `drain()`` is guaranteed not to panic as the returned `DeviceRespones` should have
                        //   exactly the same count as the amount of partial mdocs that we submitted for signing.
                        // * The .`unwrap()` is guaranteed to succeed, as the count is non-zero.
                        let responses = received_device_responses
                            .drain(..count.get())
                            .collect_vec()
                            .try_into()
                            .unwrap();

                        (id, VerifiablePresentation::MsoMdoc(responses))
                    })
                    .collect();

                (vp_token, poa)
            }
            DisclosableAttestations::SdJwt(unsigned_presentations_map) => {
                info!("signing disclosed SD-JWT documents");

                // Lay out all of the SD-JWT presentations in a linear `Vec`, but remember the keys and
                // attestation counts of the `HashMap`, as we will need to reconstruct this later.
                let mut id_and_counts = Vec::with_capacity(unsigned_presentations_map.len());
                let unsigned_presentations = unsigned_presentations_map
                    .into_iter()
                    .flat_map(|(id, unsigned_presentations_and_key_ids)| {
                        id_and_counts.push((id, unsigned_presentations_and_key_ids.len()));

                        unsigned_presentations_and_key_ids
                    })
                    .collect_vec()
                    .try_into()
                    // This unwrap to `VecNonEmpty` is safe, as the `NonEmptyDisclosableAttestations`
                    // type guarantees that it contains at least one attestation.
                    .unwrap();

                // Have the WSCD sign all of the unsigned presentations in one operation,
                // producing a PoA if multiple unique keys are used for this.
                let key_binding_builder =
                    KeyBindingJwtBuilder::new(self.auth_request.client_id.clone(), self.auth_request.nonce.clone());
                let result = UnsignedSdJwtPresentation::sign_multiple(
                    unsigned_presentations,
                    key_binding_builder,
                    wscd,
                    poa_input,
                    time,
                )
                .await;
                let (signed_presentations, poa) = match result {
                    Ok(value) => value,
                    Err(error) => {
                        return Err((
                            self,
                            DisclosureError::before_sharing(VpClientError::SdJwtSigning(error).into()),
                        ));
                    }
                };

                // Reconstruct a `HashMap` from the identifier and `SdJwtPresentation`s.
                let mut received_presentations = VecDeque::from(signed_presentations.into_inner());
                let vp_token = id_and_counts
                    .into_iter()
                    .map(|(id, count)| {
                        // Note that:
                        // * The `drain()`` is guaranteed not to panic as the returned `DeviceRespones` should have
                        //   exactly the same count as the amount of unsigned presentations that we submitted for
                        //   signing.
                        // * The .`unwrap()` is guaranteed to succeed, as the count is non-zero.
                        let presentations = received_presentations
                            .drain(..count.get())
                            .map(|presentation| presentation.into_unverified())
                            .collect_vec()
                            .try_into()
                            .unwrap();

                        (id, VerifiablePresentation::SdJwt(presentations))
                    })
                    .collect();
                (vp_token, poa)
            }
        };

        // Finally, encrypt the response and send it to the verifier.
        let result = VpAuthorizationResponse::new_encrypted(vp_token, &self.auth_request, &encryption_nonce, poa);
        let jwe = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    self,
                    DisclosureError::before_sharing(VpClientError::AuthResponseEncryption(error).into()),
                ));
            }
        };

        info!("sending Authorization Response to verifier");

        let result = self
            .client
            .send_authorization_response(self.auth_request.response_uri.clone(), jwe)
            .await
            .inspect_err(|err| {
                warn!("sending Authorization Response failed: {err}");
            });
        let redirect_uri = match result {
            Ok(value) => value,
            Err(error) => return Err((self, error.into())),
        };

        info!("sending Authorization Response succeeded");

        Ok(redirect_uri)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use http::StatusCode;
    use http_utils::urls::BaseUrl;
    use rstest::rstest;
    use serde::de::Error;
    use serde_json::json;

    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_types::claim_path::ClaimPath;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::x509::CertificateUsage;
    use dcql::CredentialFormat;
    use dcql::normalized::NormalizedCredentialRequests;
    use mdoc::holder::disclosure::PartialMdoc;
    use sd_jwt::builder::SignedSdJwt;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::mock_remote::MockRemoteWscd;

    use crate::disclosure_session::error::DataDisclosed;
    use crate::errors::AuthorizationErrorCode;
    use crate::errors::VpAuthorizationErrorCode;
    use crate::openid4vp::RequestUriMethod;
    use crate::verifier::SessionType;

    use super::super::DisclosableAttestations;
    use super::super::DisclosureSession;
    use super::super::NonEmptyDisclosableAttestations;
    use super::super::error::DisclosureError;
    use super::super::error::VpClientError;
    use super::super::error::VpSessionError;
    use super::super::error::VpVerifierError;
    use super::super::message_client::VpMessageClientError;
    use super::super::message_client::mock::MockErrorFactoryVpMessageClient;
    use super::super::message_client::mock::MockVerifierSession;
    use super::super::message_client::mock::MockVerifierVpMessageClient;
    use super::super::message_client::mock::WalletMessage;
    use super::super::verifier_certificate::VerifierCertificate;
    use super::VpDisclosureSession;

    static VERIFIER_URL: LazyLock<BaseUrl> = LazyLock::new(|| "http://example.com/disclosure".parse().unwrap());

    /// Creates a `VpDisclosureSession` that has already been started, along with a `MockVerifierSession`.
    fn setup_disclosure_session(
        redirect_uri: Option<BaseUrl>,
        credential_requests: NormalizedCredentialRequests,
    ) -> (
        VpDisclosureSession<MockVerifierVpMessageClient>,
        Arc<MockVerifierSession>,
    ) {
        let session_type = SessionType::SameDevice;

        let verifier_session = MockVerifierSession::new(
            &VERIFIER_URL,
            session_type,
            RequestUriMethod::GET,
            redirect_uri,
            Some(ReaderRegistration::new_mock()),
            credential_requests,
        );

        let verifier_session = Arc::new(verifier_session);

        let mock_client = MockVerifierVpMessageClient::new(Arc::clone(&verifier_session));
        let disclosure_session = VpDisclosureSession {
            client: mock_client,
            session_type,
            verifier_certificate: VerifierCertificate::try_new(verifier_session.key_pair.certificate().clone())
                .unwrap()
                .unwrap(),
            auth_request: verifier_session.normalized_auth_request(None),
        };

        (disclosure_session, verifier_session)
    }

    /// Creates a `VpDisclosureSession` that has already been started where the verified will return a HTTP error.
    fn setup_disclosure_session_http_error<F>(
        credential_requests: NormalizedCredentialRequests,
        response_factory: F,
    ) -> VpDisclosureSession<MockErrorFactoryVpMessageClient<F>>
    where
        F: Fn() -> VpMessageClientError,
    {
        let (disclosure_session, _verifier_session) = setup_disclosure_session(None, credential_requests);

        // Replace the `VpDisclosureSession`'s client with one that returns errors.
        let error_client = MockErrorFactoryVpMessageClient::new(response_factory, true);

        VpDisclosureSession {
            client: error_client,
            session_type: disclosure_session.session_type,
            verifier_certificate: disclosure_session.verifier_certificate,
            auth_request: disclosure_session.auth_request,
        }
    }

    fn setup_disclosure_mdoc() -> (
        NormalizedCredentialRequests,
        NonEmptyDisclosableAttestations,
        MockRemoteWscd,
    ) {
        let requests = NormalizedCredentialRequests::new_mock_mdoc_pid_example();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let mdoc_key = MockRemoteEcdsaKey::new_random("mdoc_key".to_string());

        let partial_mdoc = PartialMdoc::new_mock_with_ca_and_key(&ca, &mdoc_key);
        let attestations = DisclosableAttestations::MsoMdoc(HashMap::from([(
            "mdoc_pid_example".try_into().unwrap(),
            vec_nonempty![partial_mdoc],
        )]))
        .try_into()
        .unwrap();

        let wscd = MockRemoteWscd::new(vec![mdoc_key]);

        (requests, attestations, wscd)
    }

    fn setup_disclosure_sd_jwt() -> (
        NormalizedCredentialRequests,
        NonEmptyDisclosableAttestations,
        MockRemoteWscd,
    ) {
        let requests = NormalizedCredentialRequests::new_mock_sd_jwt_pid_example();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key_pair = ca
            .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
            .unwrap();
        let sd_jwt_key = MockRemoteEcdsaKey::new_random("sd_jwt_key".to_string());
        let sd_jwt_public_key = *sd_jwt_key.verifying_key();
        let wscd = MockRemoteWscd::new(vec![sd_jwt_key]);

        let verified_sd_jwt = SignedSdJwt::pid_example(&issuer_key_pair, &sd_jwt_public_key).into_verified();
        let unsigned_presentation = verified_sd_jwt
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())])
            .unwrap()
            .finish();

        let attestations = DisclosableAttestations::SdJwt(HashMap::from([(
            "sd_jwt_pid_example".try_into().unwrap(),
            vec_nonempty![(unsigned_presentation, "sd_jwt_key".to_string())],
        )]))
        .try_into()
        .unwrap();

        (requests, attestations, wscd)
    }

    /// This contains a lightweight test of `VpDisclosureSession::disclose()`. For a more
    /// thorough test see `test_vp_disclosure_client_full()` in the `client` submodule.
    #[rstest]
    fn test_disclosure_session_disclose_abridged(
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<BaseUrl>,
        #[values(CredentialFormat::MsoMdoc, CredentialFormat::SdJwt)] credential_format: CredentialFormat,
    ) {
        let (requests, attestations, wscd) = match credential_format {
            CredentialFormat::MsoMdoc => setup_disclosure_mdoc(),
            CredentialFormat::SdJwt => setup_disclosure_sd_jwt(),
        };

        let (disclosure_session, verifier_session) = setup_disclosure_session(redirect_uri.clone(), requests);

        let disclosure_redirect_uri = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect("disclosing attestation using VpDisclosureSession should succeed");

        assert_eq!(disclosure_redirect_uri, redirect_uri);

        let wallet_messages = verifier_session.wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.last().unwrap(), WalletMessage::Disclosure(_));
    }

    #[test]
    fn test_disclosure_session_disclose_error_device_response() {
        // Calling `VPDisclosureSession::disclose()` with a malfunctioning WSCD should result in an error.
        let (requests, attestations, mut wscd) = setup_disclosure_mdoc();
        let (disclosure_session, _verifier_session) = setup_disclosure_session(None, requests);

        wscd.disclosure.has_multi_key_signing_error = true;

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared: DataDisclosed::NotDisclosed,
                error: VpSessionError::Client(VpClientError::DeviceResponse(_))
            }
        );
    }

    #[test]
    fn test_disclosure_session_disclose_error_sd_jwt_presentation() {
        // Calling `VPDisclosureSession::disclose()` with a malfunctioning WSCD should result in an error.
        let (requests, attestations, mut wscd) = setup_disclosure_sd_jwt();
        let (disclosure_session, _verifier_session) = setup_disclosure_session(None, requests);

        wscd.disclosure.has_multi_key_signing_error = true;

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing SD-JWT using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared: DataDisclosed::NotDisclosed,
                error: VpSessionError::Client(VpClientError::SdJwtSigning(_))
            }
        );
    }

    #[test]
    fn test_disclosure_session_disclose_error_auth_response_encryption() {
        // Calling `VPDisclosureSession::disclose()` with a malformed encryption key should result in an error.
        let (requests, attestations, wscd) = setup_disclosure_mdoc();
        let (mut disclosure_session, _verifier_session) = setup_disclosure_session(None, requests);

        disclosure_session
            .auth_request
            .encryption_pubkey
            .set_parameter("kty", Some(json!("invalid_value")))
            .unwrap();

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared: DataDisclosed::NotDisclosed,
                error: VpSessionError::Client(VpClientError::AuthResponseEncryption(_))
            }
        );
    }

    /// Helper function for testing `VpDisclosureSession::disclose()` HTTP errors.
    fn test_disclosure_session_disclose_http_error<F>(response_factory: F) -> VpSessionError
    where
        F: Fn() -> VpMessageClientError,
    {
        let (requests, attestations, wscd) = setup_disclosure_mdoc();
        let disclosure_session = setup_disclosure_session_http_error(requests, response_factory);
        let wallet_messages = Arc::clone(&disclosure_session.client.wallet_messages);

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        let wallet_messages = wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.last().unwrap(), WalletMessage::Disclosure(_));

        assert!(error.data_shared == DataDisclosed::Disclosed);

        error.error
    }

    #[test]
    fn test_disclosure_session_disclose_error_verifier_request() {
        let error = test_disclosure_session_disclose_http_error(|| serde_json::Error::custom("").into());

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
    }

    #[test]
    fn test_disclosure_session_disclose_error_client_request() {
        let error = test_disclosure_session_disclose_http_error(|| {
            let response = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("")
                .unwrap();

            reqwest::Response::from(response).error_for_status().unwrap_err().into()
        });

        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::Request(VpMessageClientError::Http(_)))
        );
    }

    #[rstest]
    fn test_disclosure_session_terminate(
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<BaseUrl>,
    ) {
        let (disclosure_session, verifier_session) = setup_disclosure_session(
            redirect_uri.clone(),
            NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
        );

        let terminate_redirect_uri = disclosure_session
            .terminate()
            .now_or_never()
            .unwrap()
            .expect("terminating VpDisclosureSession should succeed");

        assert_eq!(terminate_redirect_uri, redirect_uri);

        // Terminating the session should result in the verified receiving an access denied error.
        let wallet_messages = verifier_session.wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        let expected_error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied);
        assert_matches!(
            wallet_messages.last().unwrap(),
            WalletMessage::Error(response) if response.error == expected_error_code
        );
    }

    /// Helper function for testing `VpDisclosureSession::terminate()` HTTP errors.
    fn test_disclosure_session_terminate_http_error<F>(response_factory: F) -> VpSessionError
    where
        F: Fn() -> VpMessageClientError,
    {
        let disclosure_session = setup_disclosure_session_http_error(
            NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
            response_factory,
        );
        let wallet_messages = Arc::clone(&disclosure_session.client.wallet_messages);

        // Terminate the session, which should result in the verified receiving an access denied error.
        let error = disclosure_session
            .terminate()
            .now_or_never()
            .unwrap()
            .expect_err("terminating VpDisclosureSession should not succeed");

        let wallet_messages = wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        let expected_error_code = VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied);
        assert_matches!(
            wallet_messages.last().unwrap(),
            WalletMessage::Error(response) if response.error == expected_error_code
        );

        error
    }

    #[test]
    fn test_disclosure_session_terminate_error_verifier_request() {
        let error = test_disclosure_session_terminate_http_error(|| serde_json::Error::custom("").into());

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
    }

    #[test]
    fn test_disclosure_session_terminate_error_client_request() {
        let error = test_disclosure_session_terminate_http_error(|| {
            let response = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("")
                .unwrap();

            reqwest::Response::from(response).error_for_status().unwrap_err().into()
        });

        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::Request(VpMessageClientError::Http(_)))
        );
    }
}
