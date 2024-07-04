use derive_more::From;
use futures::TryFutureExt;
use itertools::Itertools;
use mime::Mime;
use once_cell::sync::Lazy;
use reqwest::{header::ACCEPT, Method, Response};
use tracing::{info, warn};

use wallet_common::{config::wallet_config::BaseUrl, jwt::Jwt, utils::random_string};

use nl_wallet_mdoc::{
    disclosure::DeviceResponse,
    engagement::SessionTranscript,
    holder::{
        DisclosureError, DisclosureRequestMatch, DisclosureUriSource, MdocDataSource, ProposedAttributes,
        ProposedDocument, TrustAnchor,
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
    openid4vp::{
        AuthRequestValidationError, AuthResponseError, IsoVpAuthorizationRequest, RequestUriMethod,
        VpAuthorizationRequest, VpAuthorizationResponse, VpRequestUriObject, VpResponse, WalletRequest,
    },
    verifier::{VerifierUrlParameters, VpToken},
    AuthorizationErrorCode, ErrorResponse, VpAuthorizationErrorCode,
};

#[derive(Debug, thiserror::Error)]
pub enum VpClientError {
    #[error("error sending OpenID4VP message: {0}")]
    Request(#[from] VpMessageClientError),
    #[error("error creating mdoc device response: {0}")]
    DeviceResponse(#[source] nl_wallet_mdoc::Error),
    #[error("error verifying Authorization Request: {0}")]
    AuthRequestValidation(#[from] AuthRequestValidationError),
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
    #[error("mismatch between session type and disclosure URI source: {0} not allowed from {1}")]
    DisclosureUriSourceMismatch(SessionType, DisclosureUriSource),
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
        wallet_nonce: Option<String>,
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

pub static APPLICATION_OAUTH_AUTHZ_REQ_JWT: Lazy<Mime> = Lazy::new(|| {
    "application/oauth-authz-req+jwt"
        .parse()
        .expect("could not parse MIME type")
});

#[derive(From)]
pub struct HttpVpMessageClient {
    http_client: reqwest::Client,
}

impl VpMessageClient for HttpVpMessageClient {
    async fn get_authorization_request(
        &self,
        url: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Jwt<VpAuthorizationRequest>, VpMessageClientError> {
        let method = match wallet_nonce {
            Some(_) => Method::POST,
            None => Method::GET,
        };

        let mut request = self
            .http_client
            .request(method, url.into_inner())
            .header(ACCEPT, APPLICATION_OAUTH_AUTHZ_REQ_JWT.as_ref());

        if wallet_nonce.is_some() {
            request = request.form(&WalletRequest { wallet_nonce });
        }

        request
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<String>>().await?;
                    Err(VpMessageClientError::ErrorResponse(error))
                } else {
                    Ok(response.text().await?.into())
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
            .form(&VpToken { vp_token: jwe })
            .send()
            .map_err(VpMessageClientError::from)
            .and_then(|response| async {
                // If the HTTP response code is 4xx or 5xx, parse the JSON as an error
                let status = response.status();
                if status.is_client_error() || status.is_server_error() {
                    let error = response.json::<ErrorResponse<String>>().await?;
                    Err(VpMessageClientError::ErrorResponse(error))
                } else {
                    Self::deserialize_vp_response(response).await
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
            .form(&error)
            .send()
            .await?
            .error_for_status()?;

        let response = Self::deserialize_vp_response(response).await?;
        Ok(response)
    }
}

impl HttpVpMessageClient {
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
}

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
        request_uri_query: &str,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, VpClientError>
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
                .ok_or(VpClientError::MissingSessionType)?,
        )
        .map_err(VpClientError::MalformedSessionType)?;

        // Check the `SessionType` that was contained in the verifier URL against the source of the URI.
        // A same-device session is expected to come from a Universal Link,
        // while a cross-device session should come from a scanned QR code.
        if uri_source.session_type() != session_type {
            return Err(VpClientError::DisclosureUriSourceMismatch(session_type, uri_source));
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

        let (auth_request, certificate) = VpAuthorizationRequest::verify(&jws, trust_anchors, request_nonce)?;

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

        // If sending the error results in an error, log it but do nothing else.
        let _ = client
            .send_error(url, error_response)
            .await
            .inspect_err(|err| warn!("failed to send error to server: {err}"));

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

    pub async fn terminate(self) -> Result<Option<BaseUrl>, VpClientError> {
        let data = match self {
            DisclosureSession::MissingAttributes(session) => session.data,
            DisclosureSession::Proposal(session) => session.data,
        };

        let return_url = data.client.terminate(data.auth_request.response_uri).await?;

        Ok(return_url)
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

    pub async fn disclose<KF, K>(&self, key_factory: &KF) -> Result<Option<BaseUrl>, DisclosureError<VpClientError>>
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
            .inspect_err(|err| {
                warn!("sending Authorization Response failed: {err}");
            })?;

        info!("sending Authorization Response succeeded");
        Ok(redirect_uri)
    }
}

impl From<VpMessageClientError> for DisclosureError<VpClientError> {
    fn from(source: VpMessageClientError) -> Self {
        let data_shared = match source {
            VpMessageClientError::Http(ref reqwest_error) => !reqwest_error.is_connect(),
            VpMessageClientError::ErrorResponse(_) | VpMessageClientError::Json(_) => true,
        };
        Self::new(data_shared, VpClientError::Request(source))
    }
}
