use futures::TryFutureExt;
use futures::try_join;
use http_utils::reqwest::HttpJsonClient;
use jwe::algorithm::EncryptionAlgorithm;
use jwe::decryption::ExpectedEncryptionAlgorithm;
use jwe::decryption::JweDecrypter;
use jwe::error::JweStringDecryptionError;
use jwt::Algorithm;
use jwt::Header;
use jwt::JwtTyp;
use jwt::UnverifiedJwt;
use jwt::Validation;
use jwt::error::JwtParseError;
use jwt::error::JwtVerifyError;
use jwt::headers::HeaderWithKid;
use jwt::jwk::JwkSet;
use openid4vc::errors::RemoteErrorResponse;
use openid4vc::errors::TokenErrorCode;
use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenResponse;
use reqwest::header;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use strum::EnumString;

use super::jwks::HttpJwksClient;
use super::jwks::JwksError;

const APPLICATION_JWT: &str = "application/jwt";

#[derive(Debug, thiserror::Error)]
pub enum UserInfoError {
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("config has no JWKS URI")]
    NoJwksUri,

    #[error("error requesting access token: {0:?}")]
    RequestingAccessToken(Box<RemoteErrorResponse<TokenErrorCode>>),

    #[error("error requesting userinfo: {0:?}")]
    RequestingUserInfo(Box<RemoteErrorResponse<AuthBearerErrorCode>>),

    #[error("JWE decryption error: {0}")]
    JweDecryption(#[source] JweStringDecryptionError),

    #[error("error parsing JWT: {0}")]
    JwtParse(#[from] JwtParseError),

    #[error("error verifying JWT: {0}")]
    JwtVerify(#[from] JwtVerifyError),

    #[error("config has no userinfo url")]
    NoUserinfoUrl,
}

/// Source: <https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1>
#[derive(Debug, Clone, PartialEq, Eq, strum::Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AuthBearerErrorCode {
    InvalidRequest,
    InvalidToken,
    InsufficientScope,

    // Catch-all variant, in case the server sends an error code that the holder is not aware of.
    #[strum(default)]
    Other(String),
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub bsn: String,
}

impl JwtTyp for UserInfo {
    fn is_valid_typ(_header_typ: Option<&str>) -> Result<(), JwtVerifyError> {
        // no `typ` field is set for JWTs provided by RDO-MAX
        Ok(())
    }
}

async fn request_userinfo_jwt(
    http_client: &HttpJsonClient,
    config: &OidcProviderMetadata,
    token_request: TokenRequest,
) -> Result<String, UserInfoError> {
    // Get userinfo endpoint from discovery, throw an error otherwise.
    let endpoint = config
        .as_ref()
        .userinfo_endpoint
        .clone()
        .ok_or(UserInfoError::NoUserinfoUrl)?;

    let response = http_client
        .post(config.as_ref().token_endpoint.clone(), |request| {
            request.form(&token_request)
        })
        .await?;

    let token_response = {
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = response.json::<RemoteErrorResponse<TokenErrorCode>>().await?;
            return Err(UserInfoError::RequestingAccessToken(error.into()));
        } else {
            response.json::<TokenResponse>().await?
        }
    };

    // Use the access_token to retrieve the userinfo as a JWT.
    let response = http_client
        .post(endpoint, |request| {
            request
                .header(header::ACCEPT, APPLICATION_JWT)
                .bearer_auth(token_response.access_token.as_ref())
        })
        .await?;

    let jwt = {
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = response.json::<RemoteErrorResponse<AuthBearerErrorCode>>().await?;
            return Err(UserInfoError::RequestingUserInfo(error.into()));
        } else {
            response.text().await?
        }
    };

    Ok(jwt)
}

pub async fn request_userinfo<C>(
    http_client: &HttpJsonClient,
    config: &OidcProviderMetadata,
    token_request: TokenRequest,
    client_id: &str,
    decrypter: &JweDecrypter,
    (expected_jws_alg, expected_enc_alg): (Algorithm, EncryptionAlgorithm),
) -> Result<C, UserInfoError>
where
    C: DeserializeOwned + JwtTyp,
{
    let jwks_client = HttpJwksClient::new(http_client.clone());
    let jwks_uri = config.as_ref().jwks_uri.clone().ok_or(UserInfoError::NoJwksUri)?;

    let (jwe, jwks) = try_join!(
        request_userinfo_jwt(http_client, config, token_request),
        jwks_client.jwks(jwks_uri).map_err(|e| match e {
            JwksError::Http(e) => UserInfoError::Http(e),
        })
    )?;

    let jws = decrypter
        .decrypt_string(&jwe, ExpectedEncryptionAlgorithm::Algorithms(&[expected_enc_alg]))
        .map_err(UserInfoError::JweDecryption)?;

    let validation = userinfo_validation(client_id, expected_jws_alg);
    verify_against_keys(&jws, &jwks, &validation)
}

