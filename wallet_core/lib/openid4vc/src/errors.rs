use std::error::Error;
use std::fmt::Debug;

use http::StatusCode;
use http_utils::error::HttpJsonError;
use http_utils::error::HttpJsonErrorType;
use jwt::wia::WiaError;
use oauth::errors::BodyOrRedirectErrorResponse;
use oauth::errors::DisclosureErrorResponse;
use oauth::errors::ErrorResponse;
use oauth::errors::ErrorStatusCode;
use oauth::errors::ErrorWithCode;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use strum::EnumString;

use crate::authorization_code_flow::InvalidAuthorizationRequest;
use crate::authorizing_issuer::AuthorizationRequestError;
use crate::authorizing_issuer::AuthorizeError;
use crate::authorizing_issuer::CompleteAuthorizationError;
use crate::authorizing_issuer::ParError;
use crate::issuer::CredentialPreviewError;
use crate::issuer::CredentialRequestError;
use crate::issuer::IssuanceError;
use crate::issuer::TokenRequestError;
use crate::issuer::WiaVerificationError;
use crate::verifier::CancelSessionError;
use crate::verifier::DisclosedAttributesError;
use crate::verifier::GetAuthRequestError;
use crate::verifier::NewSessionError;
use crate::verifier::PostAuthResponseError;
use crate::verifier::SessionError;
use crate::verifier::SessionStatus;
use crate::verifier::SessionStatusError;
use crate::verifier::WithRedirectUri;

impl<E, T> From<WithRedirectUri<E>> for DisclosureErrorResponse<T>
where
    E: Into<ErrorResponse<T>> + Error,
{
    fn from(value: WithRedirectUri<E>) -> Self {
        DisclosureErrorResponse {
            error_response: value.error.into(),
            redirect_uri: value.redirect_uri,
        }
    }
}

// OpenID4VCI Error Codes

/// The list of error codes that can result from an Authorization Request. Note that this is also used by OpenID4VP.
///
/// See: <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorizationErrorCode {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

impl From<AuthorizeError> for BodyOrRedirectErrorResponse<AuthorizationErrorCode> {
    fn from(value: AuthorizeError) -> Self {
        let status_code = match value {
            // The errors at the Authorization Endpoint that can occur before the PAR is retrieved and the
            // `redirect_uri` is known are represented as HTTP status code and plain-text bodies.
            AuthorizeError::UnknownClient(_) => StatusCode::UNAUTHORIZED,
            AuthorizeError::ParStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthorizeError::UnknownRequestUri(_) => StatusCode::NOT_FOUND,
            AuthorizeError::MismatchedClient { .. } => StatusCode::UNAUTHORIZED,

            // Once the `redirect_uri` is known, convert the error to a 303 redirect instead.
            AuthorizeError::AuthorizationRequest(redirect_error) => return Self::new_redirect(redirect_error.into()),
        };

        Self::new_body(status_code, value.to_string())
    }
}

impl ErrorWithCode for AuthorizationRequestError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::InvalidAuthorizationRequest(InvalidAuthorizationRequest::UnsupportedCodeChallenge) => {
                AuthorizationErrorCode::InvalidRequest
            }

            Self::InvalidAuthorizationRequest(InvalidAuthorizationRequest::NoValidScope(_)) => {
                AuthorizationErrorCode::InvalidScope
            }

            Self::AuthorizationCodeFlow(error) => error.error_code(),

            Self::CompleteAuthorization(error) => error.error_code(),
        }
    }
}

impl ErrorWithCode for CompleteAuthorizationError {
    type ErrorCode = AuthorizationErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuableDocument(_) | Self::SessionStore(_) => AuthorizationErrorCode::ServerError,
        }
    }
}

