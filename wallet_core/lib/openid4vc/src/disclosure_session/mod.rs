use std::hash::Hash;

use futures::TryFutureExt;
use itertools::Itertools;
use rustls_pki_types::TrustAnchor;
use tracing::info;
use tracing::warn;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::identifiers::AttributeIdentifier;
use attestation_data::x509::CertificateType;
use crypto::factory::KeyFactory;
use crypto::keys::CredentialEcdsaKey;
use crypto::utils::random_string;
use crypto::x509::BorrowingCertificate;
use http_utils::urls::BaseUrl;
use mdoc::disclosure::DeviceResponse;
use mdoc::engagement::SessionTranscript;
use mdoc::holder::DisclosureRequestMatch;
use mdoc::holder::MdocDataSource;
use mdoc::holder::ProposedAttributes;
use mdoc::holder::ProposedDocument;
use poa::factory::PoaFactory;
use utils::vec_at_least::VecAtLeastTwoUnique;

use crate::errors::AuthorizationErrorCode;
use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::IsoVpAuthorizationRequest;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpAuthorizationResponse;
use crate::openid4vp::VpRequestUriObject;
use crate::verifier::SessionType;
use crate::verifier::VerifierUrlParameters;

pub use self::client::HttpVpMessageClient;
pub use self::client::VpMessageClient;
pub use self::client::VpMessageClientError;
pub use self::client::VpMessageClientErrorType;
pub use self::client::APPLICATION_OAUTH_AUTHZ_REQ_JWT;
pub use self::error::DisclosureError;
pub use self::error::VpClientError;
pub use self::error::VpSessionError;
pub use self::error::VpVerifierError;
pub use self::uri_source::DisclosureUriSource;

mod client;
mod error;
mod uri_source;

#[derive(Debug)]
pub enum DisclosureSession<H, I> {
    MissingAttributes(DisclosureMissingAttributes<H>),
    Proposal(DisclosureProposal<H, I>),
}

#[derive(Debug)]
pub struct DisclosureMissingAttributes<H> {
    data: CommonDisclosureData<H>,
    missing_attributes: Vec<AttributeIdentifier>,
}

#[derive(Debug)]
pub struct DisclosureProposal<H, I> {
    data: CommonDisclosureData<H>,
    proposed_documents: Vec<ProposedDocument<I>>,
    mdoc_nonce: String,
}

#[derive(Debug)]
struct CommonDisclosureData<H> {
    client: H,
    certificate: BorrowingCertificate,
    reader_registration: ReaderRegistration,
    auth_request: IsoVpAuthorizationRequest,
    session_type: SessionType,
}

enum VerifierSessionDataCheckResult<I> {
    MissingAttributes(Vec<AttributeIdentifier>),
    ProposedDocuments(Vec<ProposedDocument<I>>),
}

