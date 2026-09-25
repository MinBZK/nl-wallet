//! OIDC login flow for the Admin Portal, authenticating against Keycloak.
//!
//! Handles the PKCE authorization code flow, id_token/access_token validation against Keycloak's
//! JWKS, and the resulting server-side sessions. Sessions and pending logins are persisted through
//! an [`AdminPortalSessionRepository`], so that they are shared between replicas rather than kept
//! in memory on a single instance; this is important since the Admin Portal runs with more than
//! one replica.

use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::DateTime;
use chrono::Utc;
use crypto::utils::random_bytes;
use http_utils::reqwest::HttpClient;
use http_utils::urls::BaseUrl;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::errors::Error as JsonWebTokenError;
use jsonwebtoken::jwk::JwkSet;
use jwt::Algorithm;
use jwt::Header;
use jwt::nonce::Nonce;
use oauth::authorization::AuthorizationCodeRequest;
use oauth::authorization::OidcAuthorizationRequest;
use oauth::issuer_identifier::IssuerIdentifier;
use oauth::metadata::well_known::WellKnownMetadata;
use oauth::pkce::PkcePair;
use oauth::pkce::S256PkcePair;
use oauth::scope::Scope;
use oauth::token::AuthorizationCode;
use oauth::token::AuthorizationCodeGrantType;
use oauth::token::TokenRequest;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::try_join;
use tracing::debug;
use tracing::info;
use tracing::warn;
use url::Url;
use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
use wallet_provider_domain::repository::AdminPortalSessionRepository;
use wallet_provider_domain::repository::PersistenceError;

use self::oidc_client::OidcHttpClientError;
use crate::admin_portal::oidc_client::OidcHttpClient;

pub mod oidc_client;

const PRIVILEGE_ROLE_PREFIX: &str = "privilege_";

/// Scopes requested from Keycloak for the Admin Portal login: `openid` for the OIDC flow and `profile` for the
/// user's display name.
static ADMIN_SCOPES: LazyLock<HashSet<Scope>> = LazyLock::new(|| {
    HashSet::from([
        "openid".parse().expect("\"openid\" is a valid scope"),
        "profile".parse().expect("\"profile\" is a valid scope"),
    ])
});

/// The logged-in user, as reported to the Admin Portal frontend.
#[derive(Debug)]
pub struct LoggedInUser {
    pub display_name: String,
    pub privileges: Vec<String>,
}

