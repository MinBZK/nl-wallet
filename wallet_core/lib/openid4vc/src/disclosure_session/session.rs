use std::hash::Hash;

use itertools::Itertools;
use tracing::info;
use tracing::warn;

use crypto::CredentialEcdsaKey;
use crypto::utils::random_string;
use dcql::normalized::NormalizedCredentialRequest;
use http_utils::urls::BaseUrl;
use mdoc::holder::Mdoc;
use mdoc::iso::disclosure::DeviceResponse;
use mdoc::iso::engagement::SessionTranscript;
use utils::vec_at_least::VecNonEmpty;
use wscd::Poa;
use wscd::keyfactory::JwtPoaInput;
use wscd::keyfactory::KeyFactory;

use crate::openid4vp::NormalizedVpAuthorizationRequest;
use crate::openid4vp::VpAuthorizationResponse;
use crate::verifier::SessionType;

use super::DisclosureSession;
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

    fn credential_requests(&self) -> &VecNonEmpty<NormalizedCredentialRequest> {
        &self.auth_request.credential_requests
    }

    fn verifier_certificate(&self) -> &VerifierCertificate {
        &self.verifier_certificate
    }

    async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        let return_url = self.client.terminate(self.auth_request.response_uri).await?;

        Ok(return_url)
    }

    async fn disclose<K, KF>(
        self,
        mdocs: VecNonEmpty<Mdoc>,
        key_factory: &KF,
    ) -> Result<Option<BaseUrl>, (Self, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K, Poa = Poa>,
    {
        info!("disclose mdoc documents");

        let expected_attestation_count = self.auth_request.credential_requests.len();
        if mdocs.len() != expected_attestation_count {
            return Err((
                self,
                DisclosureError::before_sharing(
                    VpClientError::AttestationCountMismatch {
                        expected: expected_attestation_count.get(),
                        found: mdocs.len().get(),
                    }
                    .into(),
                ),
            ));
        }

        let subset_mdocs = mdocs
            .into_iter()
            .zip_eq(self.auth_request.credential_requests.as_ref())
            .map(|(mut mdoc, request)| {
                mdoc.issuer_signed = mdoc.issuer_signed.into_attribute_subset(request.claim_paths());

                mdoc
            })
            .collect_vec();

        // Sign Document values based on the remaining contents of these mdocs and retain the keys used for signing.
        info!("signing disclosed mdoc documents");

        let mdoc_nonce = random_string(32);
        let session_transcript = SessionTranscript::new_oid4vp(
            &self.auth_request.response_uri,
            &self.auth_request.client_id,
            self.auth_request.nonce.clone(),
            &mdoc_nonce,
        );

        let poa_input = JwtPoaInput::new(Some(mdoc_nonce.clone()), self.auth_request.client_id.clone());
        let result = DeviceResponse::sign_from_mdocs(subset_mdocs, &session_transcript, key_factory, poa_input).await;
        let (device_response, poa) = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    self,
                    DisclosureError::before_sharing(VpClientError::DeviceResponse(error).into()),
                ));
            }
        };

        // Finally, encrypt the response and send it to the verifier.
        let result = VpAuthorizationResponse::new_encrypted(device_response, &self.auth_request, &mdoc_nonce, poa);
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
    use std::sync::Arc;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use http::StatusCode;
    use http_utils::urls::BaseUrl;
    use itertools::Itertools;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde::de::Error;
    use serde_json::json;

    use attestation_data::auth::reader_auth::ReaderRegistration;
    use dcql::normalized::NormalizedCredentialRequest;
    use mdoc::holder::Mdoc;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use wscd::mock_remote::MockRemoteKeyFactory;

    use crate::errors::AuthorizationErrorCode;
    use crate::errors::VpAuthorizationErrorCode;
    use crate::openid4vp::RequestUriMethod;
    use crate::verifier::SessionType;

    use super::super::DisclosureSession;
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
        credential_request: impl IntoIterator<Item = NormalizedCredentialRequest>,
    ) -> (
        VpDisclosureSession<MockVerifierVpMessageClient>,
        Arc<MockVerifierSession>,
    ) {
        let session_type = SessionType::SameDevice;

        let mut verifier_session = MockVerifierSession::new(
            &VERIFIER_URL,
            session_type,
            RequestUriMethod::GET,
            redirect_uri,
            Some(ReaderRegistration::new_mock()),
        );
        verifier_session.credential_requests = credential_request.into_iter().collect_vec().try_into().unwrap();

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
        response_factory: F,
    ) -> VpDisclosureSession<MockErrorFactoryVpMessageClient<F>>
    where
        F: Fn() -> VpMessageClientError,
    {
        let (disclosure_session, _verifier_session) =
            setup_disclosure_session(None, [NormalizedCredentialRequest::new_pid_example()]);

        // Replace the `VpDisclosureSession`'s client with one that returns errors.
        let error_client = MockErrorFactoryVpMessageClient::new(response_factory, true);

        VpDisclosureSession {
            client: error_client,
            session_type: disclosure_session.session_type,
            verifier_certificate: disclosure_session.verifier_certificate,
            auth_request: disclosure_session.auth_request,
        }
    }

    fn setup_disclosure_mdoc() -> (Mdoc, MockRemoteKeyFactory) {
        let mdoc_key = MockRemoteEcdsaKey::new("mdoc_key".to_string(), SigningKey::random(&mut OsRng));
        let mdoc = Mdoc::new_mock_with_key(&mdoc_key).now_or_never().unwrap();
        let key_factory = MockRemoteKeyFactory::new(vec![mdoc_key]);

        (mdoc, key_factory)
    }

    /// This contains a lightweight test of `VpDisclosureSession::disclose()`. For a more
    /// thorough test see `test_vp_disclosure_client_full()` in the `client` submodule.
    #[rstest]
    fn test_disclosure_session_disclose_abridged(
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<BaseUrl>,
    ) {
        let (disclosure_session, verifier_session) =
            setup_disclosure_session(redirect_uri.clone(), [NormalizedCredentialRequest::new_pid_example()]);
        let (mdoc, key_factory) = setup_disclosure_mdoc();

        let disclosure_redirect_uri = disclosure_session
            .disclose(vec![mdoc].try_into().unwrap(), &key_factory)
            .now_or_never()
            .unwrap()
            .expect("disclosing mdoc using VpDisclosureSession should succeed");

        assert_eq!(disclosure_redirect_uri, redirect_uri);

        let wallet_messages = verifier_session.wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.last().unwrap(), WalletMessage::Disclosure(_));
    }

    #[test]
    fn test_disclosure_session_disclose_error_device_response() {
        // Calling `VPDisclosureSession::disclose()` with a malfunctioning key factory should result in an error.
        let (disclosure_session, _verifier_session) =
            setup_disclosure_session(None, [NormalizedCredentialRequest::new_pid_example()]);
        let (mdoc, mut key_factory) = setup_disclosure_mdoc();

        key_factory.has_multi_key_signing_error = true;

        let (_disclosure_session, error) = disclosure_session
            .disclose(vec![mdoc].try_into().unwrap(), &key_factory)
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared,
                error: VpSessionError::Client(VpClientError::DeviceResponse(_))
            } if !data_shared
        );
    }

    #[test]
    fn test_disclosure_session_disclose_error_auth_response_encryption() {
        // Calling `VPDisclosureSession::disclose()` with a malformed encryption key should result in an error.
        let (mut disclosure_session, _verifier_session) =
            setup_disclosure_session(None, [NormalizedCredentialRequest::new_pid_example()]);
        let (mdoc, key_factory) = setup_disclosure_mdoc();

        disclosure_session
            .auth_request
            .encryption_pubkey
            .set_parameter("kty", Some(json!("invalid_value")))
            .unwrap();

        let (_disclosure_session, error) = disclosure_session
            .disclose(vec![mdoc].try_into().unwrap(), &key_factory)
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared,
                error: VpSessionError::Client(VpClientError::AuthResponseEncryption(_))
            } if !data_shared
        );
    }

    /// Helper function for testing `VpDisclosureSession::disclose()` HTTP errors.
    fn test_disclosure_session_disclose_http_error<F>(response_factory: F) -> VpSessionError
    where
        F: Fn() -> VpMessageClientError,
    {
        let disclosure_session = setup_disclosure_session_http_error(response_factory);
        let (mdoc, key_factory) = setup_disclosure_mdoc();
        let wallet_messages = Arc::clone(&disclosure_session.client.wallet_messages);

        let (_disclosure_session, error) = disclosure_session
            .disclose(vec![mdoc].try_into().unwrap(), &key_factory)
            .now_or_never()
            .unwrap()
            .expect_err("disclosing mdoc using VpDisclosureSession should not succeed");

        let wallet_messages = wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        assert_matches!(wallet_messages.last().unwrap(), WalletMessage::Disclosure(_));

        assert!(error.data_shared);

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
        let (disclosure_session, verifier_session) =
            setup_disclosure_session(redirect_uri.clone(), [NormalizedCredentialRequest::new_pid_example()]);

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
        let disclosure_session = setup_disclosure_session_http_error(response_factory);
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
