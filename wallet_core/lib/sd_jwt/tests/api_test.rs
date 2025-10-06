use std::collections::HashSet;
use std::time::Duration;

use futures::FutureExt;
use itertools::Itertools;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use serde_json::Value;
use serde_json::json;

use attestation_types::claim_path::ClaimPath;
use crypto::mock_remote::MockRemoteEcdsaKey;
use crypto::mock_remote::MockRemoteWscd;
use crypto::server_keys::generate::Ca;
use crypto::x509::CertificateUsage;
use jwt::EcdsaDecodingKey;
use jwt::jwk::jwk_from_p256;
use sd_jwt::builder::SdJwtBuilder;
use sd_jwt::builder::SignedSdJwt;
use sd_jwt::disclosure::DisclosureContent;
use sd_jwt::hasher::Hasher;
use sd_jwt::hasher::Sha256Hasher;
use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
use sd_jwt::sd_jwt::SdJwtVcClaims;
use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
use sd_jwt::sd_jwt::UnverifiedSdJwt;
use sd_jwt::sd_jwt::UnverifiedSdJwtPresentation;
use sd_jwt::sd_jwt::VerifiedSdJwt;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

async fn make_sd_jwt(
    claims: Value,
    disclosable_values: impl IntoIterator<Item = VecNonEmpty<ClaimPath>>,
    holder_pubkey: &VerifyingKey,
) -> (SignedSdJwt, EcdsaDecodingKey) {
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuer_keypair = ca.generate_issuer_mock().unwrap();

    let claims = SdJwtVcClaims::example_from_json(holder_pubkey, claims, &MockTimeGenerator::default());

    let sd_jwt = disclosable_values
        .into_iter()
        .fold(SdJwtBuilder::new(claims).unwrap(), |builder, paths| {
            builder.make_concealable(paths).unwrap()
        })
        .finish(&issuer_keypair)
        .await
        .unwrap();

    (sd_jwt, issuer_keypair.certificate().public_key().into())
}

#[test]
fn simple_sd_jwt() {
    let sd_jwt = VerifiedSdJwt::spec_simple_structured();
    let disclosed = sd_jwt.to_disclosed_object().unwrap();
    let expected = json!({
        "address": {
            "country": "JP",
            "region": "港区"
        },
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000
    })
    .as_object()
    .unwrap()
    .to_owned();

    assert_eq!(expected, disclosed);
}

#[test]
fn complex_sd_jwt() {
    let sd_jwt = VerifiedSdJwt::spec_complex_structured();
    let disclosed = sd_jwt.to_disclosed_object().unwrap();
    let expected = json!({
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
        "iss": "https://issuer.example.com/",
        "iat": 1683000000,
        "exp": 1883000000
    })
    .as_object()
    .unwrap()
    .to_owned();

    assert_eq!(expected, disclosed);
}

