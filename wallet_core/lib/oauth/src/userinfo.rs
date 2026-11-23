use http_utils::reqwest::HttpClient;
use jwt::Algorithm;
use jwt::Header;
use jwt::JwtTyp;
use jwt::JwtValidation;
use jwt::UnverifiedJwt;
use jwt::error::JwtParseError;
use jwt::error::JwtVerifyError;
use jwt::headers::HeaderWithKid;
use jwt::jwk::JwkSet;
use reqwest::header;
use serde::de::DeserializeOwned;

use crate::errors::RemoteErrorResponse;
use crate::jwks::HttpJwksClient;
use crate::jwks::JwksError;
use crate::metadata::oauth_metadata::OidcProviderMetadata;
use crate::token::AccessToken;

/// Per [OpenID Connect Core §5.3.2](https://openid.net/specs/openid-connect-core-1_0.html#UserInfoResponse), the
/// UserInfo Response is either a plain JSON object (if the client did not request a signed and/or encrypted
/// response), or a JWT. Advertise that we accept both, and let the actual response tell us which one we got instead
/// of assuming a specific profile.
const USERINFO_ACCEPT: &str = "application/jwt, application/json";

#[derive(Debug, thiserror::Error)]
pub enum UserInfoError {
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("config has no JWKS URI")]
    NoJwksUri,

    #[error("error requesting userinfo: {0:?}")]
    RequestingUserInfo(Box<RemoteErrorResponse<AuthBearerErrorCode>>),

    #[error("received an encrypted userinfo response")]
    EncryptedResponse,

    #[error("error parsing userinfo JSON response: {0}")]
    Json(#[source] serde_json::Error),

    #[error("error parsing JWT: {0}")]
    JwtParse(#[from] JwtParseError),

    #[error("error verifying JWT: {0}")]
    JwtVerify(#[from] JwtVerifyError),

    #[error("config has no userinfo url")]
    NoUserinfoUrl,
}

/// Source: <https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1>
///
/// This type should be used wrapped in [`crate::errors::RemoteErrorCode`] (possibly via [`RemoteErrorResponse`]), which
/// provides a fallback for error codes the holder is not aware of.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthBearerErrorCode {
    InvalidRequest,
    InvalidToken,
    InsufficientScope,
}

/// Fetch the raw UserInfo Response body, given a previously obtained `access_token`. The body is returned as-is: it
/// may be a plain JSON object, a signed JWT (JWS), depending on the provider's configuration.
async fn request_userinfo_body(
    http_client: &HttpClient,
    config: &OidcProviderMetadata,
    access_token: &AccessToken,
) -> Result<String, UserInfoError> {
    // Get userinfo endpoint from discovery, throw an error otherwise.
    let endpoint = config
        .oidc_metadata_extension
        .userinfo_endpoint
        .clone()
        .ok_or(UserInfoError::NoUserinfoUrl)?;

    let response = http_client
        .post(endpoint, |request| {
            request
                .header(header::ACCEPT, USERINFO_ACCEPT)
                .bearer_auth(access_token.as_ref())
        })
        .await?;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let error = response.json::<RemoteErrorResponse<AuthBearerErrorCode>>().await?;
        return Err(UserInfoError::RequestingUserInfo(error.into()));
    }

    Ok(response.text().await?)
}

/// A JSON object always starts with `{`, whereas none of the compact serializations of a JWS or JWE can, since those
/// consist exclusively of base64url characters and `.` separators. This lets us tell the two apart without relying
/// on a (possibly absent or incorrect) `Content-Type` header.
fn looks_like_json_object(body: &str) -> bool {
    body.trim_start().starts_with('{')
}

/// The compact serialization of a JWE has 5 dot-separated parts, versus 3 for a JWS.
fn looks_like_compact_jwe(token: &str) -> bool {
    token.trim().matches('.').count() == 4
}

