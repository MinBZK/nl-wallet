// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use chrono::DateTime;
use jsonwebtoken::Algorithm;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use serde_json::json;
use serde_json::Value;

use jwt::jwk::jwk_from_p256;
use jwt::EcdsaDecodingKey;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::examples;
use sd_jwt::hasher::Hasher;
use sd_jwt::hasher::Sha256Hasher;
use sd_jwt::key_binding_jwt_claims::KeyBindingJwt;
use sd_jwt::key_binding_jwt_claims::KeyBindingJwtBuilder;
use sd_jwt::sd_jwt::SdJwt;
use sd_jwt::sd_jwt::SdJwtPresentation;

async fn make_sd_jwt(
    object: Value,
    disclosable_values: impl IntoIterator<Item = &str>,
    holder_pubkey: &VerifyingKey,
) -> (SdJwt, EcdsaDecodingKey) {
    let signing_key = SigningKey::random(&mut OsRng);
    let decoding_key = EcdsaDecodingKey::from(signing_key.verifying_key());

    let sd_jwt = disclosable_values
        .into_iter()
        .fold(SdJwtBuilder::new(object).unwrap(), |builder, path| {
            builder.make_concealable(path).unwrap()
        })
        .finish(Algorithm::ES256, &signing_key, holder_pubkey)
        .await
        .unwrap();

    (sd_jwt, decoding_key)
}

fn make_kb_jwt_builder() -> KeyBindingJwtBuilder {
    KeyBindingJwt::builder()
        .nonce("abcdefghi")
        .aud("https://example.com")
        .iat(DateTime::from_timestamp_millis(1458304832).unwrap())
}

#[test]
fn simple_sd_jwt() {
    let sd_jwt = examples::simple_structured_sd_jwt();
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
    let sd_jwt: SdJwt = examples::complex_structured_sd_jwt();
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
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let hasher = Sha256Hasher::new();
    let (sd_jwt, _) = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
        holder_signing_key.verifying_key(),
    )
    .await;

    let signing_key = SigningKey::random(&mut OsRng);

    let removed_disclosures = sd_jwt
        .into_presentation(make_kb_jwt_builder(), &hasher)?
        .conceal("/parent")?
        .finish(Algorithm::ES256, &signing_key)
        .await?
        .1;
    assert_eq!(removed_disclosures.len(), 3);

    Ok(())
}

#[tokio::test]
async fn concealing_property_of_concealable_value_works() -> anyhow::Result<()> {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let hasher = Sha256Hasher::new();
    let (sd_jwt, _) = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
        holder_signing_key.verifying_key(),
    )
    .await;

    let signing_key = SigningKey::random(&mut OsRng);

    sd_jwt
        .into_presentation(make_kb_jwt_builder(), &hasher)?
        .conceal("/parent/property2/0")?
        .finish(Algorithm::ES256, &signing_key)
        .await?;

    Ok(())
}

#[tokio::test]
async fn sd_jwt_without_disclosures_works() -> anyhow::Result<()> {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let hasher = Sha256Hasher::new();
    let (sd_jwt, decoding_key) = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        [],
        holder_signing_key.verifying_key(),
    )
    .await;

    println!("{}", sd_jwt);

    // Try to serialize & deserialize `sd_jwt`.
    let sd_jwt = {
        let s = sd_jwt.to_string();
        SdJwt::parse_and_verify(&s, &decoding_key)?
    };

    assert!(sd_jwt.disclosures().is_empty());

    let disclosed = sd_jwt
        .clone()
        .into_presentation(make_kb_jwt_builder(), &hasher)?
        .finish(Algorithm::ES256, &holder_signing_key)
        .await?
        .0;

    // Try to serialize & deserialize `with_kb`.
    let with_kb = {
        let s = disclosed.to_string();
        SdJwtPresentation::parse_and_verify(&s, &decoding_key)?
    };

    assert!(with_kb.sd_jwt().disclosures().is_empty());

    Ok(())
}

#[tokio::test]
async fn sd_jwt_sd_hash() -> anyhow::Result<()> {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let hasher = Sha256Hasher::new();

    let (sd_jwt, _) = make_sd_jwt(
        json!({"parent": {"property1": "value1", "property2": [1, 2, 3]}}),
        ["/parent/property1", "/parent/property2/0", "/parent"],
        holder_signing_key.verifying_key(),
    )
    .await;

    let signing_key = SigningKey::random(&mut OsRng);

    let disclosed = sd_jwt
        .clone()
        .into_presentation(make_kb_jwt_builder(), &hasher)?
        .conceal("/parent/property1")?
        .finish(Algorithm::ES256, &signing_key)
        .await?
        .0;

    let encoded_kb_jwt = disclosed.to_string();
    let (issued_sd_jwt, _kb) = encoded_kb_jwt.rsplit_once("~").unwrap();

    let actual_sd_hash = disclosed.key_binding_jwt().claims().sd_hash.clone();
    let expected_sd_hash = hasher.encoded_digest(&format!("{}~", issued_sd_jwt));

    assert_eq!(actual_sd_hash, expected_sd_hash);

    Ok(())
}

#[tokio::test]
async fn test_presentation() -> anyhow::Result<()> {
    let object = json!({
      "sub": "user_42",
      "given_name": "John",
      "family_name": "Doe",
      "email": "johndoe@example.com",
      "phone_number": "+1-202-555-0101",
      "phone_number_verified": true,
      "address": {
        "street_address": "123 Main St",
        "locality": "Anytown",
        "region": "Anystate",
        "country": "US"
      },
      "birthdate": "1940-01-01",
      "updated_at": 1570000000,
      "nationalities": [
        "US",
        "DE"
      ]
    });

    let issuer_privkey = SigningKey::random(&mut OsRng);
    println!(
        "issuer_privkey pubkey: {0}",
        serde_json::to_string_pretty(&jwk_from_p256(issuer_privkey.verifying_key())?)?
    );
    let holder_privkey = SigningKey::random(&mut OsRng);
    println!(
        "holder_privkey pubkey: {0}",
        serde_json::to_string_pretty(&jwk_from_p256(holder_privkey.verifying_key())?)?
    );

    // issuer signs SD-JWT
    let sd_jwt = SdJwtBuilder::new(object)?
        .make_concealable("/email")?
        .make_concealable("/phone_number")?
        .make_concealable("/address/street_address")?
        .make_concealable("/address")?
        .make_concealable("/nationalities/0")?
        .add_decoys("/nationalities", 1)?
        .add_decoys("", 2)?
        .finish(Algorithm::ES256, &issuer_privkey, holder_privkey.verifying_key())
        .await?;

    let hasher = Sha256Hasher::new();

    // The holder can withhold from a verifier any concealable claim by calling `conceal`.
    let (presented_sd_jwt, _) = sd_jwt
        .into_presentation(make_kb_jwt_builder(), &hasher)?
        .conceal("/email")?
        .finish(Algorithm::ES256, &holder_privkey)
        .await?;

    println!("{}", &presented_sd_jwt);

    SdJwtPresentation::parse_and_verify(
        &presented_sd_jwt.to_string(),
        &EcdsaDecodingKey::from(issuer_privkey.verifying_key()),
    )?;

    Ok(())
}
