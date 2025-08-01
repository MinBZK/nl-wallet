use std::collections::HashMap;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::Duration;
use futures::FutureExt;
use jsonwebtoken::Algorithm;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use serde_json::json;
use ssri::Integrity;

use attestation_types::claim_path::ClaimPath;
use crypto::server_keys::KeyPair;
use crypto::utils::random_string;
use jwt::EcdsaDecodingKey;
use jwt::jwk::jwk_to_p256;

use crate::builder::SdJwtBuilder;
use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::hasher::Hasher;
use crate::hasher::Sha256Hasher;
use crate::sd_jwt::SdJwt;
use crate::sd_jwt::SdJwtPresentation;

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
pub const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/spec/simple_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-complex-structured-sd-jwt
pub const COMPLEX_STRUCTURED_SD_JWT: &str = include_str!("../examples/spec/complex_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-sd-jwt-based-verifiable-cre
pub const SD_JWT_VC: &str = include_str!("../examples/spec/sd_jwt_vc.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-presentation
pub const WITH_KB_SD_JWT: &str = include_str!("../examples/spec/with_kb.jwt");

pub const WITH_KB_SD_JWT_AUD: &str = "https://verifier.example.org";
pub const WITH_KB_SD_JWT_NONCE: &str = "1234567890";

impl SdJwtPresentation {
    pub fn spec_simple_structured() -> SdJwt {
        SdJwt::parse_and_verify(SIMPLE_STRUCTURED_SD_JWT, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap()
    }

    pub fn spec_complex_structured() -> SdJwt {
        SdJwt::parse_and_verify(
            COMPLEX_STRUCTURED_SD_JWT,
            &examples_sd_jwt_decoding_key(),
            &Sha256Hasher,
        )
        .unwrap()
    }

    pub fn spec_sd_jwt_vc() -> SdJwt {
        SdJwt::parse_and_verify(SD_JWT_VC, &examples_sd_jwt_decoding_key(), &Sha256Hasher).unwrap()
    }

    pub fn spec_sd_jwt_kb() -> SdJwtPresentation {
        SdJwtPresentation::parse_and_verify(
            WITH_KB_SD_JWT,
            &examples_sd_jwt_decoding_key(),
            &Sha256Hasher,
            WITH_KB_SD_JWT_AUD,
            WITH_KB_SD_JWT_NONCE,
            Duration::minutes(2),
        )
        .unwrap()
    }

    pub fn example_pid_sd_jwt(issuer_keypair: &KeyPair) -> SdJwt {
        let object = json!({
          "vct": "urn:eudi:pid:nl:1",
          "iat": 1683000000,
          "exp": 1883000000,
          "iss": "https://cert.issuer.example.com",
          "attestation_qualification": "QEAA",
          "bsn": "999991772",
          "recovery_code": "885ed8a2-f07a-4f77-a8df-2e166f5ebd36",
          "given_name": "John",
          "family_name": "Doe",
          "birthdate": "1940-01-01"
        });

        let holder_privkey = SigningKey::random(&mut OsRng);

        // issuer signs SD-JWT
        SdJwtBuilder::new(object)
            .unwrap()
            .make_concealable(
                vec![ClaimPath::SelectByKey(String::from("family_name"))]
                    .try_into()
                    .unwrap(),
            )
            .unwrap()
            .make_concealable(vec![ClaimPath::SelectByKey(String::from("bsn"))].try_into().unwrap())
            .unwrap()
            .add_decoys(&[], 2)
            .unwrap()
            .finish(
                Algorithm::ES256,
                Integrity::from(random_string(32)),
                issuer_keypair.private_key(),
                vec![issuer_keypair.certificate().clone()],
                holder_privkey.verifying_key(),
            )
            .now_or_never()
            .unwrap()
            .unwrap()
    }
}

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-example-sd-jwt-with-recursi
pub fn recursive_disclosures_example() -> (serde_json::Value, HashMap<String, Disclosure>) {
    let claims = json!({
      "_sd": [
        "HvrKX6fPV0v9K_yCVFBiLFHsMaxcD_114Em6VT8x1lg"
      ],
      "iss": "https://issuer.example.com",
      "iat": 1683000000,
      "exp": 1883000000,
      "sub": "6c5c0a49-b589-431d-bae7-219122a9ec2c",
      "_sd_alg": "sha-256"
    });

    let disclosures = vec![
        "WyIyR0xDNDJzS1F2ZUNmR2ZyeU5STjl3IiwgInN0cmVldF9hZGRyZXNzIiwgIlNjaHVsc3RyLiAxMiJd",
        "WyJlbHVWNU9nM2dTTklJOEVZbnN4QV9BIiwgImxvY2FsaXR5IiwgIlNjaHVscGZvcnRhIl0",
        "WyI2SWo3dE0tYTVpVlBHYm9TNXRtdlZBIiwgInJlZ2lvbiIsICJTYWNoc2VuLUFuaGFsdCJd",
        "WyJlSThaV205UW5LUHBOUGVOZW5IZGhRIiwgImNvdW50cnkiLCAiREUiXQ",
        "WyJRZ19PNjR6cUF4ZTQxMmExMDhpcm9BIiwgImFkZHJlc3MiLCB7Il9zZCI6IFsiNnZoOWJxLXpTNEdLTV83R3BnZ1ZiWXp6dTZvT0dYcm1OVkdQSFA3NVVkMCIsICI5Z2pWdVh0ZEZST0NnUnJ0TmNHVVhtRjY1cmRlemlfNkVyX2o3NmttWXlNIiwgIktVUkRQaDRaQzE5LTN0aXotRGYzOVY4ZWlkeTFvVjNhM0gxRGEyTjBnODgiLCAiV045cjlkQ0JKOEhUQ3NTMmpLQVN4VGpFeVc1bTV4NjVfWl8ycm8yamZYTSJdfV0",
    ];

    let disclosure_content = HashMap::from_iter(disclosures.into_iter().map(|disclosure_str| {
        let disclosure_type: DisclosureContent =
            serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(disclosure_str).unwrap()).unwrap();
        let disclosure = Disclosure::try_new(disclosure_type).unwrap();
        (Sha256Hasher.encoded_digest(disclosure_str), disclosure)
    }));

    (claims, disclosure_content)
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

fn decoding_key_from_jwk(jwk: serde_json::Value) -> EcdsaDecodingKey {
    let jwk: Jwk = serde_json::from_value(jwk).unwrap();
    let verifying_key = jwk_to_p256(&jwk).unwrap();
    EcdsaDecodingKey::from(&verifying_key)
}
