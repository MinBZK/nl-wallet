use base64::prelude::*;
use openid4vc::token::{TokenRequest, TokenRequestGrantType};
use url::Url;

use wallet_common::utils;

use crate::{
    pkce::{PkcePair, S256PkcePair},
    utils::url::url_find_first_query_value,
};

use super::{
    openid_client::{HttpOpenIdClient, OpenIdClient},
    DigidError, DigidSession,
};

const PARAM_ERROR: &str = "error";
const PARAM_ERROR_DESCRIPTION: &str = "error_description";
const PARAM_STATE: &str = "state";
const PARAM_CODE: &str = "code";

#[derive(Debug)]
pub struct HttpDigidSession<C = HttpOpenIdClient, P = S256PkcePair> {
    // The the redirect URL provided when starting the session, without query and fragment.
    redirect_uri_base: Url,
    // The discovered OpenID client.
    openid_client: C,
    /// CSRF token (stored in state parameter).
    csrf_token: String,
    /// The generated nonce that was used.
    nonce: String,
    /// The PKCE pair used.
    pkce_pair: P,
    /// Client ID at the OpenID issuer.
    client_id: String,
}

impl<C, P> DigidSession for HttpDigidSession<C, P>
where
    P: PkcePair + 'static,
    C: OpenIdClient,
{
    async fn start(issuer_url: Url, client_id: String, redirect_uri: Url) -> Result<Self, DigidError> {
        // Remember the `redirect_uri` base.
        let mut redirect_uri_base = redirect_uri.clone();
        redirect_uri_base.set_fragment(None);
        redirect_uri_base.set_query(None);

        // Perform OpenID discovery at the issuer.
        let openid_client = C::discover(issuer_url, client_id.clone(), redirect_uri).await?;

        // Generate a random CSRF token and nonce.
        let csrf_token = BASE64_URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let nonce = BASE64_URL_SAFE_NO_PAD.encode(utils::random_bytes(16));
        let pkce_pair = P::generate();

        // Store the client, generated tokens and auth url in a session for when the redirect URI returns.
        let session = HttpDigidSession {
            redirect_uri_base,
            openid_client,
            csrf_token,
            nonce,
            pkce_pair,
            client_id,
        };

        Ok(session)
    }

    fn auth_url(&self) -> Url {
        self.openid_client
            .auth_url(self.csrf_token.clone(), self.nonce.clone(), &self.pkce_pair)
    }

    fn matches_received_redirect_uri(&self, received_redirect_uri: &Url) -> bool {
        received_redirect_uri
            .as_str()
            .starts_with(self.redirect_uri_base.as_str())
    }

    fn get_authorization_code(&self, received_redirect_uri: &Url) -> Result<String, DigidError> {
        // Check if the redirect URL received actually belongs to us.
        if !self.matches_received_redirect_uri(received_redirect_uri) {
            return Err(DigidError::RedirectUriMismatch);
        }

        // Check if the `error` query parameter is populated, if so create an
        // error from it and a potential `error_description` query parameter.
        let error = url_find_first_query_value(received_redirect_uri, PARAM_ERROR);
        if let Some(error) = error {
            let error_description = url_find_first_query_value(received_redirect_uri, PARAM_ERROR_DESCRIPTION);
            let error = DigidError::RedirectUriError {
                error: error.into_owned(),
                error_description: error_description.map(|d| d.into_owned()),
            };

            return Err(error);
        }

        // Verify that the state query parameter matches the csrf_token.
        let state =
            url_find_first_query_value(received_redirect_uri, PARAM_STATE).ok_or(DigidError::StateTokenMismatch)?;

        if state != self.csrf_token {
            return Err(DigidError::StateTokenMismatch);
        }

        // Parse the authorization code from the redirect URL.
        let authorization_code =
            url_find_first_query_value(received_redirect_uri, PARAM_CODE).ok_or(DigidError::NoAuthCode)?;

        Ok(authorization_code.into_owned())
    }

    fn into_pre_authorized_code_request(self, pre_authorized_code: String) -> TokenRequest {
        TokenRequest {
            grant_type: TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code },
            code_verifier: Some(self.pkce_pair.code_verifier().to_string()),
            client_id: Some(self.client_id),
            redirect_uri: Some(self.redirect_uri_base.clone()),
        }
    }

    async fn get_access_token(self, received_redirect_uri: &Url) -> Result<String, DigidError> {
        let authorization_code = self.get_authorization_code(received_redirect_uri)?;

        // Use the authorization code and the PKCE verifier to request the
        // access token and verify the result.
        let access_token = self
            .openid_client
            .authenticate(&authorization_code, &self.nonce, &self.pkce_pair)
            .await?;

        Ok(access_token)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use serial_test::serial;

    use crate::{digid::openid_client::MockOpenIdClient, pkce::MockPkcePair, utils::url::url_with_query_pairs};

    use super::*;

    // These constants are used by multiple tests.
    const ISSUER_URL: &str = "http://example.com";
    const CLIENT_ID: &str = "client-1";
    const REDIRECT_URI: &str = "redirect://here";
    const CSRF_TOKEN: &str = "csrf_token";
    const NONCE: &str = "random_characters_nonce";
    const AUTH_URL: &str = "http://example.com/auth";
    const AUTH_CODE: &str = "the_authentication_code";
    const ACCESS_CODE: &str = "the_access_code";

    // Helper function for creating a `HttpDigidSession` with hardcoded state.
    fn create_digid_session() -> HttpDigidSession<MockOpenIdClient, MockPkcePair> {
        HttpDigidSession {
            redirect_uri_base: Url::parse(REDIRECT_URI).unwrap(),
            openid_client: MockOpenIdClient::new(),
            csrf_token: CSRF_TOKEN.to_string(),
            nonce: NONCE.to_string(),
            pkce_pair: MockPkcePair::new(),
            client_id: CLIENT_ID.to_string(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_http_digid_session_start_openid_error() {
        // Set up for an error being returned by OpenID discovery.
        let discover_context = MockOpenIdClient::discover_context();
        discover_context
            .expect()
            .return_once(|_, _, _| Err(openid::error::Error::CannotBeABase.into()));

        // Start a DigiD session, which should return an error.
        let error = HttpDigidSession::<MockOpenIdClient, MockPkcePair>::start(
            Url::parse(ISSUER_URL).unwrap(),
            CLIENT_ID.to_string(),
            Url::parse(REDIRECT_URI).unwrap(),
        )
        .await
        .expect_err("Starting DigiD session should have failed");

        assert_matches!(error, DigidError::OpenId(_));
    }

    #[tokio::test]
    #[serial]
    async fn test_http_digid_session_start() {
        // Create a redirect URI that should be stripped to its base.
        let redirect_uri = {
            let mut redirect_uri = Url::parse(REDIRECT_URI).unwrap();

            redirect_uri.set_query("foo=bar&bleh=blah".into());
            redirect_uri.set_fragment("test".into());

            redirect_uri
        };

        // Set up expectations, OpenId discovery and PKCE pair generation will
        // take place.
        let discover_context = MockOpenIdClient::discover_context();
        discover_context
            .expect()
            .with(
                eq(redirect_uri.clone()),
                eq(CLIENT_ID.to_string()),
                eq(Url::parse(REDIRECT_URI).unwrap()),
            )
            .return_once(|_, _, _| Ok(MockOpenIdClient::new()));

        let generate_context = MockPkcePair::generate_context();
        generate_context.expect().return_once(MockPkcePair::new);

        // Create the session and check the result.
        let session = HttpDigidSession::<MockOpenIdClient, MockPkcePair>::start(
            redirect_uri,
            CLIENT_ID.to_string(),
            Url::parse(REDIRECT_URI).unwrap(),
        )
        .await
        .expect("Could not start DigiD session");

        assert_eq!(session.redirect_uri_base.as_str(), REDIRECT_URI);
        assert!(!session.csrf_token.is_empty());
        assert!(!session.nonce.is_empty());
    }

    #[test]
    fn test_http_digid_session_auth_url() {
        // Create session and set up expectation of `OpenIdClient.auth_url_and_pkce()` being called.
        let session = {
            let mut session = create_digid_session();

            session
                .openid_client
                .expect_auth_url()
                .with(eq(CSRF_TOKEN.to_string()), eq(NONCE.to_string()), always())
                .return_once(|_, _, _: &MockPkcePair| Url::parse(AUTH_URL).unwrap());

            session
        };

        // The authentication URl returned should match the expectation above.
        let auth_url = session.auth_url();

        assert_eq!(auth_url, Url::parse(AUTH_URL).unwrap());
    }

    #[test]
    fn test_http_digid_session_matches_received_redirect_uri() {
        let session = create_digid_session();

        // These URIs should match the `base_redirect_uri`.
        assert!(session.matches_received_redirect_uri(&Url::parse(REDIRECT_URI).unwrap()));
        assert!(session.matches_received_redirect_uri(&url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[("foo", "bar"), ("bleh", "blah")]
        )));

        // These URIs should NOT match the `base_redirect_uri`.
        assert!(!session.matches_received_redirect_uri(&Url::parse("https://example.com").unwrap()));
        assert!(!session.matches_received_redirect_uri(&Url::parse("scheme://host/path").unwrap()));
    }

    // Helper function for testing `HttpDigidSession.get_access_token()`
    // calls that should result in an error.
    async fn create_session_and_get_access_token_error(uri: &Url) -> DigidError {
        let session = create_digid_session();

        session
            .get_access_token(uri)
            .await
            .expect_err("Getting access token should have failed")
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_token_redirect_uri_mismatch() {
        // This URI does not match the `redirect_uri_base`.
        let uri = Url::parse("http://not-the-redirect-uri.com").unwrap();
        let error = create_session_and_get_access_token_error(&uri).await;

        assert_matches!(error, DigidError::RedirectUriMismatch);
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_token_redirect_uri_error_description() {
        // This URI contains an `error` query parameter, with an `error_description`.
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[
                (PARAM_ERROR, "error_type"),
                (PARAM_ERROR_DESCRIPTION, "this is the error description"),
            ],
        );

        let error = create_session_and_get_access_token_error(&uri).await;

        assert_matches!(error, DigidError::RedirectUriError {
            ref error,
            error_description: Some(ref error_description)
        } if error == "error_type" && error_description == "this is the error description");
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_token_redirect_uri_error() {
        // This URI contains an `error` query parameter, without an `error_description`.
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[(PARAM_ERROR, "foobar")]);

        let error = create_session_and_get_access_token_error(&uri).await;

        assert_matches!(error, DigidError::RedirectUriError {
            ref error,
            error_description: _
        } if error == "foobar");
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_token_state_token_mismatch() {
        // This URI contains an incorrect `state` query parameter.
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[(PARAM_STATE, "foobar")]);

        let error = create_session_and_get_access_token_error(&uri).await;

        assert_matches!(error, DigidError::StateTokenMismatch);
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_token_no_auth_code() {
        // This URI is missing the `code` query parameter.
        let uri = url_with_query_pairs(Url::parse(REDIRECT_URI).unwrap(), &[(PARAM_STATE, CSRF_TOKEN)]);

        let error = create_session_and_get_access_token_error(&uri).await;

        assert_matches!(error, DigidError::NoAuthCode);
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access_openid_error() {
        // Create session and set up expectation to have `OpenIdClient.authenticate()`
        // return an error.
        let session = {
            let mut session = create_digid_session();

            session
                .openid_client
                .expect_authenticate()
                .return_once(|_, _, _: &MockPkcePair| Err(openid::error::Error::MissingOpenidScope.into()));

            session
        };

        // Create a valid redirect URI.
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[(PARAM_STATE, CSRF_TOKEN), (PARAM_CODE, AUTH_CODE)],
        );

        // Get the access token and test the resulting error.
        let error = session
            .get_access_token(&uri)
            .await
            .expect_err("Getting access token should have failed");

        assert_matches!(error, DigidError::OpenId(_));
    }

    #[tokio::test]
    async fn test_http_digid_session_get_access() {
        // Create session and set up expectation to have `OpenIdClient.authenticate()`
        // return an access token.
        let session = {
            let mut session = create_digid_session();

            session
                .openid_client
                .expect_authenticate()
                .with(eq(AUTH_CODE), eq(NONCE), always())
                .return_once(|_, _, _: &MockPkcePair| Ok(ACCESS_CODE.to_string()));

            session
        };

        // Create a valid redirect URI.
        let uri = url_with_query_pairs(
            Url::parse(REDIRECT_URI).unwrap(),
            &[(PARAM_STATE, CSRF_TOKEN), (PARAM_CODE, AUTH_CODE)],
        );

        // Get the access token and test the result.
        let access_token = session
            .get_access_token(&uri)
            .await
            .expect("Could not get access token");

        assert_eq!(access_token, ACCESS_CODE);
    }
}
