use jsonwebtoken::jwk::Jwk;
use serde_json::json;

use jwt::jwk::jwk_to_p256;
use jwt::EcdsaDecodingKey;

use crate::sd_jwt::SdJwt;
use crate::sd_jwt::SdJwtPresentation;

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
pub const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/simple_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-complex-structured-sd-jwt
pub const COMPLEX_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/complex_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-sd-jwt-based-verifiable-cre
pub const SD_JWT_VC: &str = include_str!("../examples/sd_jwt/sd_jwt_vc.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-presentation
pub const WITH_KB_SD_JWT: &str = include_str!("../examples/sd_jwt/with_kb.jwt");

pub fn simple_structured_sd_jwt() -> SdJwt {
    SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap()
}

pub fn complex_structured_sd_jwt() -> SdJwt {
    SdJwt::parse(COMPLEX_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key()).unwrap()
}

pub fn sd_jwt_vc() -> SdJwt {
    SdJwt::parse(SD_JWT_VC, &examples_sd_jwt_decoding_key()).unwrap()
}

pub fn sd_jwt_kb() -> SdJwtPresentation {
    SdJwtPresentation::parse(
        WITH_KB_SD_JWT,
        &examples_sd_jwt_decoding_key(),
        &examples_kb_jwt_decoding_key(),
    )
    .unwrap()
}

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-elliptic-curve-key-used-in-
pub fn examples_sd_jwt_decoding_key() -> EcdsaDecodingKey {
    let jwk = json!({
        "kty": "EC",
        "crv": "P-256",
        "x": "b28d4MwZMjw8-00CG4xfnn9SLMVMM19SlqZpVb_uNtQ",
        "y": "Xv5zWwuoaTgdS6hV43yI6gBwTnjukmFQQnJ_kCxzqk8"
    });

    decoding_key_from_jwk(jwk)
}

// Taken from the cnf claim of the example SD-JWT in
// https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-example-sd-jwt
pub fn examples_kb_jwt_decoding_key() -> EcdsaDecodingKey {
    let jwk = json!({
        "kty": "EC",
        "crv": "P-256",
        "x": "TCAER19Zvu3OHF4j4W4vfSVoHIP1ILilDls7vCeGemc",
        "y": "ZxjiWWbZMQGHVWKVQ4hbSIirsVfuecCE6t4jT9F2HZQ"
    });

    decoding_key_from_jwk(jwk)
}

fn decoding_key_from_jwk(jwk: serde_json::Value) -> EcdsaDecodingKey {
    let jwk: Jwk = serde_json::from_value(jwk).unwrap();
    let verifying_key = jwk_to_p256(&jwk).unwrap();
    EcdsaDecodingKey::from(&verifying_key)
}