/// Errors from [`AdminPortalService::authorization_url`] (the `/auth/login` flow).
#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("OIDC provider error: {0}")]
    Oidc(#[source] OidcHttpClientError),
    #[error("session storage error: {0}")]
    Storage(#[source] PersistenceError),
}

/// Errors from validating the id_token/access_token during the `/auth/callback` flow.
#[derive(Debug, thiserror::Error)]
pub enum TokenVerificationError {
    #[error("web token error: {0}")]
    Token(#[source] JsonWebTokenError),
    #[error("invalid nonce")]
    InvalidNonce,
    #[error("missing kid claim")]
    MissingKeyIdentifier,
    #[error("unsupported algorithm: {0:?}")]
    UnsupportedAlgorithm(Algorithm),
}

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum CallbackError {
    #[error("unknown or expired state")]
    InvalidState,
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("missing code")]
    MissingCode,
    #[error("token exchange failed")]
    TokenExchangeFailed,
    #[error("id_token validation failed")]
    InvalidToken,
    #[error("OIDC provider error: {0}")]
    Oidc(#[source] OidcHttpClientError),
    #[error("session storage error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("token verification error: {0}")]
    TokenVerification(#[source] TokenVerificationError),
}

impl From<OidcHttpClientError> for CallbackError {
    fn from(error: OidcHttpClientError) -> Self {
        // Transport and parse errors during the token exchange are surfaced distinctly, so that the
        // frontend can distinguish a transient upstream failure from an invalid grant.
        if matches!(error, OidcHttpClientError::TokenEndpoint(_)) {
            CallbackError::TokenExchangeFailed
        } else {
            CallbackError::Oidc(error)
        }
    }
}

impl CallbackError {
    /// Machine-readable reason code, meant to be forwarded to the frontend's error page.
    pub fn reason(&self) -> &str {
        match self {
            Self::Upstream(reason) => reason.as_str(),
            other => other.into(),
        }
    }
}

pub struct AdminPortalService<R> {
    keycloak_client_id: String,
    redirect_uri: Url,
    session_ttl: Duration,
    login_attempt_ttl: Duration,
    oidc_client: OidcHttpClient,
    repository: R,
}

/// The Keycloak/session parameters needed to construct an [`AdminPortalService`].
pub struct AdminPortalConfig {
    pub keycloak_url: BaseUrl,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub public_url: BaseUrl,
    pub session_ttl: Duration,
    pub login_attempt_ttl: Duration,
}

impl<R> AdminPortalService<R> {
    pub fn new(config: AdminPortalConfig, client: reqwest::Client, repository: R) -> Self {
        let issuer_url = config
            .keycloak_url
            .join_base_url(&format!("realms/{}", config.keycloak_realm));
        let issuer: IssuerIdentifier = issuer_url.to_string().parse().unwrap();

        Self {
            keycloak_client_id: config.keycloak_client_id,
            redirect_uri: config.public_url.join("/admin-portal/auth/callback"),
            session_ttl: config.session_ttl,
            login_attempt_ttl: config.login_attempt_ttl,
            oidc_client: OidcHttpClient::new(HttpClient::new(client), issuer),
            repository,
        }
    }

    pub fn session_ttl(&self) -> Duration {
        self.session_ttl
    }

    fn expires_at(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        now + self.session_ttl
    }
}

const SUPPORTED_ALGS: &[Algorithm] = &[
    Algorithm::HS256,
    Algorithm::HS384,
    Algorithm::HS512,
    Algorithm::ES256,
    Algorithm::ES384,
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::PS256,
    Algorithm::PS384,
    Algorithm::PS512,
    Algorithm::EdDSA,
];

fn jwt_validation_from_header(header: &Header) -> Option<Validation> {
    let alg: Algorithm = header.alg;
    SUPPORTED_ALGS.contains(&alg).then_some(Validation::new(alg))
}

impl<R> AdminPortalService<R>
where
    R: AdminPortalSessionRepository,
{
    /// Best-effort garbage collection of expired login attempts and sessions. Failures are logged
    /// and otherwise ignored, since this is housekeeping and should not block the login flow.
    async fn cleanup_expired(&self) {
        let now = Utc::now();

        if let Err(error) = self
            .repository
            .cleanup_expired_login_attempts(now - self.login_attempt_ttl)
            .await
        {
            warn!("failed to clean up expired admin portal login attempts: {error}");
        }
        if let Err(error) = self.repository.cleanup_expired_user_sessions(now).await {
            warn!("failed to clean up expired admin portal sessions: {error}");
        }
    }

    /// Verifies the signature, issuer, audience and nonce of an id_token.
    fn verify_id_token(
        &self,
        token: &str,
        nonce: &str,
        issuer: &IssuerIdentifier,
        jwks: &JwkSet,
    ) -> Result<IdTokenClaims, TokenVerificationError> {
        let header = jsonwebtoken::decode_header(token).map_err(TokenVerificationError::Token)?;

        // Create validation with algorithm from header
        let mut validation =
            jwt_validation_from_header(&header).ok_or(TokenVerificationError::UnsupportedAlgorithm(header.alg))?;
        validation.set_issuer(&[issuer.as_ref()]);
        validation.set_audience(&[self.keycloak_client_id.as_str()]);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);

        let claims: IdTokenClaims = Self::decode_and_validate(token, jwks, header, &validation)?;

        if claims.nonce.as_deref() != Some(nonce) {
            return Err(TokenVerificationError::InvalidNonce);
        }
        Ok(claims)
    }

    /// Verifies the signature and issuer of an access_token. Keycloak puts realm roles here rather
    /// than in the id_token, see `wallet_docs/development/keycloak-setup.md`.
    fn verify_access_token(
        token: &str,
        issuer: &IssuerIdentifier,
        jwks: &JwkSet,
    ) -> Result<AccessTokenClaims, TokenVerificationError> {
        let header = jsonwebtoken::decode_header(token).map_err(TokenVerificationError::Token)?;

        // Create validation with algorithm from header
        let mut validation =
            jwt_validation_from_header(&header).ok_or(TokenVerificationError::UnsupportedAlgorithm(header.alg))?;
        validation.set_issuer(&[issuer.as_ref()]);
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["exp", "iss"]);

        Self::decode_and_validate(token, jwks, header, &validation)
    }

    fn decode_and_validate<T: DeserializeOwned>(
        token: &str,
        jwks: &JwkSet,
        header: jwt::Header,
        validation: &Validation,
    ) -> Result<T, TokenVerificationError> {
        let kid = header.kid.ok_or(TokenVerificationError::MissingKeyIdentifier)?;
        let key = DecodingKey::from_jwk(jwks.find(&kid).ok_or(TokenVerificationError::MissingKeyIdentifier)?)
            .map_err(TokenVerificationError::Token)?;
        let data = jsonwebtoken::decode::<T>(token, &key, validation).map_err(TokenVerificationError::Token)?;
        Ok(data.claims)
    }

    /// Starts a login attempt: stores PKCE/nonce state for the pending exchange and returns the
    /// Keycloak authorization URL that the browser should be redirected to.
    pub async fn authorization_url(&self, login_hint: Option<&str>) -> Result<Url, LoginError> {
        self.cleanup_expired().await;
        let metadata = self.oidc_client.fetch_metadata().await.map_err(LoginError::Oidc)?;

        let session_state = random_token();
        let nonce = Nonce::new_random();
        let pkce_pair = S256PkcePair::generate();

        self.repository
            .insert_login_attempt(
                session_state.clone(),
                AdminPortalLoginAttempt {
                    nonce: nonce.to_string(),
                    code_verifier: pkce_pair.code_verifier().to_string(),
                    created_at: Utc::now(),
                },
            )
            .await
            .map_err(LoginError::Storage)?;

        let scope: &HashSet<Scope> = &ADMIN_SCOPES;
        let oidc_request = OidcAuthorizationRequest {
            auth_request: AuthorizationCodeRequest::new(
                self.keycloak_client_id.to_string(),
                self.redirect_uri.clone(),
                session_state.to_string(),
                scope.clone(),
                &pkce_pair,
            ),
            nonce: Some(nonce.clone()),
            login_hint: login_hint.map(str::to_string),
        };
        let auth_url = self
            .oidc_client
            .authorization_url(&oidc_request, &metadata)
            .map_err(LoginError::Oidc)?;

        info!("login redirect issued for issuer {}", metadata.issuer_identifier());

        Ok(auth_url)
    }

    /// Handles the OIDC callback: exchanges the authorization code for tokens, validates them and
    /// starts a server-side session. Returns the new session id on success.
    pub async fn handle_callback(
        &self,
        state: Option<&str>,
        code: Option<&str>,
        error: Option<&str>,
    ) -> Result<String, CallbackError> {
        self.cleanup_expired().await;

        let login_attempt = match state {
            Some(session_state) => self
                .repository
                .take_login_attempt(session_state)
                .await
                .map_err(CallbackError::Storage)?,
            None => None,
        }
        .ok_or(CallbackError::InvalidState)?;

        if let Some(error) = error {
            return Err(CallbackError::Upstream(error.to_string()));
        }

        let code = code.ok_or(CallbackError::MissingCode)?;

        // Obtain metadata
        let metadata = self.oidc_client.fetch_metadata().await.map_err(CallbackError::Oidc)?;

        // Request Token
        let token_request = TokenRequest {
            grant_type: AuthorizationCodeGrantType::AuthorizationCode {
                code: AuthorizationCode::from(code.to_string()),
            },
            client_id: Some(self.keycloak_client_id.to_string()),
            redirect_uri: Some(self.redirect_uri.clone()),
            scope: None,
            code_verifier: Some(login_attempt.code_verifier.to_string()),
        };
        let (token_response, jwks) = try_join!(
            async {
                self.oidc_client
                    .exchange_code(token_request, &metadata)
                    .await
                    .map_err(CallbackError::from)
            },
            async {
                self.oidc_client
                    .fetch_jwks(&metadata)
                    .await
                    .map_err(CallbackError::Oidc)
            },
        )?;

        // Verify id_token
        let issuer = metadata.issuer_identifier();
        let id_token = self
            .verify_id_token(&token_response.id_token, &login_attempt.nonce, issuer, &jwks)
            .map_err(CallbackError::TokenVerification)?;

        // Extract roles from access token
        let roles = Self::verify_access_token(token_response.token_response.access_token.as_ref(), issuer, &jwks)
            .map(|claims| claims.realm_access.roles)
            .map_err(CallbackError::TokenVerification)?;

        // Store session and return session ID
        let session_id = random_token();
        self.repository
            .insert_user_session(
                session_id.clone(),
                AdminPortalUserSession {
                    display_name: id_token.name,
                    roles,
                    id_token: token_response.id_token,
                    expires_at: self.expires_at(Utc::now()),
                },
            )
            .await
            .map_err(CallbackError::Storage)?;

        info!("admin portal session created");

        Ok(session_id)
    }

    /// Ends the session identified by `session_id` (if any).
    /// This method is idempotent, so it returns `()` both if `session_id` was removed and when it did not match a
    /// known session. It returns an error for any database error.
    pub async fn end_session(&self, session_id: &str) -> Result<(), PersistenceError> {
        self.cleanup_expired().await;

        match self.repository.take_user_session(session_id).await? {
            Some(_session) => Ok(()),
            None => {
                debug!("session not found");
                Ok(())
            }
        }
    }

    /// Reports the logged-in user for `session_id`, extending (sliding) the session's expiry.
    /// Returns `Ok(None)` if there is no matching session, and `Err` if the session store could
    /// not be reached, so callers can distinguish "not logged in" from a backend failure.
    pub async fn authenticated_user(&self, session_id: &str) -> Result<Option<LoggedInUser>, PersistenceError> {
        self.cleanup_expired().await;

        let now = Utc::now();
        // Sliding expiry: every authenticated call extends the session.
        // TODO(PVW-6127): Only extend on specific api calls.
        let session = self
            .repository
            .touch_user_session(session_id, now, self.expires_at(now))
            .await
            .inspect_err(|error| warn!("failed to extend admin portal session: {error}"))?;

        Ok(session.map(|session| LoggedInUser {
            display_name: session.display_name,
            privileges: to_privileges(&session.roles),
        }))
    }
}

fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(random_bytes(32))
}

