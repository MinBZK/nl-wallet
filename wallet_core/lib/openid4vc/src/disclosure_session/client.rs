use derive_more::Constructor;
use reqwest::ClientBuilder;
use tracing::info;
use tracing::warn;

use attestation_data::auth::reader_auth::ReaderRegistration;
use attestation_data::x509::CertificateType;
use attestation_types::request::NormalizedCredentialRequests;
use crypto::utils as crypto_utils;
use crypto::x509::BorrowingCertificate;
use http_utils::urls::BaseUrl;

use crate::errors::AuthorizationErrorCode;
use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::RequestUriMethod;
use crate::openid4vp::VpAuthorizationRequest;
use crate::openid4vp::VpRequestUriObject;
use crate::verifier::VerifierUrlParameters;

use super::error::VpClientError;
use super::error::VpSessionError;
use super::error::VpVerifierError;
use super::message_client::HttpVpMessageClient;
use super::message_client::VpMessageClient;
use super::message_client::VpMessageClientError;
use super::session::VpDisclosureSession;
use super::uri_source::DisclosureUriSource;
use super::AttestationAttributePaths;
use super::DisclosureClient;
use super::VerifierCertificate;

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
            VpVerifierError::AuthRequestValidation(_)
            | VpVerifierError::IncorrectClientId { .. }
            | VpVerifierError::MissingReaderRegistration
            | VpVerifierError::Request(VpMessageClientError::Json(_))
            | VpVerifierError::RequestedAttributesValidation(_)
            | VpVerifierError::RpCertificate(_) => {
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
        credential_requests: NormalizedCredentialRequests,
        certificate: &BorrowingCertificate,
    ) -> Result<(AttestationAttributePaths, ReaderRegistration), VpVerifierError> {
        // The `client_id` in the Authorization Request, which has been authenticated, has to equal
        // the `client_id` that the RP sent in the Request URI object at the start of the session.
        if auth_request_client_id != request_uri_client_id {
            return Err(VpVerifierError::IncorrectClientId {
                expected: request_uri_client_id.to_string(),
                found: auth_request_client_id.to_string(),
            })?;
        }

        // Extract `ReaderRegistration` from the certificate.
        let reader_registration =
            match CertificateType::from_certificate(certificate).map_err(VpVerifierError::RpCertificate)? {
                CertificateType::ReaderAuth(Some(reader_registration)) => *reader_registration,
                _ => return Err(VpVerifierError::MissingReaderRegistration)?,
            };

        // Verify that the requested attributes are included in the reader authentication.
        reader_registration
            .verify_requested_attributes(&credential_requests.as_ref().as_slice())
            .map_err(VpVerifierError::RequestedAttributesValidation)?;

        // Convert the request into a generic representation.
        let requested_attribute_paths = credential_requests
            .try_into_attribute_paths()
            .map_err(VpVerifierError::EmptyRequest)?;

        Ok((requested_attribute_paths, reader_registration))
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
            auth_request.credential_requests.clone(),
            &certificate,
        );
        let (requested_attribute_paths, registration) = match process_request_result {
            Ok(value) => value,
            Err(error) => return Err(self.report_error_back(auth_request.response_uri, error).await)?,
        };

        let session = VpDisclosureSession::new(
            self.client.clone(),
            session_type,
            requested_attribute_paths,
            VerifierCertificate::new(certificate, registration),
            auth_request,
        );

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Implement tests for VpDisclosureClient().
}