/// The list of error codes that can result from the PAR POST request.
///
/// According to <https://datatracker.ietf.org/doc/html/rfc9126#section-2.3>, these can be taken either from
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-5.2> or
/// <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1>, i.e. the token endpoint error codes or the
/// authorization endpoint error codes.
///
/// This type represents a selection among these error codes, containing only those that the issuer returns.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ParErrorCode {
    // Token error code.
    InvalidClient,
    // Both token and authorization error code.
    InvalidRequest,
    // Authorization error code.
    ServerError,

    /// Invalid Client Attestation / WIA.
    /// See <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-7.4-2.2.1>
    InvalidClientAttestation,

    /// Client Attestation / WIA is valid but not fresh enough.
    /// See <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-7.4-2.3.1>
    UseFreshAttestation,
}

impl ErrorStatusCode for ParErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient => StatusCode::UNAUTHORIZED,

            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::InvalidClientAttestation | Self::UseFreshAttestation => StatusCode::UNAUTHORIZED,
        }
    }
}

impl ErrorWithCode for ParError {
    type ErrorCode = ParErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::UnknownClient(_) => ParErrorCode::InvalidClient,

            Self::Wia(WiaVerificationError::WiaVerification(WiaError::Expired)) => ParErrorCode::UseFreshAttestation,

            Self::Wia(WiaVerificationError::ChallengeStore(_)) => ParErrorCode::ServerError,

            Self::Wia(WiaVerificationError::InvalidChallenge)
            | Self::Wia(WiaVerificationError::MissingChallenge)
            | Self::Wia(WiaVerificationError::WiaVerification(_)) => ParErrorCode::InvalidClientAttestation,

            Self::AuthorizationDetailsUnsupported | Self::InvalidRedirectUri(_) => ParErrorCode::InvalidRequest,

            Self::Store(_) => ParErrorCode::ServerError,
        }
    }
}

/// See <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-6.3>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.2>.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TokenErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,

    /// Invalid Client Attestation / WIA.
    /// See <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-7.4-2.2.1>
    InvalidClientAttestation,

    /// Client Attestation / WIA is valid but not fresh enough.
    /// See <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-attestation-based-client-auth-09#section-7.4-2.3.1>
    UseFreshAttestation,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the specs, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the specs, so we have freedom to return what we want.
    ServerError,
}

impl ErrorStatusCode for TokenErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidClient => StatusCode::UNAUTHORIZED,

            Self::InvalidGrant | Self::UnauthorizedClient | Self::UnsupportedGrantType | Self::InvalidScope => {
                StatusCode::BAD_REQUEST
            }

            Self::InvalidClientAttestation | Self::UseFreshAttestation => StatusCode::UNAUTHORIZED,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for TokenRequestError {
    type ErrorCode = TokenErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuanceError(IssuanceError::SessionStore(_)) => TokenErrorCode::ServerError,

            // A missing session (cleaned up) or a session in a terminal/wrong state (already used or
            // expired) both mean the authorization grant presented at `/token` is no longer valid.
            // Per RFC 6749 section 5.2 that is exactly `invalid_grant` ("invalid, expired, revoked ...").
            //
            // In the pre-authorized-code flow `invalid_grant` can *only* result from these two cases
            // (there are no PKCE / client_id / scope / redirect_uri checks that also yield it), so the
            // wallet can unambiguously map a pre-authorized `invalid_grant` onto a specific error without
            // relying on a non-standard error code.
            Self::SessionNotFound | Self::IssuanceError(IssuanceError::UnexpectedState) => TokenErrorCode::InvalidGrant,

            Self::IssuanceError(_) => TokenErrorCode::InvalidRequest,

            Self::UnexpectedGrantType { .. } => TokenErrorCode::UnsupportedGrantType,

            Self::Wia(WiaVerificationError::WiaVerification(WiaError::Expired)) => TokenErrorCode::UseFreshAttestation,

            Self::Wia(WiaVerificationError::ChallengeStore(_)) => TokenErrorCode::ServerError,

            Self::Wia(WiaVerificationError::InvalidChallenge)
            | Self::Wia(WiaVerificationError::MissingChallenge)
            | Self::Wia(WiaVerificationError::WiaVerification(_)) => TokenErrorCode::InvalidClientAttestation,

            Self::MissingCodeVerifier | Self::PkceVerificationFailed => TokenErrorCode::InvalidGrant,

            Self::UnknownClient(_) => TokenErrorCode::InvalidClient,

            Self::ClientIdMismatch { .. } => TokenErrorCode::InvalidGrant,

            Self::AuthorizationDetailsUnsupported => TokenErrorCode::InvalidRequest,

            Self::ScopeMismatch { .. } | Self::PreAuthorizedScopeUnsupported(_) => TokenErrorCode::InvalidScope,

            Self::MissingRedirectUri | Self::RedirectUriMismatch { .. } => TokenErrorCode::InvalidRequest,

            Self::CredentialConfigNotOffered(_) => TokenErrorCode::ServerError,
        }
    }
}

