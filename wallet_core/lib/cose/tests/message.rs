use ciborium::tag::Required;
use cose::COSE_TYP_HEADER_LABEL;
use cose::COSE_X5CHAIN_HEADER_LABEL;
use cose::CoseError;
use cose::CoseTyp;
use cose::TaggedCoseSign1;
use cose::TypedCose;
use cose::header_with_x5chain;
use coset::CoseSign1;
use coset::CoseSign1Builder;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::iana;
use crypto::server_keys::generate::Ca;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateProfile;
use crypto::x509::CertificateUsage;
use crypto::x509::CertificateVerifyError;
use crypto::x509::DistinguishedName;
use crypto::x509::crl::CertificateCrlVerificationError;
use crypto::x509::crl::mock::MockCertificateCrlVerifier;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
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

impl CoseTyp for ToyMessage {
    const TYP: &'static str = "application/toy+cwt";
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
    let key = SigningKey::generate();
    let payload = ToyMessage::default();
    let cose = TypedCose::sign(&payload, Header::default(), &key).await.unwrap();

    cose.verify(key.verifying_key()).unwrap();
    assert_eq!(cose.verify_and_parse(key.verifying_key()).unwrap(), payload);
    assert_eq!(cose.dangerous_parse_unverified().unwrap(), payload);
}

#[tokio::test]
async fn invalid_signature_is_rejected() {
    let key = SigningKey::generate();
    let mut cose = TypedCose::sign(&ToyMessage::default(), Header::default(), &key)
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
    let key = SigningKey::generate();

    let mut missing = TypedCose::sign(&ToyMessage::default(), Header::default(), &key)
        .await
        .unwrap();
    missing.as_mut().protected.header.alg = None;
    assert!(matches!(
        missing.verify(key.verifying_key()),
        Err(CoseError::MissingAlgorithm)
    ));

    let mut unsupported = TypedCose::sign(&ToyMessage::default(), Header::default(), &key)
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
    let key = SigningKey::generate();
    let header = HeaderBuilder::new()
        .value(42, 0.into())
        .text_value("Hello".to_owned(), "World".into())
        .build();
    let cose = TypedCose::sign(&ToyMessage::default(), header, &key).await.unwrap();

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
    let cose = TypedCose::sign_with_certificate(&ToyMessage::default(), &key_pair)
        .await
        .unwrap();

    assert_eq!(cose.x5chain().unwrap().first(), key_pair.certificate());
    assert_eq!(
        cose.verify_against_trust_anchors(&ca.to_trust_anchors(), &TimeGenerator, Some(CertificateUsage::Mdl),)
            .unwrap(),
        ToyMessage::default()
    );
}

#[tokio::test]
async fn signs_with_typ_and_certificate_in_protected_header() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_issuer_mock().unwrap();
    let cose = TypedCose::sign_with_typ_and_certificate(&ToyMessage::default(), &key_pair)
        .await
        .unwrap();

    let protected_header = cose.protected_header();
    let protected_value = |label: i64| {
        protected_header
            .rest
            .iter()
            .find(|(candidate, _)| candidate == &Label::Int(label))
            .map(|(_, value)| value)
    };
    assert_eq!(
        protected_header.alg,
        Some(coset::Algorithm::Assigned(iana::Algorithm::ES256))
    );
    assert_eq!(
        protected_value(COSE_TYP_HEADER_LABEL).and_then(|value| value.as_text()),
        Some("application/toy+cwt")
    );
    assert_eq!(
        protected_value(COSE_X5CHAIN_HEADER_LABEL).and_then(|value| value.as_bytes()),
        Some(&key_pair.certificate().to_vec())
    );
    assert_eq!(cose.as_ref().unprotected, Header::default());

    assert_eq!(
        cose.verify_and_parse(key_pair.private_key().verifying_key()).unwrap(),
        ToyMessage::default()
    );
}

#[tokio::test]
async fn verifies_certificate_and_crl() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock_with_crl().unwrap();
    let crl_verifier = MockCertificateCrlVerifier::new_for_ca(&ca);
    let cose = TypedCose::sign_with_certificate(&ToyMessage::default(), &key_pair)
        .await
        .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors_with_crl(&ca.to_trust_anchors(), &crl_verifier, &TimeGenerator, None,)
            .await
            .unwrap(),
        ToyMessage::default()
    );
}

