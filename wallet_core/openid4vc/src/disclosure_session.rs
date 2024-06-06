use std::collections::HashMap;

use derive_more::From;
use futures::TryFutureExt;
use itertools::Itertools;
use reqwest::Response;
use tracing::{info, warn};

use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt, utils::random_string};

use nl_wallet_mdoc::{
    disclosure::DeviceResponse,
    engagement::SessionTranscript,
    holder::{
        DisclosureRequestMatch, DisclosureUriSource, MdocDataSource, ProposedAttributes, ProposedDocument, TrustAnchor,
    },
    identifiers::AttributeIdentifier,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::{ReaderRegistration, ValidationError},
        x509::{Certificate, CertificateError, CertificateType},
    },
    verifier::SessionType,
};

use crate::{
    authorization::AuthorizationErrorCode,
    openid4vp::{
        AuthRequestError, AuthResponseError, IsoVpAuthorizationRequest, VpAuthorizationErrorCode,
        VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject, VpResponse,
    },
    verifier::VerifierUrlParameters,
    ErrorResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum VpClientError {
    #[error("error sending OpenID4VP message: {0}")]
    Request(#[from] VpMessageClientError),
    #[error("error creating mdoc device response: {0}")]
    DeviceResponse(#[source] nl_wallet_mdoc::Error),
    #[error("error verifying Authorization Request: {0}")]
    AuthorizationRequest(#[from] AuthRequestError),
    #[error("incorrect client_id: expected {expected}, found {found}")]
    IncorrectClientId { expected: String, found: String },
    #[error("no reader registration in RP certificate")]
    MissingReaderRegistration,
    #[error("error validating requested attributes: {0}")]
    RequestedAttributesValidation(#[from] ValidationError),
    #[error("error matching requested attributes against mdocs: {0}")]
    MatchRequestedAttributes(#[source] nl_wallet_mdoc::Error),
    #[error("error parsing RP certificate: {0}")]
    RpCertificate(#[from] CertificateError),
    #[error("multiple candidates for disclosure is unsupported, found for doc types: {}", .0.join(", "))]
    MultipleCandidates(Vec<String>),
    #[error("error encrypting Authorization Response: {0}")]
    AuthResponseEncryption(#[from] AuthResponseError),
    #[error("error deserializing request_uri object: {0}")]
    RequestUri(#[source] serde_urlencoded::de::Error),
    #[error("missing session_type query parameter in request URI")]
    MissingSessionType,
    #[error("malformed session_type query parameter in request URI: {0}")]
    MalformedSessionType(#[source] serde_urlencoded::de::Error),
    #[error("mismatch between session type and reader engagement source: {0} not allowed from {1}")]
    ReaderEnagementSourceMismatch(SessionType, DisclosureUriSource),
}

#[derive(Debug, thiserror::Error)]
pub enum VpMessageClientError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("error response: {0:?}")]
    ErrorResponse(ErrorResponse<String>),
}

/// Contract for sending OpenID4VP protocol messages.
pub trait VpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError>;

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError>;

    async fn send_error(
        &self,
        url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError>;

    async fn terminate(&self, url: BaseUrl) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.send_error(
            url,
            ErrorResponse {
                error: VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::AccessDenied),
                error_description: None,
                error_uri: None,
            },
        )
        .await
    }
}

#[derive(From)]
pub struct HttpVpMessageClient {
    http_client: reqwest::Client,
}

impl VpMessageClient for HttpVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        self.http_client
            .get(url.into_inner())
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<String>>().await?;
                    Err(VpMessageClientError::ErrorResponse(error))
                } else {
                    Ok(response.json().await?)
                }
            })
            .await
    }

    async fn send_authorization_response(
        &self,
        url: BaseUrl,
        jwe: String,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        self.http_client
            .post(url.into_inner())
            .form(&HashMap::from([("response", jwe)]))
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<String>>().await?;
                    Err(VpMessageClientError::ErrorResponse(error))
                } else {
                    deserialize_vp_response(response).await
                }
            })
            .await
    }

    async fn send_error(
        &self,
        url: BaseUrl,
        error: ErrorResponse<VpAuthorizationErrorCode>,
    ) -> Result<Option<BaseUrl>, VpMessageClientError> {
        let response = self
            .http_client
            .post(url.into_inner())
            .json(&error)
            .send()
            .await?
            .error_for_status()?;

        let response = deserialize_vp_response(response).await?;
        Ok(response)
    }
}