// Keycloak realm roles are named "privilege_<name>", remove this prefix.
fn to_privileges(roles: &[String]) -> Vec<String> {
    roles
        .iter()
        .filter_map(|role| role.strip_prefix(PRIVILEGE_ROLE_PREFIX))
        .map(str::to_string)
        .collect()
}

#[derive(Deserialize)]
struct IdTokenClaims {
    name: String,
    nonce: Option<String>,
}

#[derive(Deserialize)]
struct AccessTokenClaims {
    #[serde(default)]
    realm_access: RealmAccess,
}

#[derive(Default, Deserialize)]
struct RealmAccess {
    #[serde(default)]
    roles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    use chrono::Utc;
    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::urls::BaseUrl;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::EncodingKey;
    use jsonwebtoken::Header;
    use jsonwebtoken::jwk::Jwk;
    use jsonwebtoken::jwk::JwkSet;
    use mockall::predicate;
    use serde::Serialize;
    use serde_json::json;
    use url::Url;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
    use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
    use wallet_provider_domain::repository::PersistenceError;
    use wallet_provider_persistence::repositories::mock::MockAdminPortalSessionRepository;

    use crate::admin_portal::AdminPortalConfig;
    use crate::admin_portal::AdminPortalService;
    use crate::admin_portal::CallbackError;
    use crate::admin_portal::LoginError;
    use crate::admin_portal::TokenVerificationError;
    use crate::admin_portal::to_privileges;

