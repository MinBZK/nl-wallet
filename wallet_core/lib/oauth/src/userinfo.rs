use http_utils::reqwest::HttpClient;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::ExpectedEncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::error::JweStringDecryptionError;
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

    #[error("received an encrypted userinfo response, but no decrypter was configured")]
    EncryptedResponseWithoutDecrypter,

    #[error("JWE decryption error: {0}")]
    JweDecryption(#[source] JweStringDecryptionError),

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

/// Configuration needed to decrypt an encrypted UserInfo Response. Not every OpenID Provider encrypts (or even
/// signs) its UserInfo Response, so callers that know their provider never does so can simply pass `None` for this
/// and skip the JWE machinery entirely.
#[derive(Debug, Clone, Copy)]
pub struct UserInfoDecryption<'a> {
    pub decrypter: &'a JweDecrypter,
    pub expected_enc_algs: &'a [EncryptionAlgorithm],
}

/// Fetch the raw UserInfo Response body, given a previously obtained `access_token`. The body is returned as-is: it
/// may be a plain JSON object, a signed JWT (JWS) or a signed-and/or-encrypted JWT (JWE), depending on the
/// provider's configuration.
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

/// Request the UserInfo claims for a previously obtained `access_token` (see [`request_token`]) and decode them,
/// handling every combination of signing and encryption that
/// [OpenID Connect Core §5.3.2](https://openid.net/specs/openid-connect-core-1_0.html#UserInfoResponse) allows:
/// a plain JSON response, a signed-only JWS, or a JWE (which may itself wrap either a signed JWS or plain claims).
/// Pass `decryption` when the provider is known to encrypt its response; omit it (`None`) otherwise, in which case
/// an encrypted response results in [`UserInfoError::EncryptedResponseWithoutDecrypter`].
pub async fn request_userinfo<C>(
    http_client: &HttpClient,
    config: &OidcProviderMetadata,
    access_token: &AccessToken,
    client_id: &str,
    decryption: Option<UserInfoDecryption<'_>>,
) -> Result<C, UserInfoError>
where
    C: DeserializeOwned + JwtTyp,
{
    let body = request_userinfo_body(http_client, config, access_token).await?;

    if looks_like_json_object(&body) {
        // Neither signed nor encrypted: the response body already is the claims.
        return serde_json::from_str(&body).map_err(UserInfoError::Json);
    }

    let payload = if looks_like_compact_jwe(&body) {
        let decryption = decryption.ok_or(UserInfoError::EncryptedResponseWithoutDecrypter)?;
        decryption
            .decrypter
            .decrypt_string(
                &body,
                ExpectedEncryptionAlgorithm::Algorithms(decryption.expected_enc_algs),
            )
            .map_err(UserInfoError::JweDecryption)?
    } else {
        body
    };

    if looks_like_json_object(&payload) {
        // Encrypted, but not signed: decryption already yielded the claims directly.
        return serde_json::from_str(&payload).map_err(UserInfoError::Json);
    }

    let jwks_client = HttpJwksClient::new(http_client.clone());
    let jwks_uri = config.oauth_metadata.jwks_uri.clone().ok_or(UserInfoError::NoJwksUri)?;
    let jwks = jwks_client.jwks(jwks_uri).await.map_err(|e| match e {
        JwksError::Http(e) => UserInfoError::Http(e),
    })?;

    let validation = userinfo_jwt_validation(client_id);
    verify_against_keys(&payload, &jwks, validation)
}

fn verify_against_keys<C: DeserializeOwned + JwtTyp>(
    token: &str,
    jwks: &JwkSet,
    validation: JwtValidation,
) -> Result<C, UserInfoError> {
    // using `Header` make the `typ` optional, but it will still be validated against `C::TYP`, if present
    let jwt: UnverifiedJwt<C, HeaderWithKid<Header>> = token.parse()?;

    let (_, claims) = jwt.parse_and_verify_with_jwkset(jwks, validation)?;

    Ok(claims)
}

/// Algorithms accepted when verifying a userinfo JWT issued by the configured OIDC provider.
/// Both the validation call site and this module's tests (the ones that need *a* valid,
/// accepted algorithm rather than specifically testing RS256) read from this constant, so
/// accepting an additional algorithm in the future is a single change that neither the
/// validation logic nor the test suite can silently drift out of sync with.
const ACCEPTED_USERINFO_ALGORITHMS: [Algorithm; 1] = [Algorithm::RS256];

