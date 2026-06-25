use error_category::ErrorCategory;
use url::Url;

use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(critical)]
pub enum AuthorizationEndpointsError {
    #[error("oauth metadata has no authorization endpoint")]
    NoAuthorizationEndpoint,

    #[error("oauth metadata has no pushed authorization request endpoint")]
    NoPushedAuthorizationEndpoint,
}

/// Helper type that is internal to the `wallet_issuance` sub-module. It represents the subset of OAuth 2.0 metadata
/// that is relevant to the Authorization Code flow.
#[derive(Debug, Clone)]
pub struct AuthorizationEndpoints {
    pub authorization_endpoint: Url,
    pub par_endpoint: Url,
    pub token_endpoint: Url,
}

impl TryFrom<AuthorizationServerMetadata> for AuthorizationEndpoints {
    type Error = AuthorizationEndpointsError;

    fn try_from(value: AuthorizationServerMetadata) -> Result<Self, Self::Error> {
        let authorization_endpoint = value
            .authorization_endpoint
            .ok_or(AuthorizationEndpointsError::NoAuthorizationEndpoint)?;

        let pushed_authorization_request_endpoint = value
            .pushed_authorization_request_endpoint
            .ok_or(AuthorizationEndpointsError::NoPushedAuthorizationEndpoint)?;

        let endpoints = Self {
            authorization_endpoint,
            par_endpoint: pushed_authorization_request_endpoint,
            token_endpoint: value.token_endpoint,
        };

        Ok(endpoints)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::AuthorizationEndpoints;
    use super::AuthorizationEndpointsError;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;

    const ISSUER_URL: &str = "https://example.com";

    #[test]
    fn authorization_endpoints_try_from_authorization_server_metadata_ok() {
        let oauth_metadata = AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap());

        let _endpoints = AuthorizationEndpoints::try_from(oauth_metadata)
            .expect("extract authorization enpoints from OAuth metadata should succeed");
    }

    #[test]
    fn authorization_endpoints_try_from_authorization_server_metadata_error_no_authorization_endpoint() {
        let mut oauth_metadata = AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap());
        oauth_metadata.authorization_endpoint = None;

        let error = AuthorizationEndpoints::try_from(oauth_metadata)
            .expect_err("extract authorization enpoints from OAuth metadata should fail");

        assert_matches!(error, AuthorizationEndpointsError::NoAuthorizationEndpoint);
    }

    #[test]
    fn authorization_endpoints_try_from_authorization_server_metadata_error_no_par_endpoint() {
        let mut oauth_metadata = AuthorizationServerMetadata::new_mock(ISSUER_URL.parse().unwrap());
        oauth_metadata.pushed_authorization_request_endpoint = None;

        let error = AuthorizationEndpoints::try_from(oauth_metadata)
            .expect_err("extract authorization enpoints from OAuth metadata should fail");

        assert_matches!(error, AuthorizationEndpointsError::NoPushedAuthorizationEndpoint);
    }
}
