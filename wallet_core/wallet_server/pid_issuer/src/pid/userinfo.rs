use futures::TryFutureExt;
use futures::try_join;
use josekit::JoseError;
use josekit::jwe::JweContentEncryption;
use josekit::jwe::JweDecrypter;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::jwk::JwkSet;
use reqwest::header;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use http_utils::reqwest::HttpJsonClient;
use openid4vc::AuthBearerErrorCode;
use openid4vc::ErrorResponse;
use openid4vc::TokenErrorCode;
use openid4vc::issuer_identifier::IssuerIdentifier;
use openid4vc::metadata::jwks::HttpJwksClient;
use openid4vc::metadata::jwks::JwksError;
use openid4vc::metadata::oauth_metadata::AuthorizationServerMetadata;
use openid4vc::metadata::well_known;
use openid4vc::metadata::well_known::WellKnownError;

const APPLICATION_JWT: &str = "application/jwt";

#[derive(Debug, thiserror::Error)]
pub enum UserInfoError {
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("error fetching well-known metadata: {0}")]
    WellKnown(#[from] WellKnownError),

    #[error("config has no JWKS URI")]
    NoJwksUri,

    #[error("error requesting access token: {0:?}")]
    RequestingAccessToken(Box<ErrorResponse<TokenErrorCode>>),

    #[error("error requesting userinfo: {0:?}")]
    RequestingUserInfo(Box<ErrorResponse<AuthBearerErrorCode>>),

    #[error("JWE decryption error: {0}")]
    JweDecryption(#[from] JoseError),

    #[error("JWT error: {0}")]
    Jsonwebtoken(#[from] jsonwebtoken::errors::Error),

    #[error("unexpected JWE content encryption algorithm")]
    UnexpectedEncAlgorithm,

    #[error("decrypted JWE payload is not valid UTF-8")]
    JwePayloadNotUtf8,

    #[error("JWT header is missing key ID (kid)")]
    MissingKeyId,

    #[error("JWT key ID not found in JWKS")]
    KeyNotFound,

    #[error("config has no userinfo url")]
    NoUserinfoUrl,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub bsn: String,
}

async fn request_userinfo_jwt(
    http_client: &HttpJsonClient,
    config: &AuthorizationServerMetadata,
    token_request: openid4vc::token::TokenRequest,
) -> Result<String, UserInfoError> {
    // Get userinfo endpoint from discovery, throw an error otherwise.
    let endpoint = config.userinfo_endpoint.clone().ok_or(UserInfoError::NoUserinfoUrl)?;

    let response = http_client
        .post(config.token_endpoint.clone(), |request| request.form(&token_request))
        .await?;

    let token_response = {
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = response.json::<ErrorResponse<TokenErrorCode>>().await?;
            return Err(UserInfoError::RequestingAccessToken(error.into()));
        } else {
            response.json::<openid4vc::token::TokenResponse>().await?
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
            let error = response.json::<ErrorResponse<AuthBearerErrorCode>>().await?;
            return Err(UserInfoError::RequestingUserInfo(error.into()));
        } else {
            response.text().await?
        }
    };

    Ok(jwt)
}

fn decrypt_jwe(
    jwe_token: &str,
    decrypter: &impl JweDecrypter,
    expected_enc_alg: &impl JweContentEncryption,
) -> Result<Vec<u8>, UserInfoError> {
    let (jwe_payload, header) = josekit::jwe::deserialize_compact(jwe_token, decrypter)?;

    // Check the "enc" header to confirm that that the content is encoded with the expected algorithm.
    if header.content_encryption() == Some(expected_enc_alg.name()) {
        Ok(jwe_payload)
    } else {
        Err(UserInfoError::UnexpectedEncAlgorithm)
    }
}

pub async fn request_userinfo<C>(
    http_client: &HttpJsonClient,
    authorization_server: &IssuerIdentifier,
    token_request: openid4vc::token::TokenRequest,
    client_id: &str,
    expected_sig_alg: Algorithm,
    encryption: Option<(&impl JweDecrypter, &impl JweContentEncryption)>,
) -> Result<C, UserInfoError>
where
    C: DeserializeOwned,
{
    let config: AuthorizationServerMetadata = well_known::fetch_well_known(
        http_client,
        authorization_server,
        well_known::WellKnownPath::OpenidConfiguration,
    )
    .await?;

    let jwks_client = HttpJwksClient::new(http_client.clone());
    let jwks_uri = config.jwks_uri.clone().ok_or(UserInfoError::NoJwksUri)?;

    let (jwt, jwks) = try_join!(
        request_userinfo_jwt(http_client, &config, token_request),
        jwks_client.jwks(jwks_uri).map_err(|e| match e {
            JwksError::Http(e) => UserInfoError::Http(e),
        })
    )?;

    let jws = match encryption {
        Some((decrypter, expected_enc_alg)) => String::from_utf8(decrypt_jwe(&jwt, decrypter, expected_enc_alg)?)
            .map_err(|_| UserInfoError::JwePayloadNotUtf8)?,
        None => jwt,
    };

    verify_against_keys(&jws, &jwks, client_id, expected_sig_alg)
}

// We can't use our own `Jwt` types here because they only support ECDSA/P256.
fn verify_against_keys<C: DeserializeOwned>(
    token: &str,
    jwks: &JwkSet,
    audience: &str,
    algorithm: Algorithm,
) -> Result<C, UserInfoError> {
    let header = jsonwebtoken::decode_header(token)?;

    let kid = header.kid.as_deref().ok_or(UserInfoError::MissingKeyId)?;
    let jwk = jwks.find(kid).ok_or(UserInfoError::KeyNotFound)?;
    let key = DecodingKey::from_jwk(jwk)?;

    let mut validation = Validation::new(algorithm);
    validation.required_spec_claims.clear(); // don't require exp
    validation.set_audience(&[audience]);

    let verified = jsonwebtoken::decode(token, &key, &validation)?;

    Ok(verified.claims)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use josekit::jwe::ECDH_ES_A256KW;
    use josekit::jwe::JweHeader;
    use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
    use josekit::jwe::enc::aescbc_hmac::AescbcHmacJweEncryption;
    use josekit::jwk::Jwk;
    use josekit::jwk::KeyPair;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::EncodingKey;
    use jsonwebtoken::Header;
    use serde_json::json;

    use super::*;

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

    fn make_jws(include_kid: bool) -> (String, JwkSet) {
        let algoritm = Algorithm::HS256;
        let kid = "hmac_key_id";

        let mut header = Header::new(algoritm);
        if include_kid {
            header.kid = Some(kid.to_string());
        }
        let encoding_key = EncodingKey::from_secret(b"secret hmac key");
        let jws = jsonwebtoken::encode(&header, LazyLock::force(&JWS_PAYLOAD), &encoding_key).unwrap();

        let mut jwk = jsonwebtoken::jwk::Jwk::from_encoding_key(&encoding_key, algoritm).unwrap();
        jwk.common.key_id = Some(kid.to_string());
        let jwks = JwkSet { keys: vec![jwk] };

        (jws, jwks)
    }

    #[test]
    fn test_verify_against_keys_success() {
        let (jws, jwks) = make_jws(true);

        let payload = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect("verifying JWS should succeed");

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
        let (jws, jwks) = make_jws(false);

        let error = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::MissingKeyId);
    }

    #[test]
    fn test_verify_against_keys_error_key_not_found() {
        let (jws, mut jwks) = make_jws(true);

        jwks.keys.first_mut().unwrap().common.key_id = Some("wrong_kid".to_string());

        let error = verify_against_keys::<serde_json::Value>(
            &jws,
            &jwks,
            "3e58016e-bc2e-40d5-b4b1-a3e25f6193b9",
            Algorithm::HS256,
        )
        .expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::KeyNotFound);
    }

    #[test]
    fn test_verify_against_keys_error_wrong_aud() {
        let (jws, jwks) = make_jws(true);

        let error = verify_against_keys::<serde_json::Value>(&jws, &jwks, "wrong_aud", Algorithm::HS256)
            .expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::Jsonwebtoken(_));
    }

    #[test]
    fn test_verify_against_keys_error_wrong_alg() {
        let (jws, jwks) = make_jws(true);

        let error = verify_against_keys::<serde_json::Value>(&jws, &jwks, "wrong_aud", Algorithm::HS512)
            .expect_err("verifying JWS should fail");

        assert_matches!(error, UserInfoError::Jsonwebtoken(_));
    }

    const JWE_ENC: AescbcHmacJweEncryption = AescbcHmacJweEncryption::A128cbcHs256;
    const JWE_ALG: EcdhEsJweAlgorithm = ECDH_ES_A256KW;

    fn make_jwe(payload: &[u8]) -> (String, Jwk) {
        let key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
        let jwk = key_pair.to_jwk_key_pair();

        let mut header = JweHeader::new();
        header.set_content_encryption(JWE_ENC.name());

        let encrypter = JWE_ALG.encrypter_from_jwk(&jwk).unwrap();
        let jwe = josekit::jwe::serialize_compact(payload, &header, &encrypter).unwrap();

        (jwe, jwk)
    }

    #[test]
    fn test_decrypt_jwe_success() {
        let payload = b"hello world";
        let (jwe, jwk) = make_jwe(payload);
        let decrypter = JWE_ALG.decrypter_from_jwk(&jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &JWE_ENC).unwrap();

        assert_eq!(result, payload);
    }

    #[test]
    fn test_decrypt_jwe_wrong_enc_algorithm() {
        let wrong_enc = AescbcHmacJweEncryption::A256cbcHs512;
        let (jwe, jwk) = make_jwe(b"payload");
        let decrypter = JWE_ALG.decrypter_from_jwk(&jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &wrong_enc);

        assert_matches!(result, Err(UserInfoError::UnexpectedEncAlgorithm));
    }

    #[test]
    fn test_decrypt_jwe_wrong_key() {
        let (jwe, _) = make_jwe(b"payload");

        let other_jwk = EcKeyPair::generate(EcCurve::P256).unwrap().to_jwk_key_pair();
        let decrypter = JWE_ALG.decrypter_from_jwk(&other_jwk).unwrap();

        let result = decrypt_jwe(&jwe, &decrypter, &JWE_ENC);

        assert_matches!(result, Err(UserInfoError::JweDecryption(_)));
    }
}
