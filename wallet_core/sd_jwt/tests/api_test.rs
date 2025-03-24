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
    // Values taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-06.html#name-example-2-handling-structur
    let sd_jwt = "eyJhbGciOiAiRVMyNTYiLCAidHlwIjogImV4YW1wbGUrc2Qtand0In0.eyJfc2QiOiBbIkM5aW5wNllvUmFFWFI0Mjd6WUpQN1FyazFXSF84YmR3T0FfWVVyVW5HUVUiLCAiS3VldDF5QWEwSElRdlluT1ZkNTloY1ZpTzlVZzZKMmtTZnFZUkJlb3d2RSIsICJNTWxkT0ZGekIyZDB1bWxtcFRJYUdlcmhXZFVfUHBZZkx2S2hoX2ZfOWFZIiwgIlg2WkFZT0lJMnZQTjQwVjd4RXhad1Z3ejd5Um1MTmNWd3Q1REw4Ukx2NGciLCAiWTM0em1JbzBRTExPdGRNcFhHd2pCZ0x2cjE3eUVoaFlUMEZHb2ZSLWFJRSIsICJmeUdwMFdUd3dQdjJKRFFsbjFsU2lhZW9iWnNNV0ExMGJRNTk4OS05RFRzIiwgIm9tbUZBaWNWVDhMR0hDQjB1eXd4N2ZZdW8zTUhZS08xNWN6LVJaRVlNNVEiLCAiczBCS1lzTFd4UVFlVTh0VmxsdE03TUtzSVJUckVJYTFQa0ptcXhCQmY1VSJdLCAiaXNzIjogImh0dHBzOi8vaXNzdWVyLmV4YW1wbGUuY29tIiwgImlhdCI6IDE2ODMwMDAwMDAsICJleHAiOiAxODgzMDAwMDAwLCAiYWRkcmVzcyI6IHsiX3NkIjogWyI2YVVoelloWjdTSjFrVm1hZ1FBTzN1MkVUTjJDQzFhSGhlWnBLbmFGMF9FIiwgIkF6TGxGb2JrSjJ4aWF1cFJFUHlvSnotOS1OU2xkQjZDZ2pyN2ZVeW9IemciLCAiUHp6Y1Z1MHFiTXVCR1NqdWxmZXd6a2VzRDl6dXRPRXhuNUVXTndrclEtayIsICJiMkRrdzBqY0lGOXJHZzhfUEY4WmN2bmNXN3p3Wmo1cnlCV3ZYZnJwemVrIiwgImNQWUpISVo4VnUtZjlDQ3lWdWIyVWZnRWs4anZ2WGV6d0sxcF9KbmVlWFEiLCAiZ2xUM2hyU1U3ZlNXZ3dGNVVEWm1Xd0JUdzMyZ25VbGRJaGk4aEdWQ2FWNCIsICJydkpkNmlxNlQ1ZWptc0JNb0d3dU5YaDlxQUFGQVRBY2k0MG9pZEVlVnNBIiwgInVOSG9XWWhYc1poVkpDTkUyRHF5LXpxdDd0NjlnSkt5NVFhRnY3R3JNWDQiXX0sICJfc2RfYWxnIjogInNoYS0yNTYifQ.gR6rSL7urX79CNEvTQnP1MH5xthG11ucIV44SqKFZ4Pvlu_u16RfvXQd4k4CAIBZNKn2aTI18TfvFwV97gJFoA~WyJHMDJOU3JRZmpGWFE3SW8wOXN5YWpBIiwgInJlZ2lvbiIsICJcdTZlMmZcdTUzM2EiXQ~WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgImNvdW50cnkiLCAiSlAiXQ~";
    let sd_jwt: SdJwt = SdJwt::parse(sd_jwt).unwrap();
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
