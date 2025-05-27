// Copyright 2020-2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use chrono::DateTime;
use chrono::Duration;
use jsonwebtoken::Algorithm;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use serde_json::json;
use serde_json::Value;
use ssri::Integrity;

use crypto::server_keys::generate::Ca;
use crypto::x509::BorrowingCertificate;
use jwt::jwk::jwk_from_p256;
use jwt::EcdsaDecodingKey;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::hasher::Hasher;
use sd_jwt::hasher::Sha256Hasher;
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
        .finish(
            Algorithm::ES256,
            Integrity::from(""),
            &signing_key,
            vec![],
            holder_pubkey,
        )
        .await
        .unwrap();

    (sd_jwt, decoding_key)
}

#[test]
fn simple_sd_jwt() {
    let sd_jwt = SdJwt::spec_simple_structured();
    let disclosed = sd_jwt.into_disclosed_object().unwrap();
    let expected_object = json!({
      "address": {
        "country": "JP",
        "region": "港区"
      },
      "iss": "https://issuer.example.com",
      "iat": 1683000000,
      "exp": 1883000000
    });
    assert_eq!(expected_object.as_object().unwrap(), &disclosed);
}

#[test]
fn complex_sd_jwt() {
    let sd_jwt: SdJwt = SdJwt::spec_complex_structured();
    let disclosed = sd_jwt.into_disclosed_object().unwrap();
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
        .into_presentation(
            &hasher,
            DateTime::from_timestamp_millis(1458304832).unwrap(),
            String::from("https://example.com"),
            String::from("abcdefghi"),
            Algorithm::ES256,
        )?
        .finish(&signing_key)
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
        SdJwt::parse_and_verify(&s, &decoding_key, &Sha256Hasher)?
    };

    assert!(sd_jwt.disclosures().is_empty());

    let disclosed = sd_jwt
        .clone()
        .into_presentation(
            &hasher,
            DateTime::from_timestamp_millis(1458304832).unwrap(),
            String::from("https://example.com"),
            String::from("abcdefghi"),
            Algorithm::ES256,
        )?
        .finish(&holder_signing_key)
        .await?
        .0;

    // Try to serialize & deserialize `with_kb`.
    let with_kb = {
        let s = disclosed.to_string();
        SdJwtPresentation::parse_and_verify(
            &s,
            &decoding_key,
            &Sha256Hasher,
            "https://example.com",
            "abcdefghi",
            Duration::days(36500),
        )?
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
        .into_presentation(
            &hasher,
            DateTime::from_timestamp_millis(1458304832).unwrap(),
            String::from("https://example.com"),
            String::from("abcdefghi"),
            Algorithm::ES256,
        )?
        .finish(&signing_key)
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

    let ca = Ca::generate("myca", Default::default()).unwrap();
    let certificate = BorrowingCertificate::from_certificate_der(ca.as_certificate_der().clone()).unwrap();

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
        .finish(
            Algorithm::ES256,
            Integrity::from(""),
            &issuer_privkey,
            vec![certificate.clone()],
            holder_privkey.verifying_key(),
        )
        .await?;

    assert_eq!(sd_jwt.issuer_certificate_chain(), &vec![certificate]);

    let hasher = Sha256Hasher::new();

    // The holder can withhold from a verifier any concealable claim by calling `conceal`.
    let (presented_sd_jwt, _) = sd_jwt
        .into_presentation(
            &hasher,
            DateTime::from_timestamp_millis(1458304832).unwrap(),
            String::from("https://example.com"),
            String::from("abcdefghi"),
            Algorithm::ES256,
        )?
        .finish(&holder_privkey)
        .await?;

    println!("{}", &presented_sd_jwt);

    SdJwtPresentation::parse_and_verify(
        &presented_sd_jwt.to_string(),
        &EcdsaDecodingKey::from(issuer_privkey.verifying_key()),
        &Sha256Hasher,
        "https://example.com",
        "abcdefghi",
        Duration::days(36500),
    )?;

    Ok(())
}