/// If the RP does not wish to specify a redirect URI, e.g. in case of cross device flows, then the spec does not say
/// whether the RP should send an empty JSON object, i.e. `{}`, or no body at all. So this function accepts both.
async fn deserialize_vp_response(response: Response) -> Result<Option<BaseUrl>, VpMessageClientError> {
    let response_bytes = response.bytes().await?;
    if response_bytes.is_empty() {
        return Ok(None);
    }
    let response: VpResponse = serde_json::from_slice(&response_bytes)?;
    Ok(response.redirect_uri)
}

#[allow(clippy::large_enum_variant)]
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
    certificate: Certificate,
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
    pub async fn start<'a, S>(
        client: H,
        request_uri: &str,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, VpClientError>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        info!("start disclosure session");

        let request_uri_object: VpRequestUriObject =
            serde_urlencoded::from_str(request_uri).map_err(VpClientError::RequestUri)?;

        // Parse the `SessionType` from the verifier URL.
        let VerifierUrlParameters { session_type, .. } = serde_urlencoded::from_str(
            request_uri_object
                .request_uri
                .as_ref()
                .query()
                .ok_or(VpClientError::MissingSessionType)?,
        )
        .map_err(VpClientError::MalformedSessionType)?;

        // Check the `SessionType` that was contained in the verifier URL against the source of the reader engagement.
        // A same-device session is expected to come from a Universal Link,
        // while a cross-device session should come from a scanned QR code.
        if uri_source.session_type() != session_type {
            return Err(VpClientError::ReaderEnagementSourceMismatch(session_type, uri_source));
        }

        let jws = client
            .get_authorization_request(request_uri_object.request_uri.clone())
            .await?;

        let (auth_request, certificate) = VpAuthorizationRequest::verify(&jws, trust_anchors)?;

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

    async fn report_error_back<T>(error: VpClientError, client: &H, url: BaseUrl) -> Result<T, VpClientError> {
        let error_code = match error {
            VpClientError::IncorrectClientId { .. }
            | VpClientError::MissingReaderRegistration
            | VpClientError::RequestedAttributesValidation(_)
            | VpClientError::RpCertificate(_) => {
                VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest)
            }
            _ => return Err(error), // don't report other errors
        };

        let error_response = ErrorResponse {
            error: error_code,
            error_description: Some(error.to_string()),
            error_uri: None,
        };

        let _ = client.send_error(url, error_response).await; // ignore errors in sending this error
        Err(error)
    }

    /// Internal helper function for processing and checking the Authorization Request,
    /// including checking whether or not we have the requested attributes.
    async fn process_request<'a, S>(
        auth_request: &IsoVpAuthorizationRequest,
        certificate: &Certificate,
        session_transcript: &SessionTranscript,
        request_uri_object: &VpRequestUriObject,
        mdoc_data_source: &S,
    ) -> Result<(VerifierSessionDataCheckResult<I>, ReaderRegistration), VpClientError>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        // The `client_id` in the Authorization Request, which has been authenticated, has to equal
        // the `client_id` that the RP sent in the Request URI object at the start of the session.
        if auth_request.client_id != request_uri_object.client_id {
            return Err(VpClientError::IncorrectClientId {
                expected: request_uri_object.client_id.clone(),
                found: auth_request.client_id.clone(),
            });
        }

        // Extract `ReaderRegistration` from the certificate.
        let reader_registration = match CertificateType::from_certificate(certificate)? {
            CertificateType::ReaderAuth(Some(reader_registration)) => *reader_registration,
            _ => return Err(VpClientError::MissingReaderRegistration),
        };

        // Verify that the requested attributes are included in the reader authentication.
        reader_registration.verify_requested_attributes(auth_request.items_requests.as_ref().iter())?;

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

            return Err(VpClientError::MultipleCandidates(duplicate_doc_types));
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

    pub fn reader_registration(&self) -> &ReaderRegistration {
        &self.data().reader_registration
    }

    pub fn verifier_certificate(&self) -> &Certificate {
        &self.data().certificate
    }

    pub async fn terminate(self) -> Result<(), VpClientError> {
        let data = self.data();
        data.client.terminate(data.auth_request.response_uri.clone()).await?;
        Ok(())
    }

    pub fn session_type(&self) -> SessionType {
        self.data().session_type
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

    pub async fn disclose<KF, K>(&self, key_factory: &KF) -> Result<Option<BaseUrl>, DisclosureError>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        info!("disclose proposed documents");

        // Clone the proposed documents and construct a `DeviceResponse` by
        // signing these, then encrypt the response to the RP's public key.
        let proposed_documents = self.proposed_documents.clone();

        info!("sign proposed documents");

        let device_response = DeviceResponse::from_proposed_documents(proposed_documents, key_factory)
            .await
            .map_err(|err| DisclosureError::before_sharing(VpClientError::DeviceResponse(err)))?;

        info!("serialize and encrypt Authorization Response");

        let jwe = VpAuthorizationResponse::new_encrypted(device_response, &self.data.auth_request, &self.mdoc_nonce)
            .map_err(|err| DisclosureError::before_sharing(VpClientError::AuthResponseEncryption(err)))?;

        info!("send Authorization Response to verifier");

        let redirect_uri = self
            .data
            .client
            .send_authorization_response(self.data.auth_request.response_uri.clone(), jwe)
            .await
            .map_err(|err| {
                warn!("sending Authorization Response failed: {err:?}");
                DisclosureError::from(err)
            })?;

        info!("sending Authorization Response succeeded");
        Ok(redirect_uri)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("could not perform disclosure, attributes were shared: {data_shared}, error: {error}")]