/// Request the UserInfo claims for a previously obtained `access_token` (see [`request_token`]) and decode them.
///
/// [OpenID Connect Core §5.3.2](https://openid.net/specs/openid-connect-core-1_0.html#UserInfoResponse) allows:
/// a plain JSON response, a signed-only JWS, or a JWE (which may itself wrap either a signed JWS or plain claims).
/// Only plain JSON or signed-only JWS responses are supported.
pub async fn request_userinfo<C>(
    http_client: &HttpClient,
    config: &OidcProviderMetadata,
    access_token: &AccessToken,
    client_id: &str,
) -> Result<C, UserInfoError>
where
    C: DeserializeOwned + JwtTyp,
{
    let body = request_userinfo_body(http_client, config, access_token).await?;

    if looks_like_json_object(&body) {
        // Neither signed nor encrypted: the response body already is the claims.
        return serde_json::from_str(&body).map_err(UserInfoError::Json);
    }

    if looks_like_compact_jwe(&body) {
        Err(UserInfoError::EncryptedResponse)?
    }

    let jwks_client = HttpJwksClient::new(http_client.clone());
    let jwks_uri = config.oauth_metadata.jwks_uri.clone().ok_or(UserInfoError::NoJwksUri)?;
    let jwks = jwks_client.jwks(jwks_uri).await.map_err(|e| match e {
        JwksError::Http(e) => UserInfoError::Http(e),
    })?;

    let validation = userinfo_jwt_validation(client_id);
    verify_against_keys(&body, &jwks, validation)
}

fn verify_against_keys<C: DeserializeOwned + JwtTyp>(
    token: &str,
    jwks: &JwkSet,
    validation: JwtValidation,
) -> Result<C, UserInfoError> {
    // using `Header` make the `typ` optional, but it will still be validated against `C::TYP`, if present
    let jwt: UnverifiedJwt<C, HeaderWithKid<Header>> = token.parse()?;

    let (_, claims) = jwt.parse_and_verify_by_kid(jwks, validation)?;

    Ok(claims)
}

