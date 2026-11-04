use std::hash::Hash;

use attestation_data::organization::Organization;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::CredentialEcdsaKey;
use crypto::utils::random_string;
use crypto::wscd::DisclosureWscd;
use dcql::CredentialQueryIdentifier;
use dcql::normalized::NormalizedCredentialRequests;
use futures::TryFutureExt;
use itertools::Either;
use itertools::Itertools;
use jwe::algorithm::EncryptionAlgorithm;
use mdoc::iso::disclosure::DeviceResponse;
use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use tracing::info;
use tracing::warn;
use url::Url;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;

use super::DisclosableAttestations;
use super::DisclosureSession;
use super::NonEmptyDisclosableAttestations;
use super::error::DisclosureError;
use super::error::VpClientError;
use super::error::VpSessionError;
use super::message_client::VpMessageClient;
use crate::openid4vp::NormalizedVpAuthorizationRequest;
use crate::openid4vp::VerifiablePresentation;
use crate::openid4vp::VpAuthorizationResponse;
use crate::verifier::SessionType;

#[derive(Debug)]
pub struct VpDisclosureSession<H> {
    client: H,
    session_type: SessionType,
    organization: Organization,
    auth_request: NormalizedVpAuthorizationRequest,
    selected_encryption_algorithm: EncryptionAlgorithm,
}

