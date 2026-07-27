use cose::CoseError;
use cose::TypedCose;
use cose::header_with_x5chain;
use coset::CoseSign1;
use coset::CoseSign1Builder;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateUsage;
use crypto::x509::DistinguishedName;
use crypto::x509::NO_SAN;
use p256::ecdsa::SigningKey;
use rand_core::OsRng;
use serde::Deserialize;
use serde::Serialize;
use utils::generator::TimeGenerator;
use utils::vec_nonempty;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ToyMessage {
    number: u8,
    string: String,
}

impl Default for ToyMessage {
    fn default() -> Self {
        Self {
            number: 42,
            string: "Hello, world!".to_owned(),
        }
    }
}

struct NonSerdePayload;

#[test]
fn typed_cose_clone_and_serde_do_not_require_payload_traits() {
    let cose = TypedCose::<CoseSign1, NonSerdePayload>::from(CoseSign1Builder::new().build());
    let cloned = cose.clone();
    let mut encoded = Vec::new();
    ciborium::ser::into_writer(&cloned, &mut encoded).unwrap();
    let decoded: TypedCose<CoseSign1, NonSerdePayload> = ciborium::de::from_reader(encoded.as_slice()).unwrap();

    assert_eq!(decoded.as_ref().payload, cose.as_ref().payload);
    assert_eq!(decoded.as_ref().signature, cose.as_ref().signature);
}

#[tokio::test]
async fn typed_cose_signs_verifies_and_parses() {
    let key = SigningKey::random(&mut OsRng);
    let payload = ToyMessage::default();
    let cose = TypedCose::sign(&payload, Header::default(), &key, true).await.unwrap();

    cose.verify(key.verifying_key()).unwrap();
    assert_eq!(cose.verify_and_parse(key.verifying_key()).unwrap(), payload);
    assert_eq!(cose.dangerous_parse_unverified().unwrap(), payload);
}

#[tokio::test]
async fn invalid_signature_is_rejected() {
    let key = SigningKey::random(&mut OsRng);
    let mut cose = TypedCose::sign(&ToyMessage::default(), Header::default(), &key, true)
        .await
        .unwrap();

    cose.as_mut().signature[0] ^= u8::MAX;
    assert!(matches!(
        cose.verify(key.verifying_key()),
        Err(CoseError::EcdsaSignatureVerificationFailed(_))
    ));

    cose.as_mut().signature.pop();
    assert!(matches!(
        cose.verify(key.verifying_key()),
        Err(CoseError::EcdsaSignatureParsingFailed(_))
    ));
}

#[tokio::test]
async fn missing_or_unsupported_algorithm_is_rejected() {
    let key = SigningKey::random(&mut OsRng);

    let mut missing = TypedCose::sign(&ToyMessage::default(), Header::default(), &key, true)
        .await
        .unwrap();
    missing.as_mut().protected.header.alg = None;
    assert!(matches!(
        missing.verify(key.verifying_key()),
        Err(CoseError::MissingAlgorithm)
    ));

    let mut unsupported = TypedCose::sign(&ToyMessage::default(), Header::default(), &key, true)
        .await
        .unwrap();
    unsupported.as_mut().protected.header.alg = Some(coset::Algorithm::Assigned(coset::iana::Algorithm::ES384));
    assert!(matches!(
        unsupported.verify(key.verifying_key()),
        Err(CoseError::UnsupportedAlgorithm(_))
    ));
}

#[tokio::test]
async fn reads_unprotected_header_parameters() {
    let key = SigningKey::random(&mut OsRng);
    let header = HeaderBuilder::new()
        .value(42, 0.into())
        .text_value("Hello".to_owned(), "World".into())
        .build();
    let cose = TypedCose::sign(&ToyMessage::default(), header, &key, true)
        .await
        .unwrap();

    assert_eq!(
        cose.unprotected_header_item(&Label::Int(42))
            .unwrap()
            .as_integer()
            .unwrap(),
        0.into()
    );
    assert_eq!(
        cose.unprotected_header_item(&Label::Text("Hello".to_owned()))
            .unwrap()
            .as_text(),
        Some("World")
    );
}

#[tokio::test]
async fn signs_and_verifies_with_certificate() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_issuer_mock().unwrap();
    let cose = TypedCose::sign_with_certificate(&ToyMessage::default(), &key_pair, true)
        .await
        .unwrap();

    assert_eq!(cose.x5chain().unwrap().first(), key_pair.certificate());
    assert_eq!(
        cose.verify_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, Some(CertificateUsage::Mdl),)
            .unwrap(),
        ToyMessage::default()
    );
}

#[tokio::test]
async fn verifies_certificate_chain_with_intermediate() {
    let root_ca =
        Ca::generate_with_intermediate_count(DistinguishedName::create_mock("root-ca"), Default::default(), 1).unwrap();
    let intermediate_ca = root_ca
        .generate_intermediate(
            DistinguishedName::create_mock("intermediate-ca"),
            CertificateConfiguration::with_usage(CertificateUsage::Mdl),
        )
        .unwrap();
    let intermediate_certificate = intermediate_ca.as_borrowing_certificate().unwrap();
    let leaf_key_pair = intermediate_ca
        .generate_key_pair(
            DistinguishedName::create_mock("leaf"),
            CertificateConfiguration::with_usage(CertificateUsage::Mdl),
            NO_SAN,
        )
        .unwrap();
    let header = header_with_x5chain(&vec_nonempty![leaf_key_pair.certificate(), &intermediate_certificate]);
    let cose: TypedCose<_, ToyMessage> =
        TypedCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key(), true)
            .await
            .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors(
            &TrustAnchors::from(&root_ca),
            &TimeGenerator,
            Some(CertificateUsage::Mdl),
        )
        .unwrap(),
        ToyMessage::default()
    );
}

#[tokio::test]
async fn rejects_missing_intermediate_certificate() {
    let root_ca =
        Ca::generate_with_intermediate_count(DistinguishedName::create_mock("root-ca"), Default::default(), 1).unwrap();
    let intermediate_ca = root_ca
        .generate_intermediate(
            DistinguishedName::create_mock("intermediate-ca"),
            CertificateConfiguration::with_usage(CertificateUsage::Mdl),
        )
        .unwrap();
    let leaf_key_pair = intermediate_ca
        .generate_key_pair(
            DistinguishedName::create_mock("leaf"),
            CertificateConfiguration::with_usage(CertificateUsage::Mdl),
            NO_SAN,
        )
        .unwrap();
    let header = header_with_x5chain(&vec_nonempty![leaf_key_pair.certificate()]);
    let cose: TypedCose<_, ToyMessage> =
        TypedCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key(), true)
            .await
            .unwrap();

    assert!(matches!(
        cose.verify_against_trust_anchors(
            &TrustAnchors::from(&root_ca),
            &TimeGenerator,
            Some(CertificateUsage::Mdl),
        ),
        Err(CoseError::Certificate(_))
    ));
}
