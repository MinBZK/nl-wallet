// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use chrono::DateTime;
use josekit::jws::alg::hmac::HmacJwsSigner;
use josekit::jws::JwsHeader;
use josekit::jws::HS256;
use josekit::jwt;
use josekit::jwt::JwtPayload;
use serde_json::json;
use serde_json::Value;

use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::hasher::Hasher;
use sd_jwt::hasher::Sha256Hasher;
use sd_jwt::key_binding_jwt_claims::KeyBindingJwtBuilder;
use sd_jwt::key_binding_jwt_claims::{DigitalSignaturAlgorithm, KeyBindingJwt};
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt::signer::JsonObject;
use sd_jwt::signer::JwsSigner;

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/simple_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-complex-structured-sd-jwt
const COMPLEX_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/complex_structured.jwt");

const HMAC_SECRET: &[u8; 32] = b"0123456789ABCDEF0123456789ABCDEF";

struct HmacSignerAdapter(HmacJwsSigner);

#[async_trait]
impl JwsSigner for HmacSignerAdapter {
    type Error = josekit::JoseError;
    async fn sign(&self, header: &JsonObject, payload: &JsonObject) -> Result<Vec<u8>, Self::Error> {
        let header = JwsHeader::from_map(header.clone())?;
        let payload = JwtPayload::from_map(payload.clone())?;

        jwt::encode_with_signer(&payload, &header, &self.0).map(String::into_bytes)
    }
}

async fn make_sd_jwt(object: Value, disclosable_values: impl IntoIterator<Item = &str>) -> SdJwt {
    let signer = HmacSignerAdapter(HS256.signer_from_bytes(HMAC_SECRET).unwrap());
    disclosable_values
        .into_iter()
        .fold(SdJwtBuilder::new(object).unwrap(), |builder, path| {
            builder.make_concealable(path).unwrap()
        })
        .finish(&signer, "HS256")
        .await
        .unwrap()
}

fn make_kb_jwt_builder() -> KeyBindingJwtBuilder {
    KeyBindingJwt::builder()
        .nonce("abcdefghi")
        .aud("https://example.com")
        .iat(DateTime::from_timestamp_millis(1458304832).unwrap())
}

#[test]
fn simple_sd_jwt() {
    let sd_jwt: SdJwt = SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap();
    let disclosed = sd_jwt.into_disclosed_object(&Sha256Hasher::new()).unwrap();
    let expected_object = json!({
      "address": {
        "country": "JP",
        "region": "港区"
      },
      "iss": "https://issuer.example.com",
      "iat": 1683000000,
      "exp": 1883000000
    }
    );
    assert_eq!(expected_object.as_object().unwrap(), &disclosed);
}

#[test]
fn complex_sd_jwt() {
    let sd_jwt: SdJwt = SdJwt::parse(COMPLEX_STRUCTURED_SD_JWT).unwrap();
    let disclosed = sd_jwt.into_disclosed_object(&Sha256Hasher::new()).unwrap();
    let expected_object = json!({
      "verified_claims": {
            "verification": {
                "time": "2012-04-23T18:25Z",
                "trust_framework": "de_aml",
                "evidence": [
                    { "method": "pipp" }
                ]
            },
            "claims": {
                "address": {
                    "locality": "Maxstadt",
                    "postal_code": "12344",
                    "country": "DE",
                    "street_address": "Weidenstraße 22"
                },
                "given_name": "Max",
                "family_name": "Müller"
            }
      },
      "iss": "https://issuer.example.com",
      "iat": 1683000000,
      "exp": 1883000000
    });
    assert_eq!(expected_object.as_object().unwrap(), &disclosed);
}

#[tokio::test]
async fn concealing_parent_also_removes_all_sub_disclosures() -> anyhow::Result<()> {
    let hasher = Sha256Hasher::new();
    let sd_jwt = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
    )
    .await;

    let removed_disclosures = sd_jwt.into_presentation(&hasher)?.conceal("/parent")?.finish()?.1;
    assert_eq!(removed_disclosures.len(), 3);

    Ok(())
}

#[tokio::test]
async fn concealing_property_of_concealable_value_works() -> anyhow::Result<()> {
    let hasher = Sha256Hasher::new();
    let sd_jwt = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
    )
    .await;

    sd_jwt
        .into_presentation(&hasher)?
        .conceal("/parent/property2/0")?
        .finish()?;

    Ok(())
}

#[tokio::test]
async fn sd_jwt_is_verifiable() -> anyhow::Result<()> {
    let sd_jwt = make_sd_jwt(json!({"key": "value"}), []).await;
    let jwt = sd_jwt.presentation().split_once('~').unwrap().0.to_string();
    let verifier = HS256.verifier_from_bytes(HMAC_SECRET)?;

    jwt::decode_with_verifier(&jwt, &verifier)?;
    Ok(())
}

#[tokio::test]
async fn sd_jwt_without_disclosures_works() -> anyhow::Result<()> {
    let hasher = Sha256Hasher::new();
    let sd_jwt = make_sd_jwt(json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}), []).await;
    // Try to serialize & deserialize `sd_jwt`.
    let sd_jwt = {
        let s = sd_jwt.to_string();
        s.parse::<SdJwt>()?
    };

    assert!(sd_jwt.disclosures().is_empty());
    assert!(sd_jwt.key_binding_jwt().is_none());

    let signer = HmacSignerAdapter(HS256.signer_from_bytes(HMAC_SECRET)?);

    let disclosed = sd_jwt
        .clone()
        .into_presentation(&hasher)?
        .attach_key_binding_jwt(make_kb_jwt_builder())
        .finish_with_key_binding(&hasher, DigitalSignaturAlgorithm::HS256, &signer)
        .await?
        .0;
    // Try to serialize & deserialize `with_kb`.
    let with_kb = {
        let s = disclosed.to_string();
        s.parse::<SdJwt>()?
    };

    assert!(with_kb.disclosures().is_empty());
    assert!(with_kb.key_binding_jwt().is_some());

    Ok(())
}

#[tokio::test]
async fn sd_jwt_sd_hash() -> anyhow::Result<()> {
    let hasher = Sha256Hasher::new();

    let sd_jwt = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
    )
    .await;

    assert!(sd_jwt.key_binding_jwt().is_none());

    let signer = HmacSignerAdapter(HS256.signer_from_bytes(HMAC_SECRET)?);

    let disclosed = sd_jwt
        .clone()
        .into_presentation(&hasher)?
        .conceal("/parent/property1")?
        .attach_key_binding_jwt(make_kb_jwt_builder())
        .finish_with_key_binding(&hasher, DigitalSignaturAlgorithm::HS256, &signer)
        .await?
        .0;

    let encoded_kb_jwt = disclosed.to_string();
    let (issued_sd_jwt, _kb) = encoded_kb_jwt.rsplit_once("~").unwrap();

    let actual_sd_hash = disclosed.key_binding_jwt().unwrap().claims().sd_hash.clone();
    let expected_sd_hash = hasher.encoded_digest(&format!("{}~", issued_sd_jwt));

    assert_eq!(actual_sd_hash, expected_sd_hash);

    Ok(())
}