impl<H, I> DisclosureSession<H, I>
where
    H: VpMessageClient,
{
    pub async fn start<S>(
        client: H,
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Self, VpSessionError>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
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
            RequestUriMethod::POST => Some(random_string(32)),
        };

        let jws = client
            .get_authorization_request(request_uri_object.request_uri.clone(), request_nonce.clone())
            .await?;

        let (vp_auth_request, certificate) = VpAuthorizationRequest::try_new(&jws, trust_anchors)?;
        let response_uri = vp_auth_request.response_uri.clone();

        // Use async here so we get the async-version of .or_else(), as report_error_back() is async.
        let auth_request = async { vp_auth_request.validate(&certificate, request_nonce.as_deref()) }
            .or_else(|error| async {
                match response_uri {
                    None => Err(error.into()), // just return the error if we don't know the URL to report it to
                    Some(response_uri) => Self::report_error_back(error.into(), &client, response_uri).await,
                }
            })
            .await?;

        let mdoc_nonce = random_string(32);
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri,
            &auth_request.client_id,
            auth_request.nonce.clone(),
            &mdoc_nonce,
        );

        let (check_result, reader_registration) = Self::process_request(
            &auth_request,
            &certificate,
            &session_transcript,
            &request_uri_object,
            mdoc_data_source,
        )
        .or_else(|error| Self::report_error_back(error, &client, auth_request.response_uri.clone()))
        .await?;

        let data = CommonDisclosureData {
            client,
            certificate,
            reader_registration,
            auth_request,
            session_type,
        };

        // Create the appropriate `DisclosureSession` invariant, which contains
        // all of the information needed to either abort of finish the session.
        let session = match check_result {
            VerifierSessionDataCheckResult::MissingAttributes(missing_attributes) => {
                DisclosureSession::MissingAttributes(DisclosureMissingAttributes {
                    data,
                    missing_attributes,
                })
            }
            VerifierSessionDataCheckResult::ProposedDocuments(proposed_documents) => {
                DisclosureSession::Proposal(DisclosureProposal {
                    data,
                    proposed_documents,
                    mdoc_nonce,
                })
            }
        };

        Ok(session)
    }

    /// Report an error back to the RP. Note: this function only reports errors that are the RP's fault.
    async fn report_error_back<T>(error: VpSessionError, client: &H, url: BaseUrl) -> Result<T, VpSessionError> {
        let error_code = match error {
            VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(_))
            | VpSessionError::Verifier(VpVerifierError::IncorrectClientId { .. })
            | VpSessionError::Verifier(VpVerifierError::MissingReaderRegistration)
            | VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
            | VpSessionError::Verifier(VpVerifierError::RequestedAttributesValidation(_))
            | VpSessionError::Verifier(VpVerifierError::RpCertificate(_)) => {
                VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest)
            }
            _ => return Err(error), // don't report other errors
        };

        let error_response = ErrorResponse {
            error: error_code,
            error_description: Some(error.to_string()),
            error_uri: None,
        };

        // If sending the error results in an error, log it but do nothing else.
        let _ = client
            .send_error(url, error_response)
            .await
            .inspect_err(|err| warn!("failed to send error to server: {err}"));

        Err(error)
    }

    /// Internal helper function for processing and checking the Authorization Request,
    /// including checking whether or not we have the requested attributes.
    async fn process_request<S>(
        auth_request: &IsoVpAuthorizationRequest,
        certificate: &BorrowingCertificate,
        session_transcript: &SessionTranscript,
        request_uri_object: &VpRequestUriObject,
        mdoc_data_source: &S,
    ) -> Result<(VerifierSessionDataCheckResult<I>, ReaderRegistration), VpSessionError>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        // The `client_id` in the Authorization Request, which has been authenticated, has to equal
        // the `client_id` that the RP sent in the Request URI object at the start of the session.
        if auth_request.client_id != request_uri_object.client_id {
            return Err(VpVerifierError::IncorrectClientId {
                expected: request_uri_object.client_id.clone(),
                found: auth_request.client_id.clone(),
            }
            .into());
        }

        // Extract `ReaderRegistration` from the certificate.
        let reader_registration = match CertificateType::from_certificate(certificate)? {
            CertificateType::ReaderAuth(Some(reader_registration)) => *reader_registration,
            _ => return Err(VpVerifierError::MissingReaderRegistration.into()),
        };

        // Verify that the requested attributes are included in the reader authentication.
        reader_registration.verify_requested_attributes(&auth_request.items_requests.as_ref().iter())?;

        // Fetch documents from the database, calculate which ones satisfy the request and
        // formulate proposals for those documents. If there is a mismatch, return an error.
        let candidates_by_doc_type = match DisclosureRequestMatch::new(
            auth_request.items_requests.as_ref().iter(),
            mdoc_data_source,
            session_transcript,
        )
        .await
        .map_err(VpClientError::MatchRequestedAttributes)?
        {
            DisclosureRequestMatch::Candidates(candidates) => candidates,
            DisclosureRequestMatch::MissingAttributes(missing_attributes) => {
                // Attributes are missing, return these.
                let result = VerifierSessionDataCheckResult::MissingAttributes(missing_attributes);
                return Ok((result, reader_registration));
            }
        };

        // If we have multiple candidates for any of the doc types, return an error.
        // TODO: Support having the user choose between multiple candidates. (PVW-1392)
        if candidates_by_doc_type.values().any(|candidates| candidates.len() > 1) {
            let duplicate_doc_types = candidates_by_doc_type
                .into_iter()
                .filter(|(_, candidates)| candidates.len() > 1)
                .map(|(doc_type, _)| doc_type)
                .collect();

            return Err(VpClientError::MultipleCandidates(duplicate_doc_types).into());
        }

        // Now that we know that we have exactly one candidate for every `doc_type`,
        // we can flatten these candidates to a 1-dimensional `Vec`.
        let proposed_documents = candidates_by_doc_type.into_values().flatten().collect_vec();
        let result = VerifierSessionDataCheckResult::ProposedDocuments(proposed_documents);

        Ok((result, reader_registration))
    }

    fn data(&self) -> &CommonDisclosureData<H> {
        match self {
            DisclosureSession::MissingAttributes(session) => &session.data,
            DisclosureSession::Proposal(session) => &session.data,
        }
    }

    fn into_data(self) -> CommonDisclosureData<H> {
        match self {
            DisclosureSession::MissingAttributes(session) => session.data,
            DisclosureSession::Proposal(session) => session.data,
        }
    }

    pub fn reader_registration(&self) -> &ReaderRegistration {
        &self.data().reader_registration
    }

    pub fn verifier_certificate(&self) -> &BorrowingCertificate {
        &self.data().certificate
    }

    pub fn session_type(&self) -> SessionType {
        self.data().session_type
    }

    pub async fn terminate(self) -> Result<Option<BaseUrl>, VpSessionError> {
        let data = self.into_data();
        let return_url = data.client.terminate(data.auth_request.response_uri).await?;

        Ok(return_url)
    }
}

impl<H> DisclosureMissingAttributes<H> {
    pub fn missing_attributes(&self) -> &[AttributeIdentifier] {
        &self.missing_attributes
    }
}