/// Error codes for the credential preview endpoint.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CredentialPreviewErrorCode {
    InvalidRequest,
    InvalidToken,
    ServerError,
}

impl ErrorStatusCode for CredentialPreviewErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for CredentialPreviewError {
    type ErrorCode = CredentialPreviewErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::IssuanceError(IssuanceError::SessionStore(_)) => CredentialPreviewErrorCode::ServerError,

            Self::IssuanceError(_) => CredentialPreviewErrorCode::InvalidRequest,

            Self::MalformedToken | CredentialPreviewError::Unauthorized => CredentialPreviewErrorCode::InvalidToken,

            Self::MissingCredentialConfiguration(_) => CredentialPreviewErrorCode::ServerError,
        }
    }
}

/// See <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-8.3.1>.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CredentialErrorCode {
    InvalidCredentialRequest,
    UnknownCredentialConfiguration,
    UnknownCredentialIdentifier,
    InvalidProof,
    InvalidNonce,
    InvalidEncryptionParameters,
    CredentialRequestDenied,

    // From https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1
    InvalidRequest,
    InvalidToken,
    InsufficientScope,

    /// This can be returned in case of internal server errors, i.e. with HTTP status code 5xx.
    /// This error type is not defined in the spec, but then again the entire HTTP response in case
    /// 5xx status codes is not defined by the spec, so we have freedom to return what we want.
    ServerError,
}

impl ErrorStatusCode for CredentialErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentialRequest
            | Self::UnknownCredentialConfiguration
            | Self::UnknownCredentialIdentifier
            | Self::InvalidProof
            | Self::InvalidNonce
            | Self::InvalidEncryptionParameters
            | Self::CredentialRequestDenied
            | Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::InsufficientScope => StatusCode::FORBIDDEN,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for CredentialRequestError {
    type ErrorCode = CredentialErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        // TODO (PVW-5541): Return `CredentialErrorCode::UnknownCredentialIdentifier` when appropriate.
        // TODO (PVW-5538): Return `CredentialErrorCode::InvalidEncryptionParameters` when appropriate.
        match self {
            // The session backing the access token is gone (cleaned up) or in a terminal/wrong state:
            // the token is no longer valid, which per RFC 6750 is `invalid_token`. This is hit when the
            // session expires during issuance (e.g. the holder dwells on the preview or PIN screen past the
            // session timeout), so the wallet can translate this to a "session expired" error instead of a
            // generic one.
            Self::IssuanceError(IssuanceError::UnexpectedState)
            | Self::IssuanceError(IssuanceError::UnknownSession(_)) => CredentialErrorCode::InvalidToken,

            Self::IssuanceError(IssuanceError::DpopInvalid(_)) => CredentialErrorCode::InvalidCredentialRequest,

            Self::IssuanceError(IssuanceError::SessionStore(_)) => CredentialErrorCode::ServerError,

            Self::Unauthorized | Self::MalformedToken => CredentialErrorCode::InvalidToken,

            Self::CredentialTypeNotOffered(_) => CredentialErrorCode::UnknownCredentialConfiguration,

            Self::UseBatchIssuance => CredentialErrorCode::InvalidCredentialRequest,

            Self::InvalidProofJwt(_) | Self::InvalidProofPublicKey(_) => CredentialErrorCode::InvalidProof,

            Self::MissingProofNonce => CredentialErrorCode::InvalidNonce,

            Self::ProofNonceStore(_) => CredentialErrorCode::ServerError,

            Self::InvalidNonce => CredentialErrorCode::InvalidNonce,

            Self::Jwt(_) => CredentialErrorCode::InvalidProof,

            Self::MissingCredentialConfiguration(_) => CredentialErrorCode::ServerError,

            Self::CredentialTypeMismatch { .. } | Self::WrongNumberOfCredentialRequests => {
                CredentialErrorCode::InvalidCredentialRequest
            }

            Self::MissingCredentialRequestPoP => CredentialErrorCode::InvalidProof,

            Self::JwkConversion(_) | Self::MdocConversion(_) | Self::SdJwtConversion(_) => {
                CredentialErrorCode::ServerError
            }

            Self::ObtainStatusClaim(_) | Self::IncorrectNumberOfStatusClaims(_) => CredentialErrorCode::ServerError,
        }
    }
}