    // Test-only 2048 bit RSA key, used to sign fake Keycloak tokens and serve a matching JWKS.
    const TEST_RSA_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDFBdBNJ+SqMj/s
0LbzYEGnB3RlLRgzTsFvP0Qj059xO99bl18ChMm2RF9QdRw5koaXoLO4/J0/Cnd9
yDlmeADnYFilac2b/s10iCTCafkM7iEd0mJ34WWhvxKBGum+MgGUhXZhkK9B2GVO
1X+rqozVck/TfgN/9UWh+I/i/nkeOOPv4oB1jEpzBOkIzDCbfmGK3dZwMh5T8cc2
gFj2x7LQjlAEObDPnIlJe2j8WgXRhfRcvDSJp08+mA+8Q+0QWbHBlQ7GRDkcj7tB
tk9pdg8QhsOKMc1jLybX9rkCNihEPVYYMvxeEowo2GMrhiHLLTXelRB7h1LToMK2
xo3H0bQDAgMBAAECggEAIxgFgfDErMI1m7+TjudK2m8b/veY027ISsIIp58Gy/sf
rmdYj5DKgzJLjf/GLsUYP0LrMFyiv7tkDF4RR1zBwHTrZU9ixdINk5+6eHy61WBH
OtIiIvtdiIGJ0MBT+UJcALIDI57LcN2UMgYabx/6ZPyvFltgUTcFl7O4IXU1arnk
ai/2KqaVCkyoXYcJXLfLoDkiSumMW9vNFkv911q5vPqxjGmoChSS1+eZGrKpLLpm
ex0CihYoh4ZlCooFTTpFEOXBBig+j5rRnVyLru+gFSgrcJ+Xix6IcGwsoHetpISs
37MqJqlyOvYx/ta9W9Ng+TZXv/mvSdnXu7iMQp2MAQKBgQDm6Wa9h9gYaYH/PzGi
UBBLuVN/d0ko/6ocPxiRODhzuUhis4wbOYD8khcPSucp2iVPLfbt2MDfX6NXSdvk
xMzvQXXF9JoyQ32WrAI5p1k1AtdVpzs1mK9Y+B1a5YQwZ4G18P8pRcd7+CvwcGZ7
VdoAi39zDd9AZeZqL0qAEIvoAwKBgQDabdID8AXkO2il/27fqQ6HU+44ObSpBpmv
8UCWmkOQXrj+QPXxhdJ1Z75UOeFjqSP7dH9OGesr3Keq/omtMUESpqg9NYFwwyHt
fh09CQne9afI1bpQi/bRLQSwdMgHAvX+pC5Eww9iEV7DFO7OyUHHQX1YQQ1k0aNf
TeEV5zdEAQKBgHcSlqNXoKx+A8YezTZ4+N5Dk/YgCf71T8A/HSkNh7bNLbGQCsij
L4uOvgtpwaiIUELzXekqo9LMG4vQj275uQALjnLk/nq66NFAo+kdDdhTPb1yfgrW
UF2dnG2Z+z+GsJGk079xtzuLLwVOwNoK3F75kGBBIAWyRk4tUsqVPcAHAoGAUIDN
OHpMEZP7u8JqwK/0FNhQIhTSisFN/1RxM5BjemAO2lZizsM9j9vOgAhdE3gRNOn4
yXYwAJhwi0sIvvY6P3+A3h5MOJ0Scg9bA1XDd5MeZZyv8GPFcc6fvdEsr7jdpR4p
l1o03zX7sPaUFU6DmcZ/Rfmj0BabmdKANKxk6AECgYBRDEJcxu1NrWuylLBBweSY
4cgKgMSopnh1/XOWdjg8KMTYkW8NXrhHC2RiNBaZ8US/pYX7Vl/B0zALT2h6t5oH
DKmgWO9o/v+8nXMAjML3aoyqXc9UBoTcgTmmsFoOzvCrTSo5MajfblL7aYakcZh3
yP3ST1F3e7Cha7l54e71Lg==
-----END PRIVATE KEY-----";