impl<H, I> DisclosureProposal<H, I>
where
    H: VpMessageClient,
    I: Clone,
{
    pub fn proposed_source_identifiers(&self) -> Vec<&I> {
        self.proposed_documents
            .iter()
            .map(|document| &document.source_identifier)
            .collect()
    }

    pub fn proposed_attributes(&self) -> ProposedAttributes {
        // Get all of the attributes to be disclosed from the
        // prepared `IssuerSigned` on the `ProposedDocument`s.
        self.proposed_documents
            .iter()
            .map(|document| (document.doc_type.clone(), document.proposed_attributes()))
            .collect()
    }

    pub async fn disclose<K, KF>(&self, key_factory: &KF) -> Result<Option<BaseUrl>, DisclosureError<VpClientError>>
    where
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
    {
        info!("disclose proposed documents");

        // Clone the proposed documents and construct a `DeviceResponse` by
        // signing these, then encrypt the response to the RP's public key.
        let proposed_documents = self.proposed_documents.clone();

        info!("sign proposed documents");

        let (device_response, keys) = DeviceResponse::from_proposed_documents(proposed_documents, key_factory)
            .await
            .map_err(|err| DisclosureError::before_sharing(VpClientError::DeviceResponse(err)))?;

        let poa = match VecAtLeastTwoUnique::try_from(keys) {
            Ok(keys) => {
                info!("create Proof of Association");

                // Poa::new() needs a vec of references. We can unwrap because we only get here if the conversion was
                // successful.
                let keys = VecAtLeastTwoUnique::try_from(keys.as_slice().iter().collect_vec()).unwrap();
                let poa = key_factory
                    .poa(
                        keys,
                        self.data.auth_request.client_id.clone(),
                        Some(self.mdoc_nonce.clone()),
                    )
                    .await
                    .map_err(|e| DisclosureError::before_sharing(VpClientError::Poa(Box::new(e))))?;
                Some(poa)
            }
            Err(_) => None,
        };

        info!("serialize and encrypt Authorization Response");

        let jwe =
            VpAuthorizationResponse::new_encrypted(device_response, &self.data.auth_request, &self.mdoc_nonce, poa)
                .map_err(|err| DisclosureError::before_sharing(VpClientError::AuthResponseEncryption(err)))?;

        info!("send Authorization Response to verifier");

        let redirect_uri = self
            .data
            .client
            .send_authorization_response(self.data.auth_request.response_uri.clone(), jwe)
            .await
            .inspect_err(|err| {
                warn!("sending Authorization Response failed: {err}");
            })?;

        info!("sending Authorization Response succeeded");
        Ok(redirect_uri)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::identity;
    use std::hash::Hash;
    use std::iter;
    use std::str::FromStr;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use indexmap::IndexSet;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use p256::ecdsa::Signature;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use parking_lot::Mutex;
    use rand_core::OsRng;
    use reqwest::StatusCode;
    use rstest::rstest;
    use serde::ser::Error;
    use serde_json::json;

    use attestation_data::auth::reader_auth::ReaderRegistration;
    use attestation_data::auth::reader_auth::ValidationError;
    use attestation_data::identifiers::AttributeIdentifier;
    use attestation_data::identifiers::AttributeIdentifierHolder;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use attestation_data::x509::CertificateType;
    use crypto::factory::KeyFactory;
    use crypto::keys::CredentialEcdsaKey;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::mock_remote::MockRemoteKeyFactory;
    use crypto::mock_remote::MockRemoteKeyFactoryError;
    use crypto::server_keys::generate::Ca;
    use crypto::utils::random_string;
    use crypto::x509::CertificateConfiguration;
    use crypto::x509::CertificateError;
    use http_utils::urls::BaseUrl;
    use jwt::error::JwtX5cError;
    use mdoc::examples::EXAMPLE_ATTRIBUTES;
    use mdoc::examples::EXAMPLE_DOC_TYPE;
    use mdoc::examples::EXAMPLE_NAMESPACE;
    use mdoc::holder::mock::MdocDataSourceError;
    use mdoc::holder::mock::MockMdocDataSource;
    use mdoc::holder::HolderError;
    use mdoc::holder::ProposedDocument;
    use mdoc::utils::cose::ClonePayload;
    use mdoc::utils::serialization::cbor_deserialize;
    use mdoc::utils::serialization::cbor_serialize;
    use mdoc::utils::serialization::CborBase64;
    use mdoc::utils::serialization::CborSeq;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::DeviceAuth;
    use mdoc::DeviceAuthenticationKeyed;
    use mdoc::ItemsRequest;
    use mdoc::MobileSecurityObject;
    use mdoc::SessionTranscript;
    use poa::factory::PoaFactory;
    use poa::Poa;
    use utils::vec_at_least::VecAtLeastTwoUnique;

    use crate::disclosure_session::VpSessionError;
    use crate::disclosure_session::VpVerifierError;
    use crate::errors::AuthorizationErrorCode;
    use crate::errors::VpAuthorizationErrorCode;
    use crate::openid4vp::AuthRequestValidationError;
    use crate::openid4vp::IsoVpAuthorizationRequest;
    use crate::openid4vp::VerifiablePresentation;
    use crate::openid4vp::VpAuthorizationRequest;
    use crate::openid4vp::VpAuthorizationResponse;
    use crate::openid4vp::VpClientMetadata;
    use crate::openid4vp::VpJwks;
    use crate::openid4vp::VpRequestUriObject;
    use crate::verifier::SessionType;

    use super::client::mock::request_uri_object;
    use super::client::mock::MockErrorFactoryVpMessageClient;
    use super::client::mock::MockVerifierSession;
    use super::client::mock::MockVerifierVpMessageClient;
    use super::client::mock::WalletMessage;
    use super::client::VpMessageClient;
    use super::client::VpMessageClientError;
    use super::error::DisclosureError;
    use super::error::VpClientError;
    use super::uri_source::DisclosureUriSource;
    use super::CommonDisclosureData;
    use super::DisclosureMissingAttributes;
    use super::DisclosureProposal;
    use super::DisclosureSession;

    // Constants for testing.
    const VERIFIER_URL: &str = "http://example.com/disclosure";

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
        (VpSessionError, Arc<MockVerifierSession<FD>>),
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

        let client = MockVerifierVpMessageClient::new(Arc::clone(&verifier_session));

        let ca = Ca::generate_issuer_mock_ca().unwrap();
        // Set up the mock data source.
        let mdoc_data_source = transform_mdoc(MockMdocDataSource::new_example_resigned(&ca).await);

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

    // This is the full happy path test of `DisclosureSession`.
    #[tokio::test]
    async fn test_disclosure_session() {
        // Starting a disclosure session should succeed.
        let (disclosure_session, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            identity,
            identity,
        )
        .await
        .expect("Could not start DisclosureSession");

        // Remember the `AttributeIdentifier`s that were in the request.
        let request_identifiers = verifier_session
            .items_requests
            .as_ref()
            .iter()
            .flat_map(|items_request| items_request.attribute_identifiers())
            .collect::<IndexSet<_>>();

        // Make sure starting the session resulted in a proposal.
        let DisclosureSession::Proposal(proposal) = disclosure_session else {
            panic!("Disclosure session should not have missing attributes");
        };

        // Extract the public keys from the `MobileSecurityObject` to verify the disclosed documents against later.
        let public_keys: Vec<VerifyingKey> = proposal
            .proposed_documents
            .iter()
            .map(|proposed_document| {
                // Can't use MdocCose::dangerous_parse_unverified() here as it is private
                let TaggedBytes(mso): TaggedBytes<MobileSecurityObject> = cbor_deserialize(
                    proposed_document
                        .issuer_signed
                        .issuer_auth
                        .0
                        .payload
                        .as_ref()
                        .unwrap()
                        .as_slice(),
                )
                .unwrap();

                (&mso.device_key_info.device_key).try_into().unwrap()
            })
            .collect();

        let redirect_uri = proposal
            .disclose(&MockRemoteKeyFactory::new_example())
            .await
            .expect("Could not disclose DisclosureSession");

        // Test if the return `Url` matches the input.
        let redirect_uri = redirect_uri.as_ref().unwrap();
        let expected_redirect_uri = verifier_session.redirect_uri.as_ref().unwrap();
        assert_eq!(redirect_uri, expected_redirect_uri);

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);

        // Decrypt the disclosure and extract the contained disclosed documents.
        let jwe = wallet_messages.last().unwrap().disclosure();
        let (mut response, mdoc_nonce) =
            VpAuthorizationResponse::decrypt(jwe, &verifier_session.encryption_keypair, &verifier_session.nonce)
                .unwrap();
        let device_response = match response.vp_token.pop().unwrap() {
            VerifiablePresentation::MsoMdoc(CborBase64(device_response)) => device_response,
        };
        let documents = device_response
            .documents
            .expect("No documents contained in DeviceResponse");
        assert_eq!(documents.len(), public_keys.len());

        // Check that the attributes contained in the response match those in the request.
        let response_identifiers = documents
            .iter()
            .flat_map(|document| document.issuer_signed_attribute_identifiers())
            .collect::<IndexSet<_>>();
        assert_eq!(response_identifiers, request_identifiers);

        let session_transcript = SessionTranscript::new_oid4vp(
            &verifier_session.response_uri,
            verifier_session.client_id(),
            verifier_session.nonce.clone(),
            &mdoc_nonce,
        );

        // Use the `SessionTranscript` to reconstruct the `DeviceAuthentication`
        // for every `Document` received in order to verify the signatures received
        // for each of these.
        documents
            .into_iter()
            .zip(public_keys)
            .for_each(|(document, public_key)| {
                let device_authentication = DeviceAuthenticationKeyed::new(&document.doc_type, &session_transcript);
                let device_authentication_bytes = cbor_serialize(&TaggedBytes(CborSeq(device_authentication))).unwrap();

                match document.device_signed.device_auth {
                    DeviceAuth::DeviceSignature(signature) => signature
                        .clone_with_payload(device_authentication_bytes)
                        .verify(&public_key)
                        .expect("Device authentication for document does not match public key"),
                    _ => panic!("Unexpected device authentication in DeviceResponse"),
                }
            });
    }

    #[tokio::test]
    async fn test_disclosure_session_start_proposal() {
        // Starting a disclosure session should succeed with a disclosure proposal.
        let (disclosure_session, verifier_session) = disclosure_session_start(
            SessionType::CrossDevice,
            DisclosureUriSource::QrCode,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            identity,
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let DisclosureSession::Proposal(ref proposal_session) = disclosure_session else {
            panic!("Disclosure session should not have missing attributes")
        };

        // Check that the wallet sent a nonce to be included in the Authorization Request JWT.
        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 1);
        assert!(wallet_messages.first().unwrap().request().wallet_nonce.is_some());

        // Test that the proposal session contains the example mdoc identifier.
        assert_eq!(proposal_session.proposed_source_identifiers(), ["id_1"]);

        // Test that the proposal for disclosure contains the example attributes, in order.
        // Note that `swap_remove()` is used to quickly gain ownership of the `Entry`s
        // contained within the proposed attributes for the example doc_type and namespace.
        let entry_keys = proposal_session
            .proposed_attributes()
            .swap_remove(EXAMPLE_DOC_TYPE)
            .and_then(|mut name_space| name_space.attributes.swap_remove(EXAMPLE_NAMESPACE))
            .map(|entries| entries.into_iter().map(|entry| entry.name).collect::<Vec<_>>())
            .unwrap_or_default();

        assert_eq!(entry_keys, EXAMPLE_ATTRIBUTES);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_missing_attributes_one() {
        // Starting a disclosure session should succeed with missing attributes.
        let (disclosure_session, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            |mut mdoc_source| {
                // Remove the last attribute from the first mdoc.
                mdoc_source
                    .mdocs
                    .first_mut()
                    .unwrap()
                    .0
                    .modify_attributes(EXAMPLE_NAMESPACE, |attributes| {
                        attributes.pop();
                    });

                mdoc_source
            },
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let DisclosureSession::MissingAttributes(ref missing_attr_session) = disclosure_session else {
            panic!("Disclosure session should have missing attributes")
        };

        // Test if `ReaderRegistration` matches the input.
        assert_eq!(
            disclosure_session.reader_registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 1);
        _ = wallet_messages.first().unwrap().request();

        let expected_missing_attributes =
            AttributeIdentifier::new_example_index_set_from_attributes(["driving_privileges"]);

        itertools::assert_equal(
            missing_attr_session.missing_attributes().iter(),
            expected_missing_attributes.iter(),
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_missing_attributes_all() {
        // Starting a disclosure session should succeed with missing attributes.
        let (disclosure_session, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            |mut mdoc_source| {
                mdoc_source.mdocs.clear();
                mdoc_source
            },
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let DisclosureSession::MissingAttributes(ref missing_attr_session) = disclosure_session else {
            panic!("Disclosure session should have missing attributes")
        };

        // Test if `ReaderRegistration` matches the input.
        assert_eq!(
            disclosure_session.reader_registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 1);
        _ = wallet_messages.first().unwrap().request();

        let expected_missing_attributes = AttributeIdentifier::new_example_index_set_from_attributes([
            "family_name",
            "issue_date",
            "expiry_date",
            "document_number",
            "driving_privileges",
        ]);

        itertools::assert_equal(
            missing_attr_session.missing_attributes().iter(),
            expected_missing_attributes.iter(),
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_decode_request_uri() {
        // Starting a `DisclosureSession` with an invalid request URI object should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                verifier_session.request_uri_override = Some("".to_string());

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, VpSessionError::Client(VpClientError::RequestUri(_)));
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_verifier_url_mising() {
        // Starting a `DisclosureSession` with a request URI object that
        // does not contain a request URI should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                let mut params: IndexMap<String, String> =
                    serde_urlencoded::from_str(&verifier_session.request_uri_query()).unwrap();
                params.swap_remove("request_uri");

                verifier_session.request_uri_override = Some(serde_urlencoded::to_string(params).unwrap());
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, VpSessionError::Client(VpClientError::RequestUri(_)));
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_missing_session_type() {
        // Starting a `DisclosureSession` with a request URI object that contains
        // a request URI without a session_type should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                // Overwrite the verifier URL with a version that does not have the `session_type` added.
                let mut request_uri_object: VpRequestUriObject =
                    serde_urlencoded::from_str(&verifier_session.request_uri_query()).unwrap();
                let mut request_uri_params: IndexMap<String, String> =
                    serde_urlencoded::from_str(request_uri_object.request_uri.as_ref().query().unwrap()).unwrap();
                request_uri_params.swap_remove("session_type");
                let mut request_uri = request_uri_object.request_uri.clone().into_inner();
                request_uri.set_query(Some(&serde_urlencoded::to_string(request_uri_params).unwrap()));
                request_uri_object.request_uri = request_uri.try_into().unwrap();

                verifier_session.request_uri_object = request_uri_object;
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::MalformedSessionType(_))
        );
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_malformed_session_type() {
        // Starting a `DisclosureSession` with a request URI object that contains
        // a request URI with an invalid session_type should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                let mut request_uri_object: VpRequestUriObject =
                    serde_urlencoded::from_str(&verifier_session.request_uri_query()).unwrap();
                request_uri_object.request_uri = format!("{}?session_type=invalid", VERIFIER_URL).parse().unwrap();

                verifier_session.request_uri_object = request_uri_object;
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::MalformedSessionType(_))
        );
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[rstest]
    #[case(SessionType::SameDevice, DisclosureUriSource::QrCode)]
    #[case(SessionType::CrossDevice, DisclosureUriSource::Link)]
    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_engagement_source_mismatch(
        #[case] session_type: SessionType,
        #[case] uri_source: DisclosureUriSource,
    ) {
        // Starting a `DisclosureSession` with a request URI object that contains a
        // `SessionType` that is incompatible with its source should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            session_type,
            uri_source,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::DisclosureUriSourceMismatch(
                typ,
                source
            )) if typ == session_type && source == uri_source);
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    async fn test_disclosure_session_start_error_http_client<F>(
        error_factory: F,
    ) -> (VpSessionError, Vec<WalletMessage>)
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

    #[tokio::test]
    async fn test_disclosure_session_start_error_http_client_data_serialization() {
        let (error, wallet_messages) =
            test_disclosure_session_start_error_http_client(|| Some(serde_json::Error::custom("").into())).await;

        // Trying to start a session in which the transport gives a JSON error
        // should result in the error being forwarded.
        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
        assert_eq!(wallet_messages.len(), 1);
        _ = wallet_messages.first().unwrap().request();
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_http_client_request() {
        let (error, wallet_messages) = test_disclosure_session_start_error_http_client(|| {
            let response = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("")
                .unwrap();
            let reqwest_error = reqwest::Response::from(response).error_for_status().unwrap_err();

            Some(VpMessageClientError::Http(reqwest_error))
        })
        .await;

        // Trying to start a session in which the transport gives a HTTP error
        // should result in the error being forwarded.
        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::Request(VpMessageClientError::Http(_)))
        );
        assert_eq!(wallet_messages.len(), 1);
        _ = wallet_messages.first().unwrap().request();
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_request_uri_without_query() {
        // Starting a `DisclosureSession` with a request URI without query parameters
        // should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                let mut request_uri = verifier_session.request_uri_object.request_uri.clone().into_inner();
                request_uri.set_query(None);
                verifier_session.request_uri_object.request_uri = request_uri.try_into().unwrap();
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, VpSessionError::Verifier(VpVerifierError::MissingSessionType));
        assert_eq!(verifier_session.wallet_messages.lock().len(), 0);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_incorrect_client_id() {
        // Starting a `DisclosureSession` with a request URI object in which the `client_id`
        // does not match the one from the RP's certificate should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                verifier_session.request_uri_object.client_id = "client_id_from_request_uri_object".to_string();
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::IncorrectClientId {
                expected,
                ..
            }) if expected == *"client_id_from_request_uri_object"
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        _ = wallet_messages.first().unwrap().request();
        _ = wallet_messages.last().unwrap().error(); // This RP error should be reported back to the RP
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_verifier_encryption_missing() {
        // Starting a `DisclosureSession` with a `VpAuthorizationRequest` without an encryption public key
        // should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            identity,
            |mut auth_request| {
                let VpClientMetadata::Direct(ref mut client_metadata) = auth_request.client_metadata.as_mut().unwrap()
                else {
                    panic!("client_metadata should not be indirect");
                };
                client_metadata.jwks = VpJwks::Direct { keys: vec![] };

                auth_request
            },
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(
                AuthRequestValidationError::UnexpectedJwkAmount(0)
            ))
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        _ = wallet_messages.first().unwrap().request();
        _ = wallet_messages.last().unwrap().error(); // This RP error should be reported back to the RP
    }

    #[rstest]
    #[case(vec![])]
    #[case(vec![ItemsRequest::new_example_empty()])]
    #[tokio::test]
    async fn test_disclosure_session_start_error_no_attributes_requested(#[case] items_requests: Vec<ItemsRequest>) {
        // Starting a `DisclosureSession` with an Authorization Request with no
        // `DocRequest` entries is received should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                verifier_session.items_requests = items_requests.into();
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(
                AuthRequestValidationError::NoAttributesRequested
            ))
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        _ = wallet_messages.first().unwrap().request();
        _ = wallet_messages.last().unwrap().error(); // This RP error should be reported back to the RP
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_auth_invalid() {
        // Starting a `DisclosureSession` without trust anchors should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                verifier_session.trust_anchors.clear();
                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::AuthRequestValidation(
                AuthRequestValidationError::JwtVerification(JwtX5cError::CertificateValidation(
                    CertificateError::Verification(_)
                ))
            ))
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 1);
        _ = wallet_messages.first().unwrap().request();
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_registration_validation() {
        // Starting a `DisclosureSession` where the Authorization Request contains an attribute
        // that is not in the `ReaderRegistration` should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            |mut verifier_session| {
                verifier_session
                    .items_requests
                    .0
                    .first_mut()
                    .unwrap()
                    .name_spaces
                    .get_mut(EXAMPLE_NAMESPACE)
                    .unwrap()
                    .insert("foobar".to_string(), false);

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        let unregistered_attribute = AttributeIdentifier {
            credential_type: "org.iso.18013.5.1.mDL".to_string(),
            namespace: "org.iso.18013.5.1".to_string(),
            attribute: "foobar".to_string(),
        };
        assert_matches!(error, VpSessionError::Verifier(VpVerifierError::RequestedAttributesValidation(
            ValidationError::UnregisteredAttributes(ids)
        )) if ids == vec![unregistered_attribute]);

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        _ = wallet_messages.first().unwrap().request();
        _ = wallet_messages.last().unwrap().error(); // This RP error should be reported back to the RP
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_mdoc_data_source() {
        // Starting a `DisclosureSession` when the database returns
        // an error should result in that error being forwarded.
        let (error, _) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            |mut mdoc_source| {
                mdoc_source.has_error = true;
                mdoc_source
            },
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::MatchRequestedAttributes(mdoc::Error::Holder(
                HolderError::MdocDataSource(mdoc_error)
            ))) if mdoc_error.is::<MdocDataSourceError>()
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_multiple_candidates() {
        // Starting a `DisclosureSession` when the database contains multiple
        // candidates for the same `doc_type` should result in an error.
        let (error, _) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::WithReaderRegistration,
            identity,
            |mut mdoc_source| {
                mdoc_source.mdocs.push(mdoc_source.mdocs.first().unwrap().clone());
                mdoc_source
            },
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Client(VpClientError::MultipleCandidates(doc_types)) if doc_types == vec![EXAMPLE_DOC_TYPE.to_string()]
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_no_reader_registration() {
        // Starting a `DisclosureSession` with an Authorization Request JWT that contains a valid
        // reader certificate but no `ReaderRegistration` should result in an error.
        let (error, verifier_session) = disclosure_session_start(
            SessionType::SameDevice,
            DisclosureUriSource::Link,
            ReaderCertificateKind::NoReaderRegistration,
            identity,
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");
        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::MissingReaderRegistration)
        );

        let wallet_messages = verifier_session.wallet_messages.lock();
        assert_eq!(wallet_messages.len(), 2);
        _ = wallet_messages.first().unwrap().request();
        _ = wallet_messages.last().unwrap().error(); // This RP error should be reported back to the RP
    }

    fn iso_auth_request() -> IsoVpAuthorizationRequest {
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let key_pair = generate_reader_mock(
            &ca,
            Some(ReaderRegistration {
                attributes: ReaderRegistration::create_attributes(
                    EXAMPLE_DOC_TYPE.to_string(),
                    EXAMPLE_NAMESPACE.to_string(),
                    EXAMPLE_ATTRIBUTES.iter().copied(),
                ),
                ..ReaderRegistration::new_mock()
            }),
        )
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

    #[allow(clippy::type_complexity)]
    async fn create_disclosure_session_proposal<F>(
        response_factory: F,
        device_key: &MockRemoteEcdsaKey,
    ) -> (
        DisclosureSession<MockErrorFactoryVpMessageClient<F>, String>,
        Arc<Mutex<Vec<WalletMessage>>>,
    )
    where
        F: Fn() -> Option<VpMessageClientError>,
    {
        let session_type = SessionType::SameDevice;

        let wallet_messages = Arc::new(Mutex::new(Vec::new()));
        let client = MockErrorFactoryVpMessageClient::new(response_factory, Arc::clone(&wallet_messages));

        let mdoc_nonce = random_string(32);

        let ca = Ca::generate("my_ca", CertificateConfiguration::default()).unwrap();
        let mock_key_pair = ca
            .generate_key_pair(
                "mock_keypair",
                CertificateType::ReaderAuth(None),
                CertificateConfiguration::default(),
            )
            .unwrap();

        let proposal_session = DisclosureSession::Proposal(DisclosureProposal {
            data: CommonDisclosureData {
                client,
                certificate: mock_key_pair.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                session_type,
                auth_request: iso_auth_request(),
            },
            proposed_documents: vec![ProposedDocument::new_mock_with_key(device_key).await],
            mdoc_nonce,
        });

        (proposal_session, wallet_messages)
    }

    async fn test_disclosure_session_terminate<H>(
        session: DisclosureSession<H, String>,
        wallet_messages: Arc<Mutex<Vec<WalletMessage>>>,
    ) -> Result<Option<BaseUrl>, VpSessionError>
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

    #[tokio::test]
    async fn test_disclosure_session_proposal_terminate() {
        let key_factory = MockRemoteKeyFactory::default();
        let device_key = key_factory.generate_new().await.unwrap();
        let (proposal_session, wallet_messages) = create_disclosure_session_proposal(|| None, &device_key).await;

        // Terminating a `DisclosureSession` with a proposal should succeed.
        test_disclosure_session_terminate(proposal_session, wallet_messages)
            .await
            .expect("Could not terminate DisclosureSession with proposal");
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_terminate_json_error() {
        let key_factory = MockRemoteKeyFactory::default();
        let device_key = key_factory.generate_new().await.unwrap();

        let (proposal_session, wallet_messages) = create_disclosure_session_proposal(
            || Some(VpMessageClientError::Json(serde_json::Error::custom(""))),
            &device_key,
        )
        .await;

        // Terminating a `DisclosureSession` with a proposal where the `VpMessageClient`
        // gives an error should result in that error being forwarded.
        let error = test_disclosure_session_terminate(proposal_session, wallet_messages)
            .await
            .expect_err("Terminating DisclosureSession with proposal should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
    }

    fn missing_attributes_session<F>(
        client: MockErrorFactoryVpMessageClient<F>,
    ) -> DisclosureSession<MockErrorFactoryVpMessageClient<F>, String>
    where
        F: Fn() -> Option<VpMessageClientError>,
    {
        let ca = Ca::generate_reader_mock_ca().unwrap();
        let mock_key_pair = generate_reader_mock(&ca, None).unwrap();
        DisclosureSession::MissingAttributes(DisclosureMissingAttributes {
            data: CommonDisclosureData {
                client,
                certificate: mock_key_pair.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                session_type: SessionType::SameDevice,
                auth_request: iso_auth_request(),
            },
            missing_attributes: Default::default(),
        })
    }

    #[tokio::test]
    async fn test_disclosure_session_missing_attributes_terminate() {
        let wallet_messages = Arc::new(Mutex::new(Vec::new()));
        let client = MockErrorFactoryVpMessageClient::new(|| None, Arc::clone(&wallet_messages));

        // Terminating a `DisclosureSession` with missing attributes should succeed.
        let missing_attr_session = missing_attributes_session(client);

        test_disclosure_session_terminate(missing_attr_session, wallet_messages)
            .await
            .expect("Could not terminate DisclosureSession with missing attributes");
    }

    #[tokio::test]
    async fn test_disclosure_session_missing_attributes_terminate_json_error() {
        let wallet_messages = Arc::new(Mutex::new(Vec::new()));
        let client = MockErrorFactoryVpMessageClient::new(
            || Some(VpMessageClientError::Json(serde_json::Error::custom(""))),
            Arc::clone(&wallet_messages),
        );

        let missing_attr_session = missing_attributes_session(client);

        // Terminating a `DisclosureSession` with missing attributes where the
        // `VpMessageClient` gives an error should result in that error being forwarded.
        let error = test_disclosure_session_terminate(missing_attr_session, wallet_messages)
            .await
            .expect_err("Terminating DisclosureSession with missing attributes should have resulted in an error");

        assert_matches!(
            error,
            VpSessionError::Verifier(VpVerifierError::Request(VpMessageClientError::Json(_)))
        );
    }

    async fn try_disclose<F, K, KF>(
        proposal_session: DisclosureSession<MockErrorFactoryVpMessageClient<F>, String>,
        wallet_messages: Arc<Mutex<Vec<WalletMessage>>>,
        key_factory: &KF,
        expect_report_error: bool,
    ) -> DisclosureError<VpClientError>
    where
        F: Fn() -> Option<VpMessageClientError>,
        K: CredentialEcdsaKey + Eq + Hash,
        KF: KeyFactory<Key = K> + PoaFactory<Key = K>,
    {
        // Disclosing the session should result in the payload being sent while returning an error.
        let error = match proposal_session {
            DisclosureSession::Proposal(proposal) => proposal
                .disclose(key_factory)
                .await
                .expect_err("disclosing should have resulted in an error"),
            _ => unreachable!(),
        };

        if expect_report_error {
            let wallet_messages = wallet_messages.lock();
            assert_eq!(wallet_messages.len(), 1);
            _ = wallet_messages.first().unwrap().disclosure();
        } else {
            assert_eq!(wallet_messages.lock().len(), 0);
        }

        error
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_device_response() {
        /// A mock key factory that just returns errors.
        struct MockKeyFactory;
        impl KeyFactory for MockKeyFactory {
            type Key = MockRemoteEcdsaKey;
            type Error = MockRemoteKeyFactoryError;

            fn generate_existing<I: Into<String>>(&self, identifier: I, _: VerifyingKey) -> Self::Key {
                // Normally this method is expected to return a key whose public key equals the specified
                // `VerifyingKey`, but for the purposes of this test, it doesn't matter that we don't do so here.
                MockRemoteEcdsaKey::new(identifier.into(), SigningKey::random(&mut OsRng))
            }

            async fn sign_multiple_with_existing_keys(
                &self,
                _: Vec<(Vec<u8>, Vec<&Self::Key>)>,
            ) -> Result<Vec<Vec<Signature>>, Self::Error> {
                Err(MockRemoteKeyFactoryError::Signing)
            }

            async fn sign_with_new_keys(&self, _: Vec<u8>, _: u64) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
                unimplemented!()
            }

            async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
                let keys =
                    iter::repeat_with(|| MockRemoteEcdsaKey::new(random_string(32), SigningKey::random(&mut OsRng)))
                        .take(count as usize)
                        .collect::<Vec<_>>();
                Ok(keys)
            }
        }

        impl PoaFactory for MockKeyFactory {
            type Key = MockRemoteEcdsaKey;
            type Error = MockRemoteKeyFactoryError;

            async fn poa(
                &self,
                _: VecAtLeastTwoUnique<&Self::Key>,
                _: String,
                _: Option<String>,
            ) -> Result<Poa, Self::Error> {
                unimplemented!()
            }
        }

        let key_factory = MockKeyFactory;
        let device_key = key_factory.generate_new().await.unwrap();

        // Attempting to create a disclosure with a malfunctioning key store should result in an error.
        let (proposal_session, wallet_messages) = create_disclosure_session_proposal(|| None, &device_key).await;
        assert_matches!(
            try_disclose(proposal_session, wallet_messages, &key_factory, false).await,
            DisclosureError {
                data_shared,
                error: VpClientError::DeviceResponse(_)
            } if !data_shared
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_invalid_encryption_key() {
        let key_factory = MockRemoteKeyFactory::default();
        let device_key = key_factory.generate_new().await.unwrap();

        // Attempting to encrypt a disclosure to a malformed encryption key should result in an error.
        let (mut proposal_session, wallet_messages) = create_disclosure_session_proposal(|| None, &device_key).await;

        let DisclosureSession::Proposal(ref mut proposal) = proposal_session else {
            panic!("disclosure session should have been a proposal")
        };
        proposal
            .data
            .auth_request
            .encryption_pubkey
            .set_parameter("kty", Some(json!("invalid_value"))) // kty (key type) is normally EC
            .unwrap();

        assert_matches!(
            try_disclose(proposal_session, wallet_messages, &key_factory, false).await,
            DisclosureError {
                data_shared,
                error: VpClientError::AuthResponseEncryption(_)
            } if !data_shared
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_http_client_request() {
        let key_factory = MockRemoteKeyFactory::default();
        let device_key = key_factory.generate_new().await.unwrap();

        // Create a `DisclosureSession` containing a proposal
        // and a `VpMessageClient` that will return a `reqwest::Error`.
        let (proposal_session, wallet_messages) = create_disclosure_session_proposal(
            || {
                let response = http::Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body("")
                    .unwrap();
                let reqwest_error = reqwest::Response::from(response).error_for_status().unwrap_err();

                Some(VpMessageClientError::Http(reqwest_error))
            },
            &device_key,
        )
        .await;

        assert_matches!(
            try_disclose(proposal_session, wallet_messages, &key_factory, true).await,
            DisclosureError {
                data_shared,
                error: VpClientError::Request(VpMessageClientError::Http(_))
            } if data_shared
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_http_client_connection() {
        let key_factory = MockRemoteKeyFactory::default();
        let device_key = key_factory.generate_new().await.unwrap();

        // Create a `DisclosureSession` containing a proposal
        // and a `VpMessageClient` that will return an error.
        let (proposal_session, wallet_messages) = create_disclosure_session_proposal(
            || {
                Some(VpMessageClientError::Http(futures::executor::block_on(async {
                    // This seems to be the only way to create a reqwest::Error whose is_connect() method returns true.
                    reqwest::get("http://nonexisting-domain-name.invalid")
                        .await
                        .unwrap_err()
                })))
            },
            &device_key,
        )
        .await;

        // No data should have been shared in this case
        assert_matches!(
            try_disclose(proposal_session, wallet_messages, &key_factory, true).await,
            DisclosureError {
                data_shared,
                error: VpClientError::Request(VpMessageClientError::Http(_))
            } if !data_shared
        );
    }
}