// OpenID4VP Error Codes

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum GetAuthRequestErrorCode {
    InvalidRequest,
    ExpiredEphemeralId,
    ExpiredSession,
    CancelledSession,
    UnknownSession,

    ServerError,
}

impl ErrorStatusCode for GetAuthRequestErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            // Per RFC 9110 we MUST include a `WWW-Authenticate` HTTP header with this, but we can't do that
            // conveniently here. It seems this header is often skipped, and we use it internally here, we skip it too.
            Self::ExpiredEphemeralId => StatusCode::UNAUTHORIZED,

            Self::ExpiredSession | Self::CancelledSession | Self::UnknownSession => StatusCode::NOT_FOUND,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ErrorWithCode for GetAuthRequestError {
    type ErrorCode = GetAuthRequestErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::Session(SessionError::SessionStore(_)) => GetAuthRequestErrorCode::ServerError,

            Self::Session(SessionError::UnknownSession(_)) => GetAuthRequestErrorCode::UnknownSession,

            Self::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                GetAuthRequestErrorCode::CancelledSession
            }

            Self::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                GetAuthRequestErrorCode::ExpiredSession
            }

            Self::Session(SessionError::UnexpectedState(_)) => GetAuthRequestErrorCode::InvalidRequest,

            Self::InvalidEphemeralId(_) => GetAuthRequestErrorCode::InvalidRequest,

            Self::ExpiredEphemeralId(_) => GetAuthRequestErrorCode::ExpiredEphemeralId,

            Self::JwtSign(_) | Self::ReturnUrlConfigurationMismatch | Self::UnknownUseCase(_) => {
                GetAuthRequestErrorCode::ServerError
            }

            Self::QueryParametersMissing | Self::QueryParametersDeserialization(_) => {
                GetAuthRequestErrorCode::InvalidRequest
            }
        }
    }
}

/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0-20.html#name-error-response>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PostAuthResponseErrorCode {
    InvalidRequest,
    ExpiredSession,
    CancelledSession,
    UnknownSession,

    ServerError,

    /// An NL Wallet specific error code, meaning the following: in a disclosure based issuance session,
    /// the issuer found no attestations to issue.
    NoIssuableAttestations,
}

impl ErrorStatusCode for PostAuthResponseErrorCode {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::ExpiredSession | Self::CancelledSession | Self::UnknownSession => StatusCode::NOT_FOUND,

            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::NoIssuableAttestations => StatusCode::NOT_FOUND,
        }
    }
}

impl ErrorWithCode for PostAuthResponseError {
    type ErrorCode = PostAuthResponseErrorCode;

    fn error_code(&self) -> Self::ErrorCode {
        match self {
            Self::Session(SessionError::SessionStore(_)) => PostAuthResponseErrorCode::ServerError,

            Self::Session(SessionError::UnknownSession(_)) => PostAuthResponseErrorCode::UnknownSession,

            Self::Session(SessionError::UnexpectedState(SessionStatus::Cancelled)) => {
                PostAuthResponseErrorCode::CancelledSession
            }

            Self::Session(SessionError::UnexpectedState(SessionStatus::Expired)) => {
                PostAuthResponseErrorCode::ExpiredSession
            }

            Self::Session(SessionError::UnexpectedState(_)) | Self::AuthResponse(_) => {
                PostAuthResponseErrorCode::InvalidRequest
            }

            Self::HandlingDisclosureResult(err) => err.error_code(),

            Self::ResponseEncoding(_) => PostAuthResponseErrorCode::ServerError,
        }
    }
}