// We can't use our own `Jwt` types here because they only support ECDSA/P256.
fn verify_against_keys<C: DeserializeOwned + JwtTyp>(
    token: &str,
    jwks: &JwkSet,
    validation: &Validation,
) -> Result<C, UserInfoError> {
    // using `Header` make the `typ` optional, but it will still be validated against `C::TYP`, if present
    let jwt: UnverifiedJwt<C, HeaderWithKid<Header>> = token.parse()?;

    let (_, claims) = jwt.parse_and_verify_with_jwkset(jwks, validation)?;

    Ok(claims)
}

fn userinfo_validation(client_id: &str, expected_jws_alg: Algorithm) -> Validation {
    let mut validation = Validation::new(expected_jws_alg);
    validation.required_spec_claims.clear(); // don't require exp
    validation.set_audience(&[client_id]);
    validation
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::sync::LazyLock;

    use http_utils::httpmock::httpmock_reqwest_client_builder;
    use http_utils::reqwest::HttpJsonClient;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use josekit::jwe::JweHeader;
    use josekit::jwe::alg::rsaes::RsaesJweAlgorithm;
    use josekit::jwk::Jwk as JosekitJwk;
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
    use openid4vc::errors::ErrorResponse;
    use openid4vc::errors::TokenErrorCode;
    use openid4vc::issuer_identifier::IssuerIdentifier;
    use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
    use openid4vc::metadata::oauth_metadata::OidcProviderMetadata;
    use openid4vc::token::AccessToken;
    use openid4vc::token::AuthorizationCode;
    use openid4vc::token::TokenRequest;
    use openid4vc::token::TokenResponse;
    use serde_json::json;
    use url::Url;

    use super::AuthBearerErrorCode;
    use super::*;

    fn create_token_request() -> TokenRequest {
        TokenRequest::new_authorization_code(
            AuthorizationCode::from("test-code".to_string()),
            "https://example.com/callback".parse::<Url>().unwrap(),
            "test-verifier".to_string(),
        )
    }

    fn create_metadata(server: &MockServer) -> OidcProviderMetadata {
        let issuer_identifier: IssuerIdentifier = server.base_url().parse().unwrap();
        OidcProviderMetadata::new(AuthorizationServerMetadata::new_mock(issuer_identifier))
    }

    #[tokio::test]
    async fn request_userinfo_jwt_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let token_response = TokenResponse::new(AccessToken::from("test-access-token".to_string()));

        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&token_response).unwrap());
            })
            .await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).body("the.userinfo.jwt");
            })
            .await;

        let http_client = HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_userinfo_jwt(&http_client, &metadata, create_token_request()).await;

        assert_eq!(result.unwrap(), "the.userinfo.jwt");
    }

    #[tokio::test]
    async fn request_userinfo_jwt_token_endpoint_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let error_response = ErrorResponse {
            error: TokenErrorCode::InvalidRequest,
            error_description: Some("invalid code".to_string()),
            error_uri: None,
        };

        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(400)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&error_response).unwrap());
            })
            .await;

        let http_client = HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_userinfo_jwt(&http_client, &metadata, create_token_request()).await;

        assert_matches!(result, Err(UserInfoError::RequestingAccessToken(_)));
    }

    #[tokio::test]
    async fn request_userinfo_jwt_userinfo_endpoint_error() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let token_response = TokenResponse::new(AccessToken::from("test-access-token".to_string()));
        let error_response = ErrorResponse {
            error: AuthBearerErrorCode::InvalidToken,
            error_description: Some("token expired".to_string()),
            error_uri: None,
        };

        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&token_response).unwrap());
            })
            .await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/userinfo");
                then.status(401)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&error_response).unwrap());
            })
            .await;

        let http_client = HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_userinfo_jwt(&http_client, &metadata, create_token_request()).await;

        assert_matches!(result, Err(UserInfoError::RequestingUserInfo(_)));
    }

    #[tokio::test]
    async fn request_userinfo_jwt_no_userinfo_url() {
        let server = MockServer::start_async().await;
        let issuer_identifier: IssuerIdentifier = server.base_url().parse().unwrap();
        let token_endpoint = issuer_identifier.as_base_url().as_ref().join("/token").unwrap();
        let metadata = OidcProviderMetadata::new(AuthorizationServerMetadata::new(issuer_identifier, token_endpoint));

        let http_client = HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_userinfo_jwt(&http_client, &metadata, create_token_request()).await;

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

    fn create_jws(include_kid: bool) -> (String, JwkSet) {
        let algoritm = Algorithm::HS256;
        let kid = "hmac_key_id";

        let mut header = Header::new(algoritm);
        if include_kid {
            header.kid = Some(kid.to_string());
        }
        let encoding_key = EncodingKey::from_secret(b"secret hmac key");
        let jws = jsonwebtoken::encode(&header, LazyLock::force(&JWS_PAYLOAD), &encoding_key).unwrap();

        let mut jwk = Jwk::from_encoding_key(&encoding_key, algoritm).unwrap();
        jwk.common.key_id = Some(kid.to_string());
        let jwks = JwkSet { keys: vec![jwk] };

        (jws, jwks)
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

        let mut validation = userinfo_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9", Algorithm::RS256);
        validation.validate_exp = false; // we have no way to set the clock, so skip exp validation
        let payload: UserInfo = verify_against_keys(jwt, &jwks, &validation).unwrap();

        assert_eq!(payload.bsn, "999991772".to_owned());
    }

    #[test]
    fn test_verify_against_keys_success() {
        let (jws, jwks) = create_jws(true);

        let validation = userinfo_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9", Algorithm::HS256);
        let payload =
            verify_against_keys::<serde_json::Value>(&jws, &jwks, &validation).expect("verifying JWS should succeed");

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
        let (jws, jwks) = create_jws(false);

        let validation = userinfo_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9", Algorithm::HS256);
        let error =
            verify_against_keys::<serde_json::Value>(&jws, &jwks, &validation).expect_err("verifying JWS should fail");

        assert_matches!(
            error,
            UserInfoError::JwtVerify(JwtVerifyError::ParseError(JwtParseError::JsonParsing(parse_error)))
                if parse_error.to_string().contains("missing field `kid`")
        );
    }

    #[test]
    fn test_verify_against_keys_error_key_not_found() {
        let (jws, mut jwks) = create_jws(true);

        jwks.keys.first_mut().unwrap().common.key_id = Some("wrong_kid".to_string());

        let validation = userinfo_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9", Algorithm::HS256);
        let error =
            verify_against_keys::<serde_json::Value>(&jws, &jwks, &validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(JwtVerifyError::KeyNotFound(_)));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_aud() {
        let (jws, jwks) = create_jws(true);

        let validation = userinfo_validation("wrong_aud", Algorithm::HS256);
        let error =
            verify_against_keys::<serde_json::Value>(&jws, &jwks, &validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(_));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_alg() {
        let (jws, jwks) = create_jws(true);

        let validation = userinfo_validation("3e58016e-bc2e-40d5-b4b1-a3e25f6193b9", Algorithm::HS512);
        let error =
            verify_against_keys::<serde_json::Value>(&jws, &jwks, &validation).expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::JwtVerify(_));
    }

    fn create_test_decrypter() -> JweDecrypter {
        let jwk: jwk_simple::Key = serde_json::from_str(crate::pid::digid::TEST_RSA_JWK_JSON).unwrap();
        let private_key = JweRsaPrivateKey::try_from_jwk(&jwk, RsaAlgorithm::RsaOaep).unwrap();
        JweDecrypter::from_rsa_private_key(&private_key)
    }

    fn create_test_jwe(jws: &str) -> String {
        let josekit_jwk = JosekitJwk::from_bytes(crate::pid::digid::TEST_RSA_JWK_JSON.as_bytes()).unwrap();
        let encrypter = RsaesJweAlgorithm::RsaOaep.encrypter_from_jwk(&josekit_jwk).unwrap();

        let mut header = JweHeader::new();
        header.set_content_encryption(EncryptionAlgorithm::A128CbcHs256.to_string());
        header.set_key_id(crate::pid::digid::TEST_RSA_KEY_ID);

        josekit::jwe::serialize_compact(jws.as_bytes(), &header, &encrypter).unwrap()
    }

    #[tokio::test]
    async fn request_userinfo_happy_path() {
        let server = MockServer::start_async().await;
        let metadata = create_metadata(&server);
        let (jws, jwks) = create_jws(true);
        let jwe = create_test_jwe(&jws);
        let token_response = TokenResponse::new(AccessToken::from("test-access-token".to_string()));

        let _token_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/issuance/token");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::to_value(&token_response).unwrap());
            })
            .await;

        let _userinfo_mock = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/userinfo")
                    .header("Authorization", "Bearer test-access-token");
                then.status(200).body(jwe);
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

        let http_client = HttpJsonClient::try_new(httpmock_reqwest_client_builder()).unwrap();
        let result = request_userinfo::<UserInfo>(
            &http_client,
            &metadata,
            create_token_request(),
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            &create_test_decrypter(),
            (Algorithm::HS256, EncryptionAlgorithm::A128CbcHs256),
        )
        .await;

        assert_eq!(result.unwrap().bsn, "999991772");
    }
}