pub fn userinfo_jwt_validation(client_id: impl ToString) -> JwtValidation {
    let mut validation = JwtValidation::default_with_algorithms(ACCEPTED_USERINFO_ALGORITHMS);
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
    use josekit::jwe::JweHeader;
    use josekit::jwe::alg::rsaes::RsaesJweAlgorithm;
    use josekit::jwk::Jwk as JosekitJwk;
    use josekit::jwk::alg::rsa;
    use jwe::algorithm::EncryptionAlgorithm;
    use jwe::algorithm::RsaAlgorithm;
    use jwe::decryption::JweDecrypter;
    use jwe::decryption::JweRsaPrivateKey;
    use jwt::Algorithm;
    use jwt::EncodingKey;
    use jwt::Header;
    use jwt::error::JwtVerifyError;
    use jwt::jwk::Jwk;
    use jwt::jwk::JwkSet;
    use rstest::rstest;
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

    // This value was captured from nl-rdo-max in a local dev environment.
    const TEST_RSA_JWK_JSON: &str = r#"{
      "kty": "RSA",
      "kid": "cc34c0a0-bd5a-4a3c-a50d-a2a7db7643df",
      "alg": "RSA-OAEP",
      "n": "pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGaut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByyFKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk9_B_f1lVrmOKuHjTNHq48w",
      "e": "AQAB",
      "d": "ksDmucdMJXkFGZxiomNHnroOZxe8AmDLDGO1vhs-POa5PZM7mtUPonxwjVmthmpbZzla-kg55OFfO7YcXhg-Hm2OWTKwm73_rLh3JavaHjvBqsVKuorX3V3RYkSro6HyYIzFJ1Ek7sLxbjDRcDOj4ievSX0oN9l-JZhaDYlPlci5uJsoqro_YrE0PRRWVhtGynd-_aWgQv1YzkfZuMD-hJtDi1Im2humOWxA4eZrFs9eG-whXcOvaSwO4sSGbS99ecQZHM2TcdXeAs1PvjVgQ_dKnZlGN3lTWoWfQP55Z7Tgt8Nf1q4ZAKd-NlMe-7iqCFfsnFwXjSiaOa2CRGZn-Q",
      "p": "4A5nU4ahEww7B65yuzmGeCUUi8ikWzv1C81pSyUKvKzu8CX41hp9J6oRaLGesKImYiuVQK47FhZ--wwfpRwHvSxtNU9qXb8ewo-BvadyO1eVrIk4tNV543QlSe7pQAoJGkxCia5rfznAE3InKF4JvIlchyqs0RQ8wx7lULqwnn0",
      "q": "ven83GM6SfrmO-TBHbjTk6JhP_3CMsIvmSdo4KrbQNvp4vHO3w1_0zJ3URkmkYGhz2tgPlfd7v1l2I6QkIh4Bumdj6FyFZEBpxjE4MpfdNVcNINvVj87cLyTRmIcaGxmfylY7QErP8GFA-k4UoH_eQmGKGK44TRzYj5hZYGWIC8",
      "dp": "lmmU_AG5SGxBhJqb8wxfNXDPJjf__i92BgJT2Vp4pskBbr5PGoyV0HbfUQVMnw977RONEurkR6O6gxZUeCclGt4kQlGZ-m0_XSWx13v9t9DIbheAtgVJ2mQyVDvK4m7aRYlEceFh0PsX8vYDS5o1txgPwb3oXkPTtrmbAGMUBpE",
      "dq": "mxRTU3QDyR2EnCv0Nl0TCF90oliJGAHR9HJmBe__EjuCBbwHfcT8OG3hWOv8vpzokQPRl5cQt3NckzX3fs6xlJN4Ai2Hh2zduKFVQ2p-AF2p6Yfahscjtq-GY9cB85NxLy2IXCC0PF--Sq9LOrTE9QV988SJy_yUrAjcZ5MmECk",
      "qi": "ldHXIrEmMZVaNwGzDF9WG8sHj2mOZmQpw9yrjLK9hAsmsNr5LTyqWAqJIYZSwPTYWhY4nu2O0EY9G9uYiqewXfCKw_UngrJt8Xwfq1Zruz0YY869zPN4GiE9-9rzdZB33RBw8kIOquY3MK74FMwCihYx_LiU2YTHkaoJ3ncvtvg"
    }"#;

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

    // This value was captured from nl-rdo-max in a local dev environment.
    static JWS_PAYLOAD: LazyLock<serde_json::Value> = LazyLock::new(|| {
        json!({
            "aud": "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            "bsn": "999991772",
            "iss": "https://localhost:8006",
            "loa_authn": "http://eidas.europa.eu/LoA/substantial",
            "session_id": "oKir-PwoC36a5TxX5vwIIPAU7WXoGVEsTkUwGSAv9ZM",
            "sub": "ff5a4850ab665a3196ec4311d187a24d615d164787b38c89b98f6144855ddcfe"
        })
    });

    fn create_jws(include_kid: bool, algorithm: Algorithm) -> (String, JwkSet) {
        let kid = "key_id";

        let mut header = Header::new(algorithm);
        if include_kid {
            header.kid = Some(kid.to_string());
        }
        let encoding_key = EncodingKey::from_rsa_der(&rsa::RsaKeyPair::generate(2048).unwrap().to_raw_private_key());
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

    /// Generic claims wrapper for tests, with a `typ` check permissive enough to accept the IdPs that omit the `typ`
    /// header entirely (as some do in practice).
    #[derive(serde::Deserialize)]
    #[serde(transparent)]
    struct TestClaims(serde_json::Value);

    impl JwtTyp for TestClaims {
        fn is_valid_typ(_header_typ: Option<&str>) -> Result<(), JwtVerifyError> {
            Ok(())
        }
    }

    #[test]
    fn test_verify_actual_token() {
        let jwt = "eyJhbGciOiJSUzI1NiIsImtpZCI6InhvaVkzek5WMEFoWWp6YjVSbmtKc0xia3BxdGt3d25zcmVUbEZHRFZxM009IiwieDV0Ijo\
                   iZlp4NTAta21TWEtSV3k0YUNnem9JbDl4T004In0.eyJhdWQiOiIzZTU4MDE2ZS1iYzJlLTQwZDUtYjRiMS1hM2UyNWY2MTkzYj\
                   kiLCJic24iOiI5OTk5OTE3NzIiLCJleHAiOjE3ODE2ODk3OTEsImlzcyI6Imh0dHBzOi8vbG9jYWxob3N0OjgwMDYiLCJsb2FfY\
                   XV0aG4iOiJodHRwOi8vZWlkYXMuZXVyb3BhLmV1L0xvQS9zdWJzdGFudGlhbCIsIm5iZiI6MTc4MTY4OTcyMSwic2Vzc2lvbl9p\
                   ZCI6IkV5eXdtRVVMbEZIMkgwRDBIWnBrZUE1RFQxTzFaY1k0eTFKWFlRWUJ2dzgiLCJzdWIiOiJiNTE2MTQ1MDQ5MDY0ZDA4NDA\
                   zM2MxMjY4MjcwNWM2ODVkMTAzOTEwNmY0ZmY4MzlkODgzNjQ3ZDk3NjI1NDEwIn0.0ChyJXXPIDKpernNCtpKMO6ONmi3cSxcBm\
                   mQgJc8KXmAo5jKf2xIQBIovj3O4CCpQiR4LM1gLkwKcpT1KvEb1SycYNl7-DIyOOvlYXnUlh4VmHL73lmGUKewXoY7inl9_1Uir\
                   VopCI8pVsrhxvsun0gDtHgLUkOrcjRYn0TxTRKk6jmbuR3hRxKAbQHx_Oa9SJvnNoWR5D3YrSQW-Z7ijA8jGh5WXraI6yXUF_vi\
                   E1yeX2Hm875c8JOwbIzkIu1gXHBQiUYooYF12NFiINus7HTqlFLvFJAz4XF6sX3q-wf8-qJ-VpcG9XM2IIzNLNtEH1-IAIO46i2\
                   RKNazXnQNTQ"; // obtained from RDO-MAX

        let jwks: JwkSet = serde_json::from_value(json!({
            "keys":[{
                "kty":"RSA",
                "kid":"d/+fA5nlfbnWFPuPYSBkOsayHFVIaTldFnecZ2ZuI2Y=",
                "alg":"RS256",
                "e":"AQAB",
                "n":"2-T8nK09CNJ3L8hP6ukvkZPdqCIdne0So44LbD0jYChpjPnlUBGkSCfogNPjLuLoBwNUcY0UrrXSnpCL-RwxFbJsLojXGp054MxL8iO-l_FJCxK2hx-kOqPDpy-_6vXJTkz9eQuRZ85FOkbfYBxpvl4RXMm_I7I_L4F2vjmMZQA2lpzQUmNtn7mGi7IlaETvzxANj1HtSc9xbjQ1U3vH1nUgi7i2qh5Dx0PKO-jo20SYJ4zGnLvDW66xS5FSGqofsz3bZkrttNdseVjX0fNihWxNWgf_9bFnI89ZCwtXC6sZry4dRBl2x_aQ_SGTPVEMvyJluugdvkf0rwckj90mdw"
            }, {
                "kty":"RSA",
                "n":"2-T8nK09CNJ3L8hP6ukvkZPdqCIdne0So44LbD0jYChpjPnlUBGkSCfogNPjLuLoBwNUcY0UrrXSnpCL-RwxFbJsLojXGp054MxL8iO-l_FJCxK2hx-kOqPDpy-_6vXJTkz9eQuRZ85FOkbfYBxpvl4RXMm_I7I_L4F2vjmMZQA2lpzQUmNtn7mGi7IlaETvzxANj1HtSc9xbjQ1U3vH1nUgi7i2qh5Dx0PKO-jo20SYJ4zGnLvDW66xS5FSGqofsz3bZkrttNdseVjX0fNihWxNWgf_9bFnI89ZCwtXC6sZry4dRBl2x_aQ_SGTPVEMvyJluugdvkf0rwckj90mdw",
                "e":"AQAB",
                "kid":"d/+fA5nlfbnWFPuPYSBkOsayHFVIaTldFnecZ2ZuI2Y=",
                "x5t":"R0TM80khwVsJg4vTl8-2bds4HE4",
                "alg":"RS256"
            }, {
                "kty":"RSA",
                "n":"vNXjISjuPyVynDhDO9cqfRsfqehHzxOGGzBlmrfUWWJmiKzXaPGkiBjUwtZnlfIqRk-mw8ddhcZcAye8VbIMl4kvVGx4vERSowSIeSXO8CHJyLt6-zCJeJhsPI6PJDwl4p43sf-jSuLmuVJAPSdRRhl4Wxkb-nGrYY3kGR8bjAdlUkS1f6aTnHXc3vpvsONK5Dr3BBjbEzislLU6W-117bMdkUARX3ogqNOs-Hs1SHAMYTLUUCUFrIe8qJurNLqx6D4s_2MHxOAKb5y9Y1U8cR97r6yq8I_zFZ4qvBb9TtOQEIK_F5bhDCoYt-BAPd7pT31iZhZkwHfQeZjSvw7kUQ",
                "e":"AQAB",
                "kid":"TPLJl1kcXIKcx/hNT3c6nnVjt3H6KskGJcSxixq9w4A=",
                "x5t":"v66Q0fKOQ95P7sbUfk_YbU_NX_I",
                "alg":"RS256"
            }, {
                "kty":"RSA",
                "n":"22LzGPlkPTqRzhCj9y8fSz-RibqYj81NQ-wcVlrD1BAynT9SMfCEfADAy0JdIeFqrStlwb_-H3x8e9nasfQt95LsH51jxtt-plyJLIKe0bbvEet4N2FOfRzt-vvK8OV456YhXGZZwNEh0JpNjui7QbAWYB17pkqz1_g0eTpgAoSdktdsUU5tXxufUqOuveGQ6RyrNWCIl6f3uoEXkv4zv8hiEPauCA9aYl2El8w9ojBVc3CYsDVP8HHqXbUj6nOM8t6VMQ-A1rthT1Az6oNSWKLHG3W1kneaTw7VoCr8ek3aXldDUHgb5-2ASc6liv2p067roWUU_jG3Vy2djHxSBQ",
                "e":"AQAB",
                "kid":"xoiY3zNV0AhYjzb5RnkJsLbkpqtkwwnsreTlFGDVq3M=",
                "x5t":"fZx50-kmSXKRWy4aCgzoIl9xOM8",
                "alg":"RS256"
            }]
        })).unwrap();

        let mut validation = userinfo_jwt_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9");
        validation.dont_validate_exp();
        let payload: TestClaims = verify_test_keys(jwt, &jwks, validation).unwrap();

        assert_eq!(
            payload
                .0
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
        );
    }

    #[test]
    fn test_verify_against_keys_success() {
        let (jws, jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);

        let validation = userinfo_jwt_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9");
        let payload =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect("verifying JWS should succeed");

        assert_eq!(
            payload
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
        );
    }

    #[test]
    fn test_verify_against_keys_error_missing_key_id() {
        let (jws, jwks) = create_jws(false, ACCEPTED_USERINFO_ALGORITHMS[0]);

        let validation = userinfo_jwt_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9");
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
        let (jws, mut jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);

        jwks.keys.first_mut().unwrap().common.key_id = Some("wrong_kid".to_string());

        let validation = userinfo_jwt_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(JwtVerifyError::KeyNotFound(_)));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_aud() {
        let (jws, jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);

        let validation = userinfo_jwt_validation("wrong_aud");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(_));
    }

    #[rstest]
    fn test_verify_against_keys_error_wrong_alg(
        // anything other than RS256 should fail, non-RSA aren't compatible with the generated RSA key
        #[values(
            Algorithm::RS384,
            Algorithm::RS512,
            Algorithm::PS256,
            Algorithm::PS384,
            Algorithm::PS512
        )]
        algorithm: Algorithm,
    ) {
        let (jws, jwks) = create_jws(true, algorithm);

        let validation = userinfo_jwt_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9");
        let error =
            verify_test_keys::<serde_json::Value>(&jws, &jwks, validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(_));
    }

    fn create_test_decrypter() -> JweDecrypter {
        let jwk: jwk_simple::Key = serde_json::from_str(TEST_RSA_JWK_JSON).unwrap();
        let private_key = JweRsaPrivateKey::try_from_jwk(&jwk, RsaAlgorithm::RsaOaep).unwrap();
        JweDecrypter::from_rsa_private_key(&private_key)
    }

    fn create_test_jwe(jws: &str) -> String {
        let josekit_jwk = JosekitJwk::from_bytes(TEST_RSA_JWK_JSON.as_bytes()).unwrap();
        let encrypter = RsaesJweAlgorithm::RsaOaep.encrypter_from_jwk(&josekit_jwk).unwrap();

        let mut header = JweHeader::new();
        header.set_content_encryption(EncryptionAlgorithm::A128CbcHs256.to_string());
        header.set_key_id("cc34c0a0-bd5a-4a3c-a50d-a2a7db7643df");

        josekit::jwe::serialize_compact(jws.as_bytes(), &header, &encrypter).unwrap()
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
    async fn request_userinfo_signed_and_encrypted_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let (jws, jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);
        let jwe = create_test_jwe(&jws);

        let _token_mock = mock_token_endpoint(&server).await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).header("content-type", "application/jwt").body(jwe);
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

        let decrypter = create_test_decrypter();
        let decryption = UserInfoDecryption {
            decrypter: &decrypter,
            expected_enc_algs: &[EncryptionAlgorithm::A128CbcHs256],
        };

        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result = request_userinfo::<serde_json::Value>(
            &http_client,
            &metadata,
            &token_response.access_token,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Some(decryption),
        )
        .await;

        assert_eq!(
            result
                .unwrap()
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
        );
    }

    #[tokio::test]
    async fn request_userinfo_signed_only_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let (jws, jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);

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

        // No `decryption` is configured: this provider never encrypts its UserInfo Response.
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result = request_userinfo::<serde_json::Value>(
            &http_client,
            &metadata,
            &token_response.access_token,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            None,
        )
        .await;

        assert_eq!(
            result
                .unwrap()
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
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

        // Neither a `decryption` config nor a JWKS mock is needed: this provider returns unsigned, unencrypted
        // claims directly, so no JWE or JWS machinery (and thus no JWKS lookup) is ever exercised.
        let http_client = HttpClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let token_response = request_token::<AuthorizationCodeGrantType, TokenResponse, TestTokenErrorCode>(
            &http_client,
            &metadata,
            create_token_request(),
        )
        .await
        .unwrap();
        let result = request_userinfo::<serde_json::Value>(
            &http_client,
            &metadata,
            &token_response.access_token,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            None,
        )
        .await;

        assert_eq!(
            result
                .unwrap()
                .as_object()
                .and_then(|payload| payload.get("bsn"))
                .and_then(serde_json::Value::as_str),
            Some("999991772")
        );
    }

    #[tokio::test]
    async fn request_userinfo_encrypted_without_decrypter_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let (jws, _jwks) = create_jws(true, ACCEPTED_USERINFO_ALGORITHMS[0]);
        let jwe = create_test_jwe(&jws);

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
        let result = request_userinfo::<serde_json::Value>(
            &http_client,
            &metadata,
            &token_response.access_token,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            None,
        )
        .await;

        assert_matches!(result, Err(UserInfoError::EncryptedResponseWithoutDecrypter));
    }
}