    const TEST_KID: &str = "test-kid";
    const TEST_CLIENT_ID: &str = "test-client";
    const TEST_REALM: &str = "test-realm";

    #[derive(Serialize)]
    struct TestIdTokenClaims<'a> {
        iss: &'a str,
        aud: &'a str,
        sub: &'a str,
        name: &'a str,
        exp: i64,
        iat: i64,
        nonce: &'a str,
    }

    #[derive(Serialize)]
    struct TestRealmAccess {
        roles: Vec<String>,
    }

    #[derive(Serialize)]
    struct TestAccessTokenClaims<'a> {
        iss: &'a str,
        exp: i64,
        iat: i64,
        realm_access: TestRealmAccess,
    }

    /// Like [`TestAccessTokenClaims`], but missing the `iss` claim that Keycloak access tokens are
    /// required to carry. Used to simulate an access_token that fails validation.
    #[derive(Serialize)]
    struct AccessTokenClaimsMissingIssuer {
        exp: i64,
        iat: i64,
        realm_access: TestRealmAccess,
    }

    fn public_url() -> BaseUrl {
        BaseUrl::try_new(Url::parse("https://portal.example.org/").unwrap()).unwrap()
    }

    fn service_with_repository(
        keycloak_url: &BaseUrl,
        repository: MockAdminPortalSessionRepository,
    ) -> AdminPortalService<MockAdminPortalSessionRepository> {
        service_with_client(keycloak_url, reqwest::Client::new(), repository)
    }

    fn service_with_client(
        keycloak_url: &BaseUrl,
        client: reqwest::Client,
        repository: MockAdminPortalSessionRepository,
    ) -> AdminPortalService<MockAdminPortalSessionRepository> {
        AdminPortalService::new(
            AdminPortalConfig {
                keycloak_url: keycloak_url.clone(),
                keycloak_realm: TEST_REALM.to_string(),
                keycloak_client_id: TEST_CLIENT_ID.to_string(),
                public_url: public_url(),
                session_ttl: Duration::from_mins(60),
                login_attempt_ttl: Duration::from_mins(10),
            },
            client,
            repository,
        )
    }

    /// Every public `AdminPortalService` method starts with a best-effort cleanup of expired
    /// sessions, so every test needs expectations for it.
    fn expect_cleanup(repository: &mut MockAdminPortalSessionRepository) {
        repository.expect_cleanup_expired_login_attempts().returning(|_| Ok(()));
        repository.expect_cleanup_expired_user_sessions().returning(|_| Ok(()));
    }

    fn encoding_key() -> EncodingKey {
        EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_KEY_PEM.as_bytes()).unwrap()
    }

    fn sign(claims: &impl Serialize) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_string());

        jsonwebtoken::encode(&header, claims, &encoding_key()).unwrap()
    }

    fn login_attempt(nonce: &str) -> AdminPortalLoginAttempt {
        AdminPortalLoginAttempt {
            nonce: nonce.to_string(),
            code_verifier: "test-code-verifier".to_string(),
            created_at: Utc::now(),
        }
    }

    /// A fake Keycloak server that serves a JWKS matching [`TEST_RSA_PRIVATE_KEY_PEM`] and a token
    /// endpoint, so that the full callback flow (including signature verification) can be tested.
    struct TestOidc {
        server: MockServer,
        keycloak_url: BaseUrl,
        issuer: String,
    }

    impl TestOidc {
        async fn start() -> Self {
            let server = MockServer::start_async().await;
            let keycloak_url = BaseUrl::from_str(&server.base_url()).unwrap();
            let issuer = keycloak_url.join_base_url(&format!("realms/{TEST_REALM}")).to_string();

            Self {
                server,
                keycloak_url,
                issuer,
            }
        }

        async fn mock_metadata(&self) {
            self.server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(format!("/.well-known/openid-configuration/realms/{TEST_REALM}"));
                    then.status(200).json_body(json!({
                        "issuer": self.issuer,
                        "authorization_endpoint": format!("{}/protocol/openid-connect/auth", self.issuer),
                        "token_endpoint": format!("{}/protocol/openid-connect/token", self.issuer),
                        "jwks_uri": format!("{}/protocol/openid-connect/certs", self.issuer),
                        "response_types_supported": ["code"],
                        "code_challenge_methods_supported": ["S256"]
                    }));
                })
                .await;
        }

        async fn mock_jwks(&self) {
            let mut jwk = Jwk::from_encoding_key(&encoding_key(), Algorithm::RS256).unwrap();
            jwk.common.key_id = Some(TEST_KID.to_string());
            let jwks = JwkSet { keys: vec![jwk] };

            self.server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(format!("/realms/{TEST_REALM}/protocol/openid-connect/certs"));
                    then.status(200).json_body(serde_json::to_value(&jwks).unwrap());
                })
                .await;
        }

        async fn mock_token_endpoint(&self, id_token: String, access_token: String) {
            self.server
                .mock_async(move |when, then| {
                    when.method(POST)
                        .path(format!("/realms/{TEST_REALM}/protocol/openid-connect/token"));
                    then.status(200).json_body(json!({
                        "id_token": id_token,
                        "access_token": access_token,
                        "token_type": "Bearer"
                    }));
                })
                .await;
        }

        async fn mock_token_endpoint_error(&self) {
            self.server
                .mock_async(|when, then| {
                    when.method(POST)
                        .path(format!("/realms/{TEST_REALM}/protocol/openid-connect/token"));
                    then.status(500);
                })
                .await;
        }

        /// Builds a service pointed at this mock server. Uses a client that trusts httpmock's
        /// pinned root CA, since httpmock may or may not serve TLS depending on which features are
        /// active for it elsewhere in the build (its "https" feature is unified workspace-wide).
        fn service(
            &self,
            repository: MockAdminPortalSessionRepository,
        ) -> AdminPortalService<MockAdminPortalSessionRepository> {
            let client = httpmock_reqwest_client_builder().build().unwrap();

            service_with_client(&self.keycloak_url, client, repository)
        }
    }

    #[test]
    fn to_privileges_strips_prefix_and_ignores_non_privilege_roles() {
        let roles = vec![
            "privilege_admin".to_string(),
            "privilege_support".to_string(),
            "offline_access".to_string(),
        ];

        assert_eq!(to_privileges(&roles), vec!["admin".to_string(), "support".to_string()]);
    }

    #[test]
    fn callback_error_reason_uses_snake_case_variant_name_except_for_upstream() {
        assert_eq!(CallbackError::InvalidState.reason(), "invalid_state");
        assert_eq!(CallbackError::MissingCode.reason(), "missing_code");
        assert_eq!(CallbackError::TokenExchangeFailed.reason(), "token_exchange_failed");
        assert_eq!(CallbackError::InvalidToken.reason(), "invalid_token");
        assert_eq!(
            CallbackError::Storage(PersistenceError::NoRowsUpdated).reason(),
            "storage"
        );
        assert_eq!(
            CallbackError::Upstream("custom_reason".to_string()).reason(),
            "custom_reason"
        );
    }

    #[test]
    fn session_ttl_returns_configured_duration() {
        let repository = MockAdminPortalSessionRepository::new();
        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        assert_eq!(service.session_ttl(), Duration::from_secs(3600));
    }

    #[tokio::test]
    async fn authorization_url_stores_login_attempt_and_builds_correct_url() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);

        let captured = Arc::new(Mutex::new(None));
        let captured_clone = Arc::clone(&captured);
        repository
            .expect_insert_login_attempt()
            .returning(move |state, session| {
                *captured_clone.lock().unwrap() = Some((state, session));
                Ok(())
            });

        let service = oidc.service(repository);

        let url = service.authorization_url(None).await.unwrap();

        let (state, session) = captured.lock().unwrap().take().unwrap();
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();

        assert_eq!(query["response_type"], "code");
        assert_eq!(query["client_id"], TEST_CLIENT_ID);
        assert_eq!(
            query["redirect_uri"],
            public_url().join("/admin-portal/auth/callback").to_string()
        );
        assert_eq!(
            query["scope"].split(' ').collect::<HashSet<_>>(),
            HashSet::from(["openid", "profile"])
        );
        assert_eq!(query["state"], state);
        assert_eq!(query["nonce"], session.nonce);
        assert_eq!(query["code_challenge_method"], "S256");
        assert!(!query["code_challenge"].is_empty());
        assert!(!query.contains_key("login_hint"));
    }

    #[tokio::test]
    async fn authorization_url_includes_login_hint_when_provided() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository.expect_insert_login_attempt().returning(|_, _| Ok(()));

        let service = oidc.service(repository);

        let url = service.authorization_url(Some("user@example.org")).await.unwrap();

        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["login_hint"], "user@example.org");
    }

    #[tokio::test]
    async fn authorization_url_propagates_storage_error() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_insert_login_attempt()
            .returning(|_, _| Err(PersistenceError::NoRowsUpdated));

        let service = oidc.service(repository);

        let result = service.authorization_url(None).await;

        assert_matches!(result, Err(LoginError::Storage(PersistenceError::NoRowsUpdated)));
    }

    #[tokio::test]
    async fn handle_callback_without_state_returns_invalid_state() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let result = service.handle_callback(None, Some("code"), None).await;

        assert_matches!(result, Err(CallbackError::InvalidState));
    }

    #[tokio::test]
    async fn handle_callback_with_unknown_state_returns_invalid_state() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .with(predicate::eq("unknown-state"))
            .returning(|_| Ok(None));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let result = service.handle_callback(Some("unknown-state"), Some("code"), None).await;

        assert_matches!(result, Err(CallbackError::InvalidState));
    }

    #[tokio::test]
    async fn handle_callback_maps_login_attempt_storage_error() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .returning(|_| Err(PersistenceError::NoRowsUpdated));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let result = service.handle_callback(Some("state"), Some("code"), None).await;

        assert_matches!(result, Err(CallbackError::Storage(PersistenceError::NoRowsUpdated)));
    }

    #[tokio::test]
    async fn handle_callback_with_error_param_returns_upstream_error() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .returning(|_| Ok(Some(login_attempt("nonce"))));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let result = service
            .handle_callback(Some("state"), None, Some("access_denied"))
            .await;

        assert_matches!(result, Err(CallbackError::Upstream(ref reason)) if reason == "access_denied");
    }

    #[tokio::test]
    async fn handle_callback_without_code_returns_missing_code() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .returning(|_| Ok(Some(login_attempt("nonce"))));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let result = service.handle_callback(Some("state"), None, None).await;

        assert_matches!(result, Err(CallbackError::MissingCode));
    }

    #[tokio::test]
    async fn handle_callback_with_failing_token_endpoint_returns_token_exchange_failed() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        oidc.mock_token_endpoint_error().await;

        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .returning(|_| Ok(Some(login_attempt("nonce"))));

        let service = oidc.service(repository);

        let result = service.handle_callback(Some("state"), Some("code"), None).await;

        assert_matches!(result, Err(CallbackError::TokenExchangeFailed));
    }

    #[tokio::test]
    async fn handle_callback_with_nonce_mismatch_returns_invalid_nonce() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        oidc.mock_jwks().await;

        let now = Utc::now().timestamp();
        let id_token = sign(&TestIdTokenClaims {
            iss: &oidc.issuer,
            aud: TEST_CLIENT_ID,
            sub: "subject-1",
            name: "Jane Doe",
            exp: now + 300,
            iat: now,
            nonce: "actual-nonce",
        });
        let access_token = sign(&TestAccessTokenClaims {
            iss: &oidc.issuer,
            exp: now + 300,
            iat: now,
            realm_access: TestRealmAccess { roles: vec![] },
        });
        oidc.mock_token_endpoint(id_token, access_token).await;

        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        // The stored login attempt has a different nonce than the one in the id_token.
        repository
            .expect_take_login_attempt()
            .returning(|_| Ok(Some(login_attempt("expected-nonce"))));

        let service = oidc.service(repository);

        let result = service.handle_callback(Some("state"), Some("code"), None).await;

        assert_matches!(
            result,
            Err(CallbackError::TokenVerification(TokenVerificationError::InvalidNonce))
        );
    }

    #[tokio::test]
    async fn handle_callback_creates_session_on_successful_login() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        oidc.mock_jwks().await;

        let nonce = "test-nonce";
        let now = Utc::now().timestamp();
        let id_token = sign(&TestIdTokenClaims {
            iss: &oidc.issuer,
            aud: TEST_CLIENT_ID,
            sub: "subject-1",
            name: "Jane Doe",
            exp: now + 300,
            iat: now,
            nonce,
        });
        let access_token = sign(&TestAccessTokenClaims {
            iss: &oidc.issuer,
            exp: now + 300,
            iat: now,
            realm_access: TestRealmAccess {
                roles: vec!["privilege_admin".to_string(), "offline_access".to_string()],
            },
        });
        oidc.mock_token_endpoint(id_token.clone(), access_token).await;

        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .with(predicate::eq("session-state"))
            .returning(move |_| Ok(Some(login_attempt(nonce))));

        let captured = Arc::new(Mutex::new(None));
        let captured_clone = Arc::clone(&captured);
        repository
            .expect_insert_user_session()
            .returning(move |session_id, session| {
                *captured_clone.lock().unwrap() = Some((session_id, session));
                Ok(())
            });

        let service = oidc.service(repository);

        let session_id = service
            .handle_callback(Some("session-state"), Some("auth-code"), None)
            .await
            .unwrap();

        let (captured_session_id, session) = captured.lock().unwrap().take().unwrap();
        assert_eq!(captured_session_id, session_id);
        assert_eq!(session.display_name, "Jane Doe");
        assert_eq!(
            session.roles,
            vec!["privilege_admin".to_string(), "offline_access".to_string()]
        );
        assert_eq!(session.id_token, id_token);
        assert!(session.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn handle_callback_fails_when_access_token_invalid() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        oidc.mock_jwks().await;

        let nonce = "test-nonce";
        let now = Utc::now().timestamp();
        let id_token = sign(&TestIdTokenClaims {
            iss: &oidc.issuer,
            aud: TEST_CLIENT_ID,
            sub: "subject-1",
            name: "Jane Doe",
            exp: now + 300,
            iat: now,
            nonce,
        });
        // Missing the required `iss` claim, so access_token validation fails even though the
        // signature itself is valid.
        let access_token = sign(&AccessTokenClaimsMissingIssuer {
            exp: now + 300,
            iat: now,
            realm_access: TestRealmAccess {
                roles: vec!["privilege_admin".to_string()],
            },
        });
        oidc.mock_token_endpoint(id_token, access_token).await;

        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_login_attempt()
            .returning(move |_| Ok(Some(login_attempt(nonce))));

        let service = oidc.service(repository);

        service
            .handle_callback(Some("state"), Some("code"), None)
            .await
            .expect_err("login should fail when the access_token cannot be validated");
    }

    #[tokio::test]
    async fn end_session_returns_successfully_for_unknown_session() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository.expect_take_user_session().returning(|_| Ok(None));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        service.end_session("unknown").await.expect("end_session is idempotent");
    }

    #[tokio::test]
    async fn end_session_returns_error_on_storage_error() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_take_user_session()
            .returning(|_| Err(PersistenceError::NoRowsUpdated));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let error = service.end_session("session").await.expect_err("storage error");

        assert_matches!(error, PersistenceError::NoRowsUpdated);
    }

    #[tokio::test]
    async fn end_session_succeeds_for_known_session() {
        let oidc = TestOidc::start().await;
        oidc.mock_metadata().await;
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository.expect_take_user_session().returning(|_| {
            Ok(Some(AdminPortalUserSession {
                display_name: "Jane Doe".to_string(),
                roles: vec![],
                id_token: "the-id-token".to_string(),
                expires_at: Utc::now(),
            }))
        });

        let service = oidc.service(repository);

        service.end_session("session").await.expect("success");
    }

    #[tokio::test]
    async fn authenticated_user_returns_none_for_unknown_session() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository.expect_touch_user_session().returning(|_, _, _| Ok(None));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        assert_matches!(service.authenticated_user("session").await, Ok(None));
    }

    #[tokio::test]
    async fn authenticated_user_propagates_storage_error() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository
            .expect_touch_user_session()
            .returning(|_, _, _| Err(PersistenceError::NoRowsUpdated));

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        assert_matches!(
            service.authenticated_user("session").await,
            Err(PersistenceError::NoRowsUpdated)
        );
    }

    #[tokio::test]
    async fn authenticated_user_reports_display_name_and_stripped_privileges() {
        let mut repository = MockAdminPortalSessionRepository::new();
        expect_cleanup(&mut repository);
        repository.expect_touch_user_session().returning(|_, _, _| {
            Ok(Some(AdminPortalUserSession {
                display_name: "Jane Doe".to_string(),
                roles: vec![
                    "privilege_admin".to_string(),
                    "privilege_support".to_string(),
                    "offline_access".to_string(),
                ],
                id_token: "the-id-token".to_string(),
                expires_at: Utc::now(),
            }))
        });

        let keycloak_url = BaseUrl::from_str("https://keycloak.example.org").unwrap();
        let service = service_with_repository(&keycloak_url, repository);

        let user = service
            .authenticated_user("session")
            .await
            .expect("no errors")
            .expect("session found");

        assert_eq!(user.display_name, "Jane Doe");
        assert_eq!(user.privileges, vec!["admin".to_string(), "support".to_string()]);
    }
}