pub fn userinfo_jwt_validation(client_id: impl ToString) -> JwtValidation {
    let mut validation = JwtValidation::default_with_algorithms([Algorithm::ES256]);
    validation.require_aud(client_id);
    validation
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::sync::LazyLock;

    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpClient;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use jwt::Algorithm;
    use jwt::EncodingKey;
    use jwt::Header;
    use jwt::error::JwtVerifyError;
    use jwt::jwk::Jwk;
    use jwt::jwk::JwkSet;
    use p256::elliptic_curve::Generate;
    use p256::pkcs8::EncodePrivateKey;
    use serde_json::json;
    use url::Url;

    use super::AuthBearerErrorCode;
    use super::*;
    use crate::errors::ErrorResponse;
    use crate::issuer_identifier::IssuerIdentifier;
    use crate::metadata::oauth_metadata::AuthorizationServerMetadata;
    use crate::metadata::oauth_metadata::OidcProviderMetadata;
    use crate::metadata::oauth_metadata::OpenIdMetadataExtension;
    use crate::token::AccessToken;
    use crate::token::AuthorizationCode;
    use crate::token::AuthorizationCodeGrantType;
    use crate::token::TokenRequest;
    use crate::token::TokenResponse;
    use crate::token::TokenType;
    use crate::token::request_token;

    // Test-only stand-in for a token endpoint's error code, to avoid depending on any concrete profile's error codes.
    #[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    enum TestTokenErrorCode {
        InvalidRequest,
    }

    fn create_token_request() -> TokenRequest<AuthorizationCodeGrantType> {
        TokenRequest {
            grant_type: AuthorizationCodeGrantType::AuthorizationCode {
                code: AuthorizationCode::from("test-code".to_string()),
            },
            client_id: None,
            redirect_uri: Some("https://example.com/callback".parse::<Url>().unwrap()),
            scope: None,
            code_verifier: Some("test-verifier".to_string()),
        }
    }

    fn create_metadata(server: &MockServer) -> OidcProviderMetadata {
        let issuer_identifier: IssuerIdentifier = server.base_url().parse().unwrap();
        OidcProviderMetadata::new_mock(issuer_identifier)
    }

    async fn request_test_userinfo_body(
        http_client: &HttpClient,
        config: &OidcProviderMetadata,
        access_token: &AccessToken,
    ) -> Result<String, UserInfoError> {
        request_userinfo_body(http_client, config, access_token).await
    }

    #[tokio::test]
    async fn request_userinfo_body_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let access_token = AccessToken::from("test-access-token".to_string());

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).body("the.userinfo.jwt");
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_test_userinfo_body(&http_client, &metadata, &access_token).await;

        assert_eq!(result.unwrap(), "the.userinfo.jwt");
    }

    #[tokio::test]
    async fn request_userinfo_body_userinfo_endpoint_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let access_token = AccessToken::from("test-access-token".to_string());
        let error_response = ErrorResponse {
            error: AuthBearerErrorCode::InvalidToken,
            error_description: Some("token expired".to_string()),
            error_uri: None,
        };

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/userinfo");
                then.status(401)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&error_response).unwrap());
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_test_userinfo_body(&http_client, &metadata, &access_token).await;

        assert_matches!(result, Err(UserInfoError::RequestingUserInfo(_)));
    }

    #[tokio::test]
    async fn request_userinfo_body_no_userinfo_url() {
        let issuer_identifier: IssuerIdentifier = "https://example.com".parse().unwrap();
        let token_endpoint = issuer_identifier.as_base_url().as_ref().join("/token").unwrap();
        let metadata = OidcProviderMetadata::new(
            AuthorizationServerMetadata::new(issuer_identifier, token_endpoint),
            OpenIdMetadataExtension::default(),
        );
        let access_token = AccessToken::from("test-access-token".to_string());

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_test_userinfo_body(&http_client, &metadata, &access_token).await;

        assert_matches!(result, Err(UserInfoError::NoUserinfoUrl));
    }

    static JWS_PAYLOAD: LazyLock<serde_json::Value> = LazyLock::new(|| {
        json!({
            "aud": "test",
            "sub": "frouke",
            "iss": "https://localhost:8080",
        })
    });

    fn create_jws(include_kid: bool, algorithm: Algorithm) -> (String, JwkSet) {
        let kid = "key_id";

        let mut header = Header::new(algorithm);
        if include_kid {
            header.kid = Some(kid.to_string());
        }

        let signing_key = p256::ecdsa::SigningKey::generate();
        let encoding_key = EncodingKey::from_ec_der(signing_key.to_pkcs8_der().unwrap().as_bytes());
        let jws = jsonwebtoken::encode(&header, LazyLock::force(&JWS_PAYLOAD), &encoding_key).unwrap();

        let mut jwk = Jwk::from_encoding_key(&encoding_key, algorithm).unwrap();
        jwk.common.key_id = Some(kid.to_string());
        let jwks = JwkSet { keys: vec![jwk] };

        (jws, jwks)
    }

    fn verify_test_keys<C: DeserializeOwned + JwtTyp>(
        token: &str,
        jwks: &JwkSet,
        validation: JwtValidation,
    ) -> Result<C, UserInfoError> {
        verify_against_keys(token, jwks, validation)
    }

    #[test]
    fn test_verify_against_keys_success() {
        let (jws, jwks) = create_jws(true, Algorithm::ES256);

        let validation = userinfo_jwt_validation("test");
        let payload =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect("verifying JWS should succeed");

        assert_eq!(
            payload
                .as_object()
                .and_then(|payload| payload.get("sub"))
                .and_then(serde_json::Value::as_str),
            Some("frouke")
        );
    }

    #[test]
    fn test_verify_against_keys_error_missing_key_id() {
        let (jws, jwks) = create_jws(false, Algorithm::ES256);

        let validation = userinfo_jwt_validation("test");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(
            error,
            UserInfoError::JwtVerify(JwtVerifyError::ParseError(JwtParseError::JsonParsing(parse_error)))
                if parse_error.to_string().contains("missing field `kid`")
        );
    }

    #[test]
    fn test_verify_against_keys_error_key_not_found() {
        let (jws, mut jwks) = create_jws(true, Algorithm::ES256);

        jwks.keys.first_mut().unwrap().common.key_id = Some("wrong_kid".to_string());

        let validation = userinfo_jwt_validation("test");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(JwtVerifyError::KeyNotFound(_)));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_aud() {
        let (jws, jwks) = create_jws(true, Algorithm::ES256);

        let validation = userinfo_jwt_validation("wrong_aud");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(_));
    }

    async fn mock_token_endpoint(server: &MockServer) -> httpmock::Mock<'_> {
        let token_response = TokenResponse::new(AccessToken::from("test-access-token".to_string()), TokenType::Bearer);

        server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&token_response).unwrap());
            })
            .await
    }

    #[tokio::test]
    async fn request_userinfo_signed_only_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let (jws, jwks) = create_jws(true, Algorithm::ES256);

        let _token_mock = mock_token_endpoint(&server).await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).header("content-type", "application/jwt").body(jws);
            })
            .await;

        let _jwks_mock = server
            .mock_async(|when, then| {
                when.method(GET).path("/jwks.json");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&jwks).unwrap());
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result =
            request_userinfo::<serde_json::Value>(&http_client, &metadata, &token_response.access_token, "test").await;

        assert_eq!(
            result
                .unwrap()
                .as_object()
                .and_then(|payload| payload.get("sub"))
                .and_then(serde_json::Value::as_str),
            Some("frouke")
        );
    }

    #[tokio::test]
    async fn request_userinfo_plain_json_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);

        let _token_mock = mock_token_endpoint(&server).await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(LazyLock::force(&JWS_PAYLOAD).clone());
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result =
            request_userinfo::<serde_json::Value>(&http_client, &metadata, &token_response.access_token, "test").await;

        assert_eq!(
            result
                .unwrap()
                .as_object()
                .and_then(|payload| payload.get("sub"))
                .and_then(serde_json::Value::as_str),
            Some("frouke")
        );
    }

    #[tokio::test]
    async fn request_userinfo_encrypted_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let jwe = "eyJhbGciOiJFQ0RILUVTIiwiZW5jIjoiQTI1NkdDTSIsImVwayI6eyJ4IjoidVBzT3dBdWtpMWpiV05Cb3N5NHo1Y05IOTRWMGczUXh5R3FDT1gwcjdzayIsImNydiI6IlAtMjU2Iiwia3R5IjoiRUMiLCJ5IjoicTVYQm9MclkwcEZ5X2p3dFBhbjI5NFU3dDg1RklON0hRZTduRGQ5Y1dldyJ9fQ..Lvpy5asLB0zKUl4C.8hAaCeDPMZgqJSKKkGM3O5YS3QOM2JSmuBrwsacwj367obc7r_cauSXEU2SVh0oY-SEWYK31uONnfrx8OPHXSmxiJoA3XIpqfB6bzj7BY6Bn3R2RRhY2Q7WTRnGsXj0SmuEJKwJXziyw3HnT2KGIpvp0hGdIF0pB3KBPdTbyr67mNthxcdNmZ9HtUxQbf1ePvpfAXPdYa8GlHQ9kDaEBWWFmvks79b2rlxWoR0c7OO8.w0WV3uvDQAQfdTz9qZ_b5w";

        let _token_mock = mock_token_endpoint(&server).await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).header("content-type", "application/jwt").body(jwe);
            })
            .await;

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result =
            request_userinfo::<serde_json::Value>(&http_client, &metadata, &token_response.access_token, "test").await;

        assert_matches!(result, Err(UserInfoError::EncryptedResponse));
    }
}