pub struct DisclosureError {
    pub data_shared: bool,
    #[source]
    pub error: VpClientError,
}

impl DisclosureError {
    pub fn new(data_shared: bool, error: VpClientError) -> Self {
        Self { data_shared, error }
    }

    pub fn before_sharing(error: VpClientError) -> Self {
        Self {
            data_shared: false,
            error,
        }
    }
}

impl From<VpMessageClientError> for DisclosureError {
    fn from(source: VpMessageClientError) -> Self {
        let data_shared = match source {
            VpMessageClientError::Http(ref reqwest_error) => !reqwest_error.is_connect(),
            VpMessageClientError::ErrorResponse(_) | VpMessageClientError::Json(_) => true,
        };
        Self::new(data_shared, VpClientError::Request(source))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};

    use crate::{
        disclosure_session::DisclosureSession,
        jwt,
        mock::MockMdocDataSource,
        openid4vp::{VpAuthorizationErrorCode, VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject},
        verifier::VerifierUrlParameters,
        ErrorResponse,
    };
    use nl_wallet_mdoc::{
        examples::{Examples, IsoCertTimeGenerator},
        holder::DisclosureUriSource,
        server_keys::KeyPair,
        software_key_factory::SoftwareKeyFactory,
        unsigned::Entry,
        utils::reader_auth::ReaderRegistration,
        verifier::SessionType,
    };
    use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt};

    use super::{VpMessageClient, VpMessageClientError};

    // A mock implementation of the `VpMessageClient` trait that implements the RP side of OpenID4VP
    // directly in its methods.
    struct MockVpMessageClient {
        nonce: String,
        encryption_keypair: EcKeyPair,
        auth_keypair: KeyPair,
        auth_request: VpAuthorizationRequest,
        request_uri: BaseUrl,
        response_uri: BaseUrl,
    }

    impl MockVpMessageClient {
        fn new(auth_keypair: KeyPair) -> Self {
            let query = serde_urlencoded::to_string(VerifierUrlParameters {
                session_type: SessionType::SameDevice,
                ephemeral_id: vec![42],
                time: Utc::now(),
            })
            .unwrap();
            let request_uri = ("https://example.com/request_uri?".to_string() + &query)
                .parse()
                .unwrap();

            let nonce = "nonce".to_string();
            let response_uri: BaseUrl = "https://example.com/response_uri".parse().unwrap();
            let encryption_keypair = EcKeyPair::generate(EcCurve::P256).unwrap();

            let auth_request = VpAuthorizationRequest::new(
                &Examples::items_requests(),
                auth_keypair.certificate(),
                nonce.clone(),
                encryption_keypair.to_jwk_public_key().try_into().unwrap(),
                response_uri.clone(),
            )
            .unwrap();

            Self {
                nonce,
                encryption_keypair,
                auth_keypair,
                auth_request,
                request_uri,
                response_uri,
            }
        }

        fn start_session(&self) -> String {
            serde_urlencoded::to_string(VpRequestUriObject {
                request_uri: self.request_uri.clone(),
                client_id: self.auth_keypair.certificate().san_dns_name().unwrap().unwrap(),
            })
            .unwrap()
        }
    }

    impl VpMessageClient for MockVpMessageClient {
        async fn get_authorization_request(
            &self,
            url: BaseUrl,
        ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
            assert_eq!(url, self.request_uri);

            let jws = jwt::sign_with_certificate(&self.auth_request, &self.auth_keypair)
                .await
                .unwrap();
            Ok(jws)
        }

        async fn send_authorization_response(
            &self,
            url: BaseUrl,
            jwe: String,
        ) -> Result<Option<BaseUrl>, VpMessageClientError> {
            assert_eq!(url, self.response_uri);

            let (auth_response, mdoc_nonce) =
                VpAuthorizationResponse::decrypt(&jwe, &self.encryption_keypair, &self.nonce).unwrap();
            let disclosed_attrs = auth_response
                .verify(
                    &self.auth_request.clone().try_into().unwrap(),
                    &mdoc_nonce,
                    &IsoCertTimeGenerator,
                    Examples::iaca_trust_anchors(),
                )
                .unwrap();

            assert_eq!(
                *disclosed_attrs["org.iso.18013.5.1.mDL"].attributes["org.iso.18013.5.1"]
                    .first()
                    .unwrap(),
                Entry {
                    name: "family_name".to_string(),
                    value: "Doe".into()
                }
            );

            Ok(None)
        }

        async fn send_error(
            &self,
            _url: BaseUrl,
            error: ErrorResponse<VpAuthorizationErrorCode>,
        ) -> Result<Option<BaseUrl>, VpMessageClientError> {
            panic!("error: {:?}", error)
        }
    }

    #[tokio::test]
    async fn disclosure() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let trust_anchors = &[ca.certificate().try_into().unwrap()];
        let rp_keypair = ca
            .generate_reader_mock(Some(ReaderRegistration::new_mock_from_requests(
                &Examples::items_requests(),
            )))
            .unwrap();

        // Initialize the "wallet"
        let mdocs = MockMdocDataSource::default();
        let key_factory = &SoftwareKeyFactory::default();

        // Start a session at the "RP"
        let message_client = MockVpMessageClient::new(rp_keypair);
        let request_uri = message_client.start_session();

        // Perform the first part of the session, resulting in the proposed disclosure.
        let session = DisclosureSession::start(
            message_client,
            &request_uri,
            DisclosureUriSource::Link,
            &mdocs,
            trust_anchors,
        )
        .await
        .unwrap();

        let DisclosureSession::Proposal(proposal) = session else {
            panic!("should have requested attributes")
        };

        // Finish the disclosure.
        proposal.disclose(key_factory).await.unwrap();
    }
}