#[tokio::test]
async fn concealing_property_of_concealable_value_works() {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let (signed_sd_jwt, _) = make_sd_jwt(
        json!({
            "iss": "https://issuer.example.com/",
            "iat": 1683000000,
            "parent": {
                "property1": "value1",
                "property2": [1, 2, 3]
            }
        }),
        [
            vec![
                ClaimPath::SelectByKey(String::from("parent")),
                ClaimPath::SelectByKey(String::from("property1")),
            ]
            .try_into()
            .unwrap(),
            vec![
                ClaimPath::SelectByKey(String::from("parent")),
                ClaimPath::SelectByKey(String::from("property2")),
                ClaimPath::SelectByIndex(0),
            ]
            .try_into()
            .unwrap(),
            vec![ClaimPath::SelectByKey(String::from("parent"))].try_into().unwrap(),
        ],
        holder_signing_key.verifying_key(),
    )
    .await;

    let holder_key = SigningKey::random(&mut OsRng);
    let verified_sd_jwt = signed_sd_jwt.into_verified();
    verified_sd_jwt
        .into_presentation_builder()
        .finish()
        .sign(
            KeyBindingJwtBuilder::new(String::from("https://example.com"), String::from("abcdefghi")),
            &holder_key,
            &MockTimeGenerator::default(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn sd_jwt_without_disclosures_works() {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let (signed_sd_jwt, decoding_key) = make_sd_jwt(
        json!({
            "iss": "https://issuer.example.com",
            "iat": 1683000000,
            "parent": {
                "property1": "value1",
                "property2": [1, 2, 3]
            }
        }),
        [],
        holder_signing_key.verifying_key(),
    )
    .await;

    // Try to serialize & deserialize `sd_jwt`.
    let verified_sd_jwt = signed_sd_jwt
        .to_string()
        .parse::<UnverifiedSdJwt>()
        .unwrap()
        .into_verified(&decoding_key)
        .unwrap();

    assert!(verified_sd_jwt.disclosures().is_empty());

    let disclosed = verified_sd_jwt
        .into_presentation_builder()
        .finish()
        .sign(
            KeyBindingJwtBuilder::new(String::from("https://example.com"), String::from("abcdefghi")),
            &holder_signing_key,
            &MockTimeGenerator::default(),
        )
        .await
        .unwrap();

    // Try to serialize & deserialize `with_kb`.
    let with_kb = &disclosed
        .to_string()
        .parse::<UnverifiedSdJwtPresentation>()
        .unwrap()
        .into_verified(
            &decoding_key,
            "https://example.com",
            "abcdefghi",
            Duration::from_secs(60),
            &MockTimeGenerator::default(),
        )
        .unwrap();

    assert!(with_kb.sd_jwt().disclosures().is_empty());
}

#[tokio::test]
async fn sd_jwt_sd_hash() {
    let holder_signing_key = SigningKey::random(&mut OsRng);
    let hasher = Sha256Hasher;

    let (signed_sd_jwt, _) = make_sd_jwt(
        json!({
            "iss": "https://issuer.example.com",
            "iat": 1683000000,
            "parent": {
                "property1": "value1",
                "property2": [1, 2, 3]
            }
        }),
        [
            vec![
                ClaimPath::SelectByKey(String::from("parent")),
                ClaimPath::SelectByKey(String::from("property1")),
            ]
            .try_into()
            .unwrap(),
            vec![
                ClaimPath::SelectByKey(String::from("parent")),
                ClaimPath::SelectByKey(String::from("property2")),
                ClaimPath::SelectByIndex(0),
            ]
            .try_into()
            .unwrap(),
            vec![ClaimPath::SelectByKey(String::from("parent"))].try_into().unwrap(),
        ],
        holder_signing_key.verifying_key(),
    )
    .await;

    let signing_key = SigningKey::random(&mut OsRng);

    let verified_sd_jwt = signed_sd_jwt.into_verified();
    let signed_presentation = verified_sd_jwt
        .into_presentation_builder()
        .finish()
        .sign(
            KeyBindingJwtBuilder::new(String::from("https://example.com"), String::from("abcdefghi")),
            &signing_key,
            &MockTimeGenerator::default(),
        )
        .await
        .unwrap();

    let encoded_kb_jwt = signed_presentation.to_string();
    let (issued_sd_jwt, _) = encoded_kb_jwt.rsplit_once("~").unwrap();

    let actual_sd_hash = signed_presentation
        .key_binding_jwt()
        .to_owned()
        .into_verified()
        .payload()
        .sd_hash
        .clone();
    let expected_sd_hash = hasher.encoded_digest(&format!("{issued_sd_jwt}~"));

    assert_eq!(*actual_sd_hash, expected_sd_hash);
}

#[tokio::test]
async fn test_presentation() {
    let holder_key = SigningKey::random(&mut OsRng);

    let claims = SdJwtVcClaims::example_from_json(
        holder_key.verifying_key(),
        json!({
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
            "nationalities": [
                "US",
                "DE"
            ]
        }),
        &MockTimeGenerator::default(),
    );

    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let issuer_keypair = ca.generate_issuer_mock().unwrap();

    println!(
        "issuer_privkey pubkey: {0}",
        serde_json::to_string_pretty(&jwk_from_p256(issuer_keypair.certificate().public_key()).unwrap()).unwrap()
    );
    println!(
        "holder_key pubkey: {0}",
        serde_json::to_string_pretty(&jwk_from_p256(holder_key.verifying_key()).unwrap()).unwrap()
    );

    // issuer signs SD-JWT
    let signed_sd_jwt = SdJwtBuilder::new(claims)
        .unwrap()
        .make_concealable(vec![ClaimPath::SelectByKey(String::from("email"))].try_into().unwrap())
        .unwrap()
        .make_concealable(
            vec![ClaimPath::SelectByKey(String::from("phone_number"))]
                .try_into()
                .unwrap(),
        )
        .unwrap()
        .make_concealable(
            vec![
                ClaimPath::SelectByKey(String::from("address")),
                ClaimPath::SelectByKey(String::from("street_address")),
            ]
            .try_into()
            .unwrap(),
        )
        .unwrap()
        .make_concealable(
            vec![ClaimPath::SelectByKey(String::from("address"))]
                .try_into()
                .unwrap(),
        )
        .unwrap()
        .make_concealable(
            vec![
                ClaimPath::SelectByKey(String::from("nationalities")),
                ClaimPath::SelectByIndex(0),
            ]
            .try_into()
            .unwrap(),
        )
        .unwrap()
        .add_decoys(&[ClaimPath::SelectByKey(String::from("nationalities"))], 1)
        .unwrap()
        .add_decoys(&[], 2)
        .unwrap()
        .finish(&issuer_keypair)
        .await
        .unwrap();

    let verified_sd_jwt = signed_sd_jwt.into_verified();
    assert_eq!(
        verified_sd_jwt.issuer_certificate_chain(),
        &vec_nonempty![issuer_keypair.certificate().to_owned()]
    );

    // The holder can withhold from a verifier any concealable claim by calling `conceal`.
    let signed_presentation = verified_sd_jwt
        .into_presentation_builder()
        .disclose(&vec![ClaimPath::SelectByKey(String::from("email"))].try_into().unwrap())
        .unwrap()
        .disclose(
            &vec![
                ClaimPath::SelectByKey(String::from("address")),
                ClaimPath::SelectByKey(String::from("street_address")),
            ]
            .try_into()
            .unwrap(),
        )
        .unwrap()
        .finish()
        .sign(
            KeyBindingJwtBuilder::new(String::from("https://example.com"), String::from("abcdefghi")),
            &holder_key,
            &MockTimeGenerator::default(),
        )
        .await
        .unwrap();

    let parsed_presentation = signed_presentation
        .to_string()
        .parse::<UnverifiedSdJwtPresentation>()
        .unwrap()
        .into_verified(
            &EcdsaDecodingKey::from(issuer_keypair.certificate().public_key()),
            "https://example.com",
            "abcdefghi",
            Duration::from_secs(60),
            &MockTimeGenerator::default(),
        )
        .unwrap();

    let disclosed_paths = parsed_presentation
        .sd_jwt()
        .disclosures()
        .values()
        .map(|v| match &v.content {
            DisclosureContent::ObjectProperty(_, name, _) => name.as_str(),
            _ => panic!("unexpected disclosure content"),
        })
        .collect::<HashSet<_>>();

    assert_eq!(HashSet::from(["email", "address", "street_address"]), disclosed_paths);
}

#[test]
fn test_wscd_presentation() {
    let time = MockTimeGenerator::default();

    let holder_key = MockRemoteEcdsaKey::new_random("holder_key".to_string());
    let claims = SdJwtVcClaims::example_from_json(
        holder_key.verifying_key(),
        json!({
            "given_name": "John",
            "family_name": "Doe",
        }),
        &time,
    );

    let ca = Ca::generate("myca", Default::default()).unwrap();
    let issuer_key_pair = ca
        .generate_key_pair("mycert", CertificateUsage::Mdl, Default::default())
        .unwrap();

    let wscd = MockRemoteWscd::new(vec![holder_key]);

    // Create a SD-JWT, signed by the issuer.
    let signed_sd_jwt = SdJwtBuilder::new(claims)
        .unwrap()
        .make_concealable(vec_nonempty![ClaimPath::SelectByKey(String::from("given_name"))])
        .unwrap()
        .make_concealable(vec_nonempty![ClaimPath::SelectByKey(String::from("family_name"))])
        .unwrap()
        .finish(&issuer_key_pair)
        .now_or_never()
        .unwrap()
        .expect("signing SD-JWT should succeed");

    let verified_sd_jwt = signed_sd_jwt.into_verified();
    let unsigned_presentation = verified_sd_jwt
        .into_presentation_builder()
        .disclose(&vec_nonempty![ClaimPath::SelectByKey(String::from("family_name"))])
        .unwrap()
        .finish();

    let (sd_jwt_presentations, poa) = UnsignedSdJwtPresentation::sign_multiple(
        vec_nonempty![(unsigned_presentation, "holder_key")],
        KeyBindingJwtBuilder::new(String::from("https://example.com"), String::from("abcdefghi")),
        &wscd,
        (),
        &time,
    )
    .now_or_never()
    .unwrap()
    .expect("signing SD-JWT presentation should succeed");

    assert!(poa.is_none());

    let signed_presentation = sd_jwt_presentations.into_iter().exactly_one().unwrap();
    let unverified_sd_jwt_presentation = signed_presentation.into_unverified();

    let verified_sd_jwt_presentation = unverified_sd_jwt_presentation
        .into_verified_against_trust_anchors(
            &[ca.to_trust_anchor()],
            "https://example.com",
            "abcdefghi",
            Duration::from_secs(10 * 60),
            &MockTimeGenerator::default(),
        )
        .expect("validating SD-JWT presentation should succeed");

    let disclosed_object = verified_sd_jwt_presentation.sd_jwt().to_disclosed_object().unwrap();

    assert_eq!(
        disclosed_object.get("family_name").and_then(|val| val.as_str()),
        Some("Doe")
    );
    assert!(!disclosed_object.contains_key("given_name"));
}