/// Error codes that the wallet sends to the verifier when it encounters an error or rejects the session.
/// See <https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1> and
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.5>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum VpAuthorizationErrorCode {
    // Error codes from RFC 6749.
    InvalidRequest,
    AccessDenied,
    UnsupportedResponseType,

    // Error codes from OpenID4VP.
    InvalidClient,
    VpFormatsNotSupported,
    InvalidRequestUriMethod,
    InvalidTransactionData,
    WalletUnavailable,
}

// The RP error types and `VerificationErrorCode` are handled differently from the errors above:
// instead of returning them as an `ErrorResponse`, they are returned as a `HttpJsonErrorBody`.
// This is because the endpoints that return these errors are not part of a protocol from the
// OAuth/OpenID family, which uses `ErrorResponse`, but instead they are specific to this implementation.

/// Error codes sent to the Relying Party when an error occurs when handling their request.
#[derive(Debug, Clone, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum VerificationErrorCode {
    ServerError,
    InvalidRequest,
    UnknownSession,
    Nonce,
    SessionState,
}

impl HttpJsonErrorType for VerificationErrorCode {
    fn title(&self) -> &'static str {
        match self {
            Self::ServerError => "Internal server error occurred",
            Self::InvalidRequest => "Invalid request",
            Self::UnknownSession => "Unknown session",
            Self::Nonce => "Redirect URI nonce incorrect or missing",
            Self::SessionState => "Session is not in the required state",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,

            Self::InvalidRequest => StatusCode::BAD_REQUEST,

            Self::UnknownSession => StatusCode::NOT_FOUND,

            // See the comment on `StatusCode::UNAUTHORIZED` for `GetAuthRequestErrorCode`.
            Self::Nonce => StatusCode::UNAUTHORIZED,

            Self::SessionState => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<&SessionError> for VerificationErrorCode {
    fn from(error: &SessionError) -> Self {
        match error {
            SessionError::SessionStore(_) => VerificationErrorCode::ServerError,
            SessionError::UnknownSession(_) => VerificationErrorCode::UnknownSession,
            SessionError::UnexpectedState(_) => VerificationErrorCode::SessionState,
        }
    }
}

impl From<&NewSessionError> for VerificationErrorCode {
    fn from(error: &NewSessionError) -> Self {
        match error {
            NewSessionError::Session(session_error) => session_error.into(),
            NewSessionError::NoCredentialRequests
            | NewSessionError::UnknownUseCase(_)
            | NewSessionError::UnsupportedDcqlFeatures(_)
            | NewSessionError::ReturnUrlConfigurationMismatch => VerificationErrorCode::InvalidRequest,
        }
    }
}

impl From<&SessionStatusError> for VerificationErrorCode {
    fn from(error: &SessionStatusError) -> Self {
        match error {
            SessionStatusError::Session(session_error) => session_error.into(),
            SessionStatusError::UrlEncoding(_) => VerificationErrorCode::ServerError,
        }
    }
}

impl From<&CancelSessionError> for VerificationErrorCode {
    fn from(error: &CancelSessionError) -> Self {
        match error {
            CancelSessionError::Session(session_error) => session_error.into(),
        }
    }
}

impl From<&DisclosedAttributesError> for VerificationErrorCode {
    fn from(error: &DisclosedAttributesError) -> Self {
        match error {
            DisclosedAttributesError::Session(session_error) => session_error.into(),
            DisclosedAttributesError::RedirectUriNonceMissing
            | DisclosedAttributesError::RedirectUriNonceMismatch(_) => Self::Nonce,
        }
    }
}

impl From<NewSessionError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: NewSessionError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

impl From<SessionStatusError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: SessionStatusError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

impl From<CancelSessionError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: CancelSessionError) -> Self {
        HttpJsonError::from_error(&error)
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DisclosedAttributesErrorData {
    pub session_status: Option<String>,
    pub session_error: Option<String>,
}

impl From<DisclosedAttributesError> for HttpJsonError<VerificationErrorCode> {
    fn from(error: DisclosedAttributesError) -> Self {
        let r#type = (&error).into();
        let detail = error.to_string();

        // The `session_status` field is only included if the session was in an unexpected state,
        // while the `session_error` field is further only included if that status is "FAILED".
        let data = match error {
            DisclosedAttributesError::Session(SessionError::UnexpectedState(session_status)) => {
                let status = Some(session_status.to_string());
                let error = match session_status {
                    SessionStatus::Failed { error } => Some(error),
                    _ => None,
                };

                DisclosedAttributesErrorData {
                    session_status: status,
                    session_error: error,
                }
            }
            _ => Default::default(),
        };

        // As `DisclosedAttributesErrorData` is a struct that only contains two simple strings,
        // we can assume that this will serialize to a `serde_json::Map` without fault.
        let Ok(serde_json::Value::Object(data)) = serde_json::to_value(data) else {
            panic!("serialized DisclosedAttributesErrorData should be an object");
        };

        Self::new(r#type, detail, data)
    }
}

#[cfg(test)]
mod tests {
    use oauth::errors::RemoteErrorCode;
    use oauth::errors::RemoteErrorResponse;
    use rstest::rstest;
    use serde_json::json;