#[tokio::test]
async fn crl_verification_requires_distribution_point() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
    let crl_verifier = MockCertificateCrlVerifier::new_for_ca(&ca);
    let cose = TypedCose::sign_with_certificate(&ToyMessage::default(), &key_pair)
        .await
        .unwrap();

    let error = cose
        .verify_against_trust_anchors_with_crl(&ca.to_trust_anchors(), &crl_verifier, &TimeGenerator, None)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        CoseError::CertificateCrl(CertificateCrlVerificationError::NoCrlDistributionPoint)
    ));
}

#[tokio::test]
async fn rejects_trust_anchor_in_x5chain() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_issuer_mock().unwrap();
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();

    assert!(matches!(
        cose.verify_against_trust_anchors(&ca.to_trust_anchors(), &TimeGenerator, Some(CertificateUsage::Mdl)),
        Err(CoseError::CertificateVerify(CertificateVerifyError::TrustAnchorInChain))
    ));
}

#[tokio::test]
async fn crl_verification_rejects_trust_anchor_in_x5chain() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock_with_crl().unwrap();
    let crl_verifier = MockCertificateCrlVerifier::new_for_ca(&ca);
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();

    assert!(matches!(
        cose.verify_against_trust_anchors_with_crl(&ca.to_trust_anchors(), &crl_verifier, &TimeGenerator, None)
            .await,
        Err(CoseError::CertificateVerify(CertificateVerifyError::TrustAnchorInChain))
    ));
}

#[tokio::test]
async fn verifies_certificate_chain_with_intermediate() {
    let root_ca =
        Ca::generate_with_intermediate_count(DistinguishedName::create_mock("root-ca"), Default::default(), 1).unwrap();
    let intermediate_ca = root_ca
        .generate_intermediate(
            DistinguishedName::create_mock("intermediate-ca"),
            CertificateConfiguration::with_profile(CertificateProfile::Issuer),
        )
        .unwrap();
    let intermediate_certificate = intermediate_ca.as_borrowing_certificate().unwrap();
    let leaf_key_pair = intermediate_ca
        .generate_key_pair(
            DistinguishedName::create_mock("leaf"),
            CertificateConfiguration::with_profile(CertificateProfile::Issuer),
        )
        .unwrap();
    let header = header_with_x5chain(&vec_nonempty![leaf_key_pair.certificate(), &intermediate_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key())
        .await
        .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors(&root_ca.to_trust_anchors(), &TimeGenerator, Some(CertificateUsage::Mdl),)
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
            CertificateConfiguration::with_profile(CertificateProfile::Issuer),
        )
        .unwrap();
    let leaf_key_pair = intermediate_ca
        .generate_key_pair(
            DistinguishedName::create_mock("leaf"),
            CertificateConfiguration::with_profile(CertificateProfile::Issuer),
        )
        .unwrap();
    let header = header_with_x5chain(&vec_nonempty![leaf_key_pair.certificate()]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key())
        .await
        .unwrap();

    assert!(matches!(
        cose.verify_against_trust_anchors(&root_ca.to_trust_anchors(), &TimeGenerator, Some(CertificateUsage::Mdl),),
        Err(CoseError::CertificateVerify(_))
    ));
}

#[tokio::test]
async fn tagged_cose_sign1_requires_tag() {
    let key = SigningKey::generate();
    let cose = TypedCose::sign(&ToyMessage::default(), Header::default(), &key)
        .await
        .unwrap();

    // Serialization emits CBOR tag 18 (0xd2) in front of the `COSE_Sign1` array (0x84)
    let tagged_cose: TaggedCoseSign1<ToyMessage> = Required(cose.clone());
    let mut tagged = Vec::new();
    ciborium::ser::into_writer(&tagged_cose, &mut tagged).unwrap();
    assert_eq!(&tagged[..2], &[0xd2, 0x84]);

    let Required(decoded): TaggedCoseSign1<ToyMessage> = ciborium::de::from_reader(tagged.as_slice()).unwrap();
    assert_eq!(
        decoded.verify_and_parse(key.verifying_key()).unwrap(),
        ToyMessage::default()
    );

    // Deserialization rejects an untagged `COSE_Sign1`
    let mut untagged = Vec::new();
    ciborium::ser::into_writer(&cose, &mut untagged).unwrap();
    assert!(ciborium::de::from_reader::<TaggedCoseSign1<ToyMessage>, _>(untagged.as_slice()).is_err());
}
