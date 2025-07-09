use std::sync::LazyLock;

use mime::Mime;

use http_utils::urls::BaseUrl;
use jwt::Jwt;

use crate::errors::AuthorizationErrorCode;
use crate::errors::ErrorResponse;
use crate::errors::VpAuthorizationErrorCode;
use crate::openid4vp::VpAuthorizationRequest;

pub use self::error::VpMessageClientError;
pub use self::error::VpMessageClientErrorType;
pub use self::http::HttpVpMessageClient;

mod error;
mod http;

#[cfg(test)]
pub mod mock;

pub static APPLICATION_OAUTH_AUTHZ_REQ_JWT: LazyLock<Mime> = LazyLock::new(|| {
    "application/oauth-authz-req+jwt"
        .parse()
        .expect("could not parse MIME type")
});

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