impl<H> VpDisclosureSession<H> {
    pub(super) fn new(
        client: H,
        session_type: SessionType,
        organization: Organization,
        auth_request: NormalizedVpAuthorizationRequest,
        selected_encryption_algorithm: EncryptionAlgorithm,
    ) -> Self {
        Self {
            client,
            session_type,
            organization,
            auth_request,
            selected_encryption_algorithm,
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

    fn organization(&self) -> &Organization {
        &self.organization
    }

    async fn terminate(self) -> Result<Option<Url>, VpSessionError> {
        let return_url = self
            .client
            .terminate(self.auth_request.response_uri.clone(), self.auth_request.state.clone())
            .await?;

        Ok(return_url)
    }

    async fn disclose<K, W>(
        self,
        attestations: NonEmptyDisclosableAttestations,
        wscd: &W,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Option<Url>, (Box<Self>, DisclosureError<VpSessionError>)>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        W: DisclosureWscd<Key = K>,
    {
        info!("disclose attestations");
        // TODO (PVW-4780): This method assumes that the attestations passed to it only contain those attributes that
        //                  were requested to be disclosed. As the `Wallet` already has to perform this operation in
        //                  order to show the disclosure to the user, we decided to have it pass those reduced versions
        //                  of the attestations to `VpDisclosureSession`. However, this responsibility would be more
        //                  appropriately housed in the disclosure session itself. This could be resolved by introducing
        //                  a new type that this method takes which encapsulates a full source attestation and a list
        //                  of the attributes to be disclosed. This type then provides the canonical method of creating
        //                  intermediate types of the attestations that contain a subset of the attributes.

        if let Err(error) = attestations.check_against_credential_requests(&self.auth_request.credential_requests) {
            return Err((
                Box::new(self),
                DisclosureError::before_sharing(VpSessionError::Client(error)),
            ));
        }

        let encryption_nonce = random_string(32);

        // Split the attestations by format, as each format is signed in its own way.
        let (partial_mdocs, unsigned_presentations): (Vec<_>, Vec<_>) = attestations
            .into_inner()
            .into_iter()
            .partition_map(|(id, attestations)| match attestations {
                DisclosableAttestations::MsoMdoc(partial_mdocs) => Either::Left((id, partial_mdocs)),
                DisclosableAttestations::SdJwt(unsigned_presentations) => Either::Right((id, unsigned_presentations)),
            });

        // Have the WSCD sign all of the partial mdocs in one operation, if there are any.
        let device_responses = match sign_per_query(partial_mdocs, |partial_mdocs| async {
            info!("signing disclosed mdoc documents");
            let session_transcript = self.auth_request.session_transcript();
            DeviceResponse::sign_multiple_from_partial_mdocs(partial_mdocs, &session_transcript, wscd)
                .await
                .map_err(VpClientError::DeviceResponse)
        })
        .await
        {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    Box::new(self),
                    DisclosureError::before_sharing(VpSessionError::Client(error)),
                ));
            }
        };

        // Have the WSCD sign all of the unsigned presentations in one operation, if there are any.
        //
        // Note that when both mdocs and SD-JWTs are disclosed, the code below results in a second signing operation to
        // the WSCD on top of the one made above, even though nothing about such an instruction precludes it from
        // containing the payloads for both formats. Combining them would require restructuring the code, as the WSCD
        // is currently passed into format-specific code in the `mdoc` and `sd_jwt` crates. As disclosing a mix of
        // formats is expected to be rare, we accept that the user has to wait slightly longer in that case.
        //
        // The two signing operations are deliberately performed sequentially instead of in parallel: the instruction
        // client holds a lock on the database while incrementing the instruction sequence number, so the instructions
        // would be executed in sequence anyway.
        let signed_presentations = match sign_per_query(unsigned_presentations, |unsigned_presentations| {
            info!("signing disclosed SD-JWT documents");
            let key_binding_builder =
                KeyBindingJwtBuilder::new(self.auth_request.client_id.to_string(), self.auth_request.nonce.clone());
            UnsignedSdJwtPresentation::sign_multiple(unsigned_presentations, key_binding_builder, wscd, time)
                .map_err(VpClientError::SdJwtSigning)
        })
        .await
        {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    Box::new(self),
                    DisclosureError::before_sharing(VpSessionError::Client(error)),
                ));
            }
        };

        // Merge the signed attestations of both formats back into a single map. The `CredentialQueryIdentifier` keys
        // cannot collide, as both lists were partitioned from the same map above.
        let vp_token = device_responses
            .into_iter()
            .map(|(id, device_responses)| (id, VerifiablePresentation::MsoMdoc(device_responses)))
            .chain(signed_presentations.into_iter().map(|(id, presentations)| {
                let presentations = presentations
                    .into_nonempty_iter()
                    .map(|p| p.into_unverified())
                    .collect();

                (id, VerifiablePresentation::SdJwt(presentations))
            }))
            .collect();

        // Finally, encrypt the response and send it to the verifier.
        let result = VpAuthorizationResponse::new_encrypted(
            vp_token,
            &self.auth_request,
            self.selected_encryption_algorithm,
            &encryption_nonce,
        );
        let jwe = match result {
            Ok(value) => value,
            Err(error) => {
                return Err((
                    Box::new(self),
                    DisclosureError::before_sharing(VpSessionError::Client(VpClientError::AuthResponseEncryption(
                        error,
                    ))),
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
            Err(error) => return Err((Box::new(self), error.into())),
        };

        info!("sending Authorization Response succeeded");

        Ok(redirect_uri)
    }
}

/// Lay out the attestations for all credential queries in a single linear list, have them all signed in one operation
/// by calling `sign`, and then group the results by credential query again. If there are no attestations at all,
/// `sign` is not called and an empty `Vec` is returned.
///
/// Note that `sign` MUST return the signed attestations in the same order as it received them.
async fn sign_per_query<T, S, F, Fut>(
    attestations: Vec<(CredentialQueryIdentifier, VecNonEmpty<T>)>,
    sign: F,
) -> Result<Vec<(CredentialQueryIdentifier, VecNonEmpty<S>)>, VpClientError>
where
    F: FnOnce(VecNonEmpty<T>) -> Fut,
    Fut: Future<Output = Result<VecNonEmpty<S>, VpClientError>>,
    S: std::fmt::Debug,
{
    // Remember the identifiers and attestation counts, as we will need these to reconstruct the grouping later.
    let (id_and_counts, attestations): (Vec<_>, Vec<_>) = attestations
        .into_iter()
        .map(|(id, attestations)| ((id, attestations.len()), attestations))
        .unzip();
    let attestations = attestations.into_iter().flatten().collect_vec();

    let Ok(attestations) = VecNonEmpty::try_from(attestations) else {
        return Ok(Vec::new());
    };

    let expected = attestations.len();
    let signed = sign(attestations).await?;

    let received = signed.len();
    if received != expected {
        return Err(VpClientError::SignedAttestationCountMismatch { expected, received });
    }

    let mut signed = signed.into_iter();
    id_and_counts
        .into_iter()
        .map(|(id, count)| {
            // Given the check above, each group receives exactly `count` signed attestations, which is non-zero.
            let group = signed.by_ref().take(count.get()).collect_vec();
            let group = VecNonEmpty::try_from(group).expect("count is non-zero and the total count was checked above");

            Ok((id, group))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::LazyLock;

    use attestation_data::organization::Organization;
    use attestation_types::claim_path::ClaimPath;
    use attestation_types::credential_format::Format;
    use crypto::keys::PublicKey;
    use crypto::mock_remote::DisclosureMockRemoteWscd;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use dcql::normalized::NormalizedCredentialRequests;
    use futures::FutureExt;
    use http::StatusCode;
    use http_utils::urls::BaseUrl;
    use itertools::Itertools;
    use jwe::algorithm::EncryptionAlgorithm;
    use mdoc::holder::disclosure::PartialMdoc;
    use rstest::rstest;
    use sd_jwt::builder::SignedSdJwt;
    use serde::de::Error;
    use url::Url;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

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
    use super::VpDisclosureSession;
    use super::sign_per_query;
    use crate::disclosure_session::error::DataDisclosed;
    use crate::errors::VpAuthorizationErrorCode;
    use crate::openid4vp::VpRequestUriMethod;
    use crate::verifier::SessionType;

    static VERIFIER_URL: LazyLock<BaseUrl> = LazyLock::new(|| "http://cert.rp.example.com/disclosure".parse().unwrap());

    /// Creates a `VpDisclosureSession` that has already been started, along with a `MockVerifierSession`.
    fn setup_disclosure_session(
        redirect_uri: Option<Url>,
        credential_requests: NormalizedCredentialRequests,
    ) -> (
        VpDisclosureSession<MockVerifierVpMessageClient>,
        Arc<MockVerifierSession>,
    ) {
        let session_type = SessionType::SameDevice;

        let verifier_session = MockVerifierSession::new(
            &VERIFIER_URL,
            session_type,
            VpRequestUriMethod::GET,
            redirect_uri,
            credential_requests,
        );

        let verifier_session = Arc::new(verifier_session);

        let mock_client = MockVerifierVpMessageClient::new(Arc::clone(&verifier_session));
        let auth_request = verifier_session.normalized_auth_request(None);
        let selected_encryption_algorithm = EncryptionAlgorithm::A256Gcm;
        let disclosure_session = VpDisclosureSession {
            client: mock_client,
            session_type,
            organization: Organization::new_mock(),
            auth_request,
            selected_encryption_algorithm,
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
            organization: disclosure_session.organization,
            auth_request: disclosure_session.auth_request,
            selected_encryption_algorithm: disclosure_session.selected_encryption_algorithm,
        }
    }

    fn setup_disclosure_mdoc() -> (
        NormalizedCredentialRequests,
        NonEmptyDisclosableAttestations,
        DisclosureMockRemoteWscd,
    ) {
        let requests = NormalizedCredentialRequests::new_mock_mdoc_pid_example();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let mdoc_key = MockRemoteEcdsaKey::new_random("mdoc_key".to_string());

        let partial_mdoc = PartialMdoc::new_mock_with_ca_and_key(&ca, &mdoc_key);
        let attestations = HashMap::from([(
            "mdoc_pid_example".try_into().unwrap(),
            DisclosableAttestations::MsoMdoc(vec_nonempty![(partial_mdoc, "mdoc_key".to_string())]),
        )])
        .try_into()
        .unwrap();

        let wscd = DisclosureMockRemoteWscd::new(vec![mdoc_key]);

        (requests, attestations, wscd)
    }

    fn setup_disclosure_sd_jwt() -> (
        NormalizedCredentialRequests,
        NonEmptyDisclosableAttestations,
        DisclosureMockRemoteWscd,
    ) {
        let requests = NormalizedCredentialRequests::new_mock_sd_jwt_pid_example();

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key_pair = ca.generate_issuer_mock().unwrap();
        let sd_jwt_key = MockRemoteEcdsaKey::new_random("sd_jwt_key".to_string());
        let sd_jwt_public_key = *sd_jwt_key.verifying_key();
        let wscd = DisclosureMockRemoteWscd::new(vec![sd_jwt_key]);

        let verified_sd_jwt =
            SignedSdJwt::pid_example(&issuer_key_pair, &PublicKey::from(sd_jwt_public_key)).into_verified();
        let unsigned_presentation = verified_sd_jwt
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())])
            .unwrap()
            .finish();

        let attestations = HashMap::from([(
            "sd_jwt_pid_example".try_into().unwrap(),
            DisclosableAttestations::SdJwt(vec_nonempty![(unsigned_presentation, "sd_jwt_key".to_string())]),
        )])
        .try_into()
        .unwrap();

        (requests, attestations, wscd)
    }

    /// This contains a lightweight test of `VpDisclosureSession::disclose()`. For a more
    /// thorough test see `test_vp_disclosure_client_full()` in the `client` submodule.
    #[rstest]
    fn test_disclosure_session_disclose_abridged(
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<Url>,
        #[values(Format::MsoMdoc, Format::SdJwt)] credential_format: Format,
    ) {
        let (requests, attestations, wscd) = match credential_format {
            Format::MsoMdoc => setup_disclosure_mdoc(),
            Format::SdJwt => setup_disclosure_sd_jwt(),
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

        wscd.has_multi_key_signing_error = true;

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

        wscd.has_multi_key_signing_error = true;

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
    fn test_disclosure_session_disclose_error_unknown_credential_query() {
        // Calling `VPDisclosureSession::disclose()` with attestations for a credential query that was not part of the
        // request should result in an error.
        let (requests, _, _) = setup_disclosure_mdoc();
        let (_, attestations, wscd) = setup_disclosure_sd_jwt();
        let (disclosure_session, verifier_session) = setup_disclosure_session(None, requests);

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared: DataDisclosed::NotDisclosed,
                error: VpSessionError::Client(VpClientError::UnknownCredentialQuery)
            }
        );
        assert!(verifier_session.wallet_messages.lock().is_empty());
    }

    #[test]
    fn test_disclosure_session_disclose_error_attestation_format_mismatch() {
        // Calling `VPDisclosureSession::disclose()` with attestations of a different format than the one requested in
        // the credential query should result in an error.
        let (requests, _, _) = setup_disclosure_mdoc();
        let (_, attestations, wscd) = setup_disclosure_sd_jwt();
        let (disclosure_session, verifier_session) = setup_disclosure_session(None, requests);

        // Provide the SD-JWT attestations as the answer to the mdoc credential query.
        let attestations = attestations
            .into_inner()
            .into_values()
            .map(|attestations| ("mdoc_pid_example".try_into().unwrap(), attestations))
            .collect::<HashMap<_, _>>()
            .try_into()
            .unwrap();

        let (_disclosure_session, error) = disclosure_session
            .disclose(attestations, &wscd, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .expect_err("disclosing using VpDisclosureSession should not succeed");

        assert_matches!(
            error,
            DisclosureError {
                data_shared: DataDisclosed::NotDisclosed,
                error: VpSessionError::Client(VpClientError::AttestationFormatMismatch {
                    requested: Format::MsoMdoc,
                    provided: Format::SdJwt,
                })
            }
        );
        assert!(verifier_session.wallet_messages.lock().is_empty());
    }

    #[rstest]
    #[case::too_few(2)]
    #[case::too_many(4)]
    fn test_sign_per_query_error_signed_attestation_count_mismatch(#[case] received_count: usize) {
        // When `sign` returns a different number of signed attestations than it received, `sign_per_query()` should
        // result in an error.
        let attestations = vec![
            ("query_1".try_into().unwrap(), vec_nonempty![1, 2]),
            ("query_2".try_into().unwrap(), vec_nonempty![3]),
        ];

        let error = sign_per_query(attestations, |attestations| async move {
            let signed = attestations.into_iter().cycle().take(received_count).collect_vec();

            Ok(VecNonEmpty::try_from(signed).unwrap())
        })
        .now_or_never()
        .unwrap()
        .expect_err("signing per query should not succeed");

        assert_matches!(
            error,
            VpClientError::SignedAttestationCountMismatch { expected, received }
                if expected.get() == 3 && received.get() == received_count
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
        #[values(None, Some("http://example.com/redirect".parse().unwrap()))] redirect_uri: Option<Url>,
    ) {
        let (mut disclosure_session, verifier_session) = setup_disclosure_session(
            redirect_uri.clone(),
            NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
        );
        disclosure_session.auth_request.state = Some("authorization_state".to_string());

        let terminate_redirect_uri = disclosure_session
            .terminate()
            .now_or_never()
            .unwrap()
            .expect("terminating VpDisclosureSession should succeed");

        assert_eq!(terminate_redirect_uri, redirect_uri);

        // Terminating the session should result in the verified receiving an access denied error.
        let wallet_messages = verifier_session.wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        let expected_error_code = VpAuthorizationErrorCode::AccessDenied;
        assert_matches!(
            wallet_messages.last().unwrap(),
            WalletMessage::Error(response)
                if response.error() == &expected_error_code
                    && response.state.as_deref() == Some("authorization_state")
        );
    }

    /// Helper function for testing `VpDisclosureSession::terminate()` HTTP errors.
    fn test_disclosure_session_terminate_http_error<F>(response_factory: F) -> VpSessionError
    where
        F: Fn() -> VpMessageClientError,
    {
        let mut disclosure_session = setup_disclosure_session_http_error(
            NormalizedCredentialRequests::new_mock_mdoc_pid_example(),
            response_factory,
        );
        disclosure_session.auth_request.state = Some("authorization_state".to_string());
        let wallet_messages = Arc::clone(&disclosure_session.client.wallet_messages);

        // Terminate the session, which should result in the verified receiving an access denied error.
        let error = disclosure_session
            .terminate()
            .now_or_never()
            .unwrap()
            .expect_err("terminating VpDisclosureSession should not succeed");

        let wallet_messages = wallet_messages.lock();

        assert_eq!(wallet_messages.len(), 1);
        let expected_error_code = VpAuthorizationErrorCode::AccessDenied;
        assert_matches!(
            wallet_messages.last().unwrap(),
            WalletMessage::Error(response)
                if response.error() == &expected_error_code
                    && response.state.as_deref() == Some("authorization_state")
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