    use super::CredentialErrorCode;
    use super::CredentialRequestError;
    use super::ErrorWithCode;
    use super::ParErrorCode;
    use super::TokenErrorCode;
    use crate::issuer::IssuanceError;
    use crate::issuer::TokenRequestError;

    #[rstest]
    #[case(json!({"error": "invalid_client"}), RemoteErrorCode::Known(ParErrorCode::InvalidClient))]
    #[case(json!({"error": "server_error"}), RemoteErrorCode::Known(ParErrorCode::ServerError))]
    #[case(json!({"error": "invalid_request"}), RemoteErrorCode::Known(ParErrorCode::InvalidRequest))]
    // A spec-compliant code the holder doesn't model explicitly falls through to the catch-all,
    // preserving the original wire value rather than failing to decode.
    #[case(
        json!({"error": "temporarily_unavailable"}),
        RemoteErrorCode::Unknown("temporarily_unavailable".to_string())
    )]
    fn test_par_error_code_deserializes_wire_format(
        #[case] body: serde_json::Value,
        #[case] expected: RemoteErrorCode<ParErrorCode>,
    ) {
        let response: RemoteErrorResponse<ParErrorCode> = serde_json::from_value(body).unwrap();
        assert_eq!(response.error, expected);
    }

    #[test]
    fn expired_or_used_session_maps_to_invalid_grant() {
        // A missing session (cleaned up) and a session in a terminal/wrong state (already used or
        // expired) both mean the authorization grant is no longer valid, which per RFC 6749 section
        // 5.2 is `invalid_grant`. In the pre-authorized-code flow this is the only source of
        // `invalid_grant`, which lets the wallet render the "QR code no longer valid" screen.
        assert_eq!(
            TokenRequestError::SessionNotFound.error_code(),
            TokenErrorCode::InvalidGrant
        );
        assert_eq!(
            TokenRequestError::IssuanceError(IssuanceError::UnexpectedState).error_code(),
            TokenErrorCode::InvalidGrant
        );
    }

    #[test]
    fn expired_session_at_credential_endpoint_maps_to_invalid_token() {
        // At the credential endpoint a missing session (cleaned up) or a session in a terminal/wrong
        // state means the access token's session is gone or expired, which per RFC 6750 is
        // `invalid_token` (401). This lets the wallet render the "session expired" screen when the
        // holder dwells past the session timeout mid-issuance.
        assert_eq!(
            CredentialRequestError::IssuanceError(IssuanceError::UnexpectedState).error_code(),
            CredentialErrorCode::InvalidToken
        );
        assert_eq!(
            CredentialRequestError::IssuanceError(IssuanceError::UnknownSession(String::from("test").into()))
                .error_code(),
            CredentialErrorCode::InvalidToken
        );
    }
}
