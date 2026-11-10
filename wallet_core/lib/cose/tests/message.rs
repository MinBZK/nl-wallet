use std::assert_matches;
use std::marker::PhantomData;

use ciborium::tag::Required;
use cose::COSE_TYP_HEADER_LABEL;
use cose::COSE_X5CHAIN_HEADER_LABEL;
use cose::Cose;
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
use coset::ProtectedHeader;
use coset::SignatureContext;
use coset::iana;
use coset::sig_structure_data;
use crypto::keys::PublicKey;
use crypto::keys::WithVerifyingKey;
use crypto::server_keys::generate::Ca;
use crypto::x509::CertificateConfiguration;
use crypto::x509::CertificateExtendedKeyUsage;
use crypto::x509::CertificateProfile;
use crypto::x509::CertificateVerifyError;
use crypto::x509::DistinguishedName;
use crypto::x509::crl::CertificateCrlVerificationError;
use crypto::x509::crl::mock::MockCertificateCrlVerifier;
use futures::future::FutureExt;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::RandomizedSigner as _;
use rsa::signature::SignatureEncoding as _;
use rsa::signature::Signer as _;
use rstest::rstest;
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

fn create_signed_message(_message: &ToyMessage) -> (TypedCose<CoseSign1, ToyMessage>, PublicKey) {
    let key = SigningKey::generate();
    let public_key = PublicKey::from(*key.verifying_key());

    let cose = TypedCose::sign(&ToyMessage::default(), Header::default(), &key)
        .now_or_never()
        .unwrap()
        .unwrap();
    (cose, public_key)
}

fn create_signed_message_and_modify_algorithm(
    alg: Option<coset::iana::Algorithm>,
) -> (TypedCose<CoseSign1, ToyMessage>, PublicKey) {
    let (mut cose, public_key) = create_signed_message(&ToyMessage::default());
    cose.as_mut().protected.header.alg = alg.map(coset::Algorithm::Assigned);
    (cose, public_key)
}

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

#[test]
fn typed_cose_signs_verifies_and_parses() {
    let payload = ToyMessage::default();
    let (cose, public_key) = create_signed_message(&payload);

    assert!(cose.verify(&public_key).is_ok());
    assert_eq!(cose.dangerous_parse_unverified().unwrap(), payload);
    assert_eq!(cose.verify_and_parse(&public_key).unwrap(), payload);
}

#[test]
fn invalid_signature_is_rejected() {
    let payload = ToyMessage::default();
    let (mut cose, public_key) = create_signed_message(&payload);

    cose.as_mut().signature[0] ^= u8::MAX;
    assert_matches!(
        cose.verify(&public_key),
        Err(CoseError::Esp256SignatureVerificationFailed(_))
    );

    cose.as_mut().signature.pop();
    assert_matches!(
        cose.verify(&public_key),
        Err(CoseError::Esp256SignatureVerificationFailed(_))
    );
}

#[test]
fn missing_algorithm_is_rejected() {
    let (cose, public_key) = create_signed_message_and_modify_algorithm(None);
    assert_matches!(cose.verify(&public_key), Err(CoseError::MissingAlgorithm));
}

#[test]
fn algorithm_mismatch_with_public_key_is_rejected() {
    let (cose, public_key) = create_signed_message_and_modify_algorithm(Some(coset::iana::Algorithm::ESP384));
    assert_matches!(
        cose.verify(&public_key),
        Err(CoseError::MismatchAlgorithmWithPublicKey(a)) if a == coset::iana::Algorithm::ESP384
    );
}

#[test]
fn unsupported_algorithm_is_rejected() {
    let (cose, public_key) = create_signed_message_and_modify_algorithm(Some(coset::iana::Algorithm::ESP512));
    assert_matches!(cose.verify(&public_key), Err(CoseError::UnsupportedAlgorithm(_)));
}

#[test]
fn reads_unprotected_header_parameters() {
    let key = SigningKey::generate();
    let header = HeaderBuilder::new()
        .value(42, 0.into())
        .text_value("Hello".to_owned(), "World".into())
        .build();
    let cose = TypedCose::sign(&ToyMessage::default(), header, &key)
        .now_or_never()
        .unwrap()
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

#[test]
fn signs_and_verifies_with_certificate() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_issuer_mock().unwrap();
    let cose = TypedCose::sign_with_certificate(&ToyMessage::default(), &key_pair)
        .now_or_never()
        .unwrap()
        .unwrap();

    assert_eq!(cose.x5chain().unwrap().first(), key_pair.certificate());
    assert_eq!(
        cose.verify_against_trust_anchors(
            &ca.to_trust_anchors(),
            &TimeGenerator,
            Some(CertificateExtendedKeyUsage::Mdl),
        )
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
        cose.verify_and_parse(&PublicKey::from(key_pair.verifying_key().await.unwrap()))
            .unwrap(),
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

#[rstest]
#[case::without_root(false)]
#[case::with_root(true)]
#[tokio::test]
async fn crl_verification_requires_distribution_point(#[case] include_root: bool) {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
    let crl_verifier = MockCertificateCrlVerifier::new_for_ca(&ca);
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let mut certs = vec_nonempty![key_pair.certificate()];
    if include_root {
        certs.push(&ca_certificate);
    }
    let cose = TypedCose::sign(
        &ToyMessage::default(),
        header_with_x5chain(&certs),
        key_pair.private_key(),
    )
    .await
    .unwrap();

    let error = cose
        .verify_against_trust_anchors_with_crl(&ca.to_trust_anchors(), &crl_verifier, &TimeGenerator, None)
        .await
        .unwrap_err();

    assert_matches!(
        error,
        CoseError::CertificateCrl(CertificateCrlVerificationError::NoCrlDistributionPoint)
    );
}

#[tokio::test]
async fn verifies_trust_anchor_in_x5chain() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_issuer_mock().unwrap();
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors(
            &ca.to_trust_anchors(),
            &TimeGenerator,
            Some(CertificateExtendedKeyUsage::Mdl)
        )
        .unwrap(),
        ToyMessage::default()
    );
    assert_eq!(
        cose.x5chain().unwrap(),
        vec_nonempty![key_pair.certificate().clone(), ca_certificate]
    );
}

#[tokio::test]
async fn crl_verification_accepts_trust_anchor_in_x5chain() {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock_with_crl().unwrap();
    let crl_verifier = MockCertificateCrlVerifier::new_for_ca(&ca);
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors_with_crl(&ca.to_trust_anchors(), &crl_verifier, &TimeGenerator, None)
            .await
            .unwrap(),
        ToyMessage::default()
    );
    assert_eq!(
        cose.x5chain().unwrap(),
        vec_nonempty![key_pair.certificate().clone(), ca_certificate]
    );
}

#[rstest]
#[case::without_crl(false)]
#[case::with_crl(true)]
#[tokio::test]
async fn rejects_trust_anchor_as_leaf(#[case] check_crl: bool) {
    let ca = Ca::generate_mock();
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![&ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, &ca.to_signing_key().unwrap())
        .await
        .unwrap();
    let trust_anchors = ca.to_trust_anchors();

    let result = if check_crl {
        cose.verify_against_trust_anchors_with_crl(
            &trust_anchors,
            &MockCertificateCrlVerifier::default(),
            &TimeGenerator,
            None,
        )
        .await
    } else {
        cose.verify_against_trust_anchors(&trust_anchors, &TimeGenerator, None)
    };
    assert_matches!(
        result,
        Err(CoseError::CertificateVerify(
            CertificateVerifyError::EndEntityIsTrustAnchor
        ))
    );
}

#[rstest]
#[case::without_crl(false)]
#[case::with_crl(true)]
#[tokio::test]
async fn rejects_untrusted_root_in_x5chain(#[case] check_crl: bool) {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock_with_crl().unwrap();
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();
    let other_ca = Ca::generate(DistinguishedName::create_mock("other-ca"), Default::default()).unwrap();
    let trust_anchors = other_ca.to_trust_anchors();

    let result = if check_crl {
        cose.verify_against_trust_anchors_with_crl(
            &trust_anchors,
            &MockCertificateCrlVerifier::default(),
            &TimeGenerator,
            None,
        )
        .await
    } else {
        cose.verify_against_trust_anchors(&trust_anchors, &TimeGenerator, None)
    };
    assert_matches!(
        result,
        Err(CoseError::CertificateVerify(CertificateVerifyError::Verification(_)))
    );
}

#[rstest]
#[case::without_crl(false)]
#[case::with_crl(true)]
#[tokio::test]
async fn rejects_invalid_signature_with_trust_anchor_in_x5chain(#[case] check_crl: bool) {
    let ca = Ca::generate_mock();
    let key_pair = ca.generate_wrpac_verifier_mock_with_crl().unwrap();
    let ca_certificate = ca.as_borrowing_certificate().unwrap();
    let header = header_with_x5chain(&vec_nonempty![key_pair.certificate(), &ca_certificate]);
    let mut cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, key_pair.private_key())
        .await
        .unwrap();
    cose.as_mut().signature[0] ^= u8::MAX;
    let trust_anchors = ca.to_trust_anchors();

    let result = if check_crl {
        cose.verify_against_trust_anchors_with_crl(
            &trust_anchors,
            &MockCertificateCrlVerifier::new_for_ca(&ca),
            &TimeGenerator,
            None,
        )
        .await
    } else {
        cose.verify_against_trust_anchors(&trust_anchors, &TimeGenerator, None)
    };
    assert_matches!(result, Err(CoseError::Esp256SignatureVerificationFailed(_)));
}

#[rstest]
#[case::without_root(false)]
#[case::with_root(true)]
#[tokio::test]
async fn verifies_certificate_chain_with_intermediate(#[case] include_root: bool) {
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
    let root_certificate = root_ca.as_borrowing_certificate().unwrap();
    let mut certs = vec_nonempty![leaf_key_pair.certificate(), &intermediate_certificate];
    if include_root {
        certs.push(&root_certificate);
    }
    let header = header_with_x5chain(&certs);
    let cose: TypedCose<_, ToyMessage> = TypedCose::sign(&ToyMessage::default(), header, leaf_key_pair.private_key())
        .await
        .unwrap();

    assert_eq!(
        cose.verify_against_trust_anchors(
            &root_ca.to_trust_anchors(),
            &TimeGenerator,
            Some(CertificateExtendedKeyUsage::Mdl),
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

    assert_matches!(
        cose.verify_against_trust_anchors(
            &root_ca.to_trust_anchors(),
            &TimeGenerator,
            Some(CertificateExtendedKeyUsage::Mdl),
        ),
        Err(CoseError::CertificateVerify(_))
    );
}

fn sign_cose(message: &ToyMessage, algorithm: iana::Algorithm, signer: impl Fn(Vec<u8>) -> Vec<u8>) -> CoseSign1 {
    let mut payload = Vec::new();
    ciborium::ser::into_writer(message, &mut payload).unwrap();

    let protected_header = ProtectedHeader {
        original_data: None,
        header: HeaderBuilder::new().algorithm(algorithm).build(),
    };
    let signature_data = sig_structure_data(
        SignatureContext::CoseSign1,
        protected_header.clone(),
        None,
        &[],
        &payload,
    );
    let signature = signer(signature_data);

    CoseSign1 {
        signature,
        payload: Some(payload),
        protected: protected_header,
        unprotected: Header::default(),
    }
}

fn verify_cose(cose: CoseSign1, public_key: &PublicKey, message: &ToyMessage) {
    assert!(cose.verify(public_key).is_ok());
    let cose: TypedCose<_, ToyMessage> = TypedCose::from(cose);
    assert!(cose.verify(public_key).is_ok());
    assert_eq!(cose.verify_and_parse(public_key).unwrap(), *message);
}

#[rstest]
#[case(iana::Algorithm::ES256)]
#[case(iana::Algorithm::ESP256)]
fn typed_cose_verifies_all_es256(#[case] algorithm: iana::Algorithm) {
    let key = SigningKey::generate();
    let message = ToyMessage::default();
    let cose = sign_cose(&message, algorithm, |data| {
        p256::ecdsa::signature::Signer::<p256::ecdsa::Signature>::sign(&key, &data).to_vec()
    });

    let public_key = PublicKey::ESP256(*key.verifying_key());
    verify_cose(cose, &public_key, &message);
}

#[rstest]
#[case(iana::Algorithm::ES384)]
#[case(iana::Algorithm::ESP384)]
fn typed_cose_verifies_all_es384(#[case] algorithm: iana::Algorithm) {
    let key = p384::ecdsa::SigningKey::generate();
    let message = ToyMessage::default();
    let cose = sign_cose(&message, algorithm, |data| {
        p256::ecdsa::signature::Signer::<p384::ecdsa::Signature>::sign(&key, &data).to_vec()
    });

    let public_key = PublicKey::ESP384(*key.verifying_key());
    verify_cose(cose, &public_key, &message);
}

#[rstest]
#[case::rsa2048(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa2048.pem")).unwrap(), PhantomData::<sha2::Sha256>, iana::Algorithm::RS256)]
#[case::rsa3072(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa3072.pem")).unwrap(), PhantomData::<sha2::Sha384>, iana::Algorithm::RS384)]
#[case::rsa4096(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa4096.pem")).unwrap(), PhantomData::<sha2::Sha512>, iana::Algorithm::RS512)]
fn typed_cose_verifies_all_rsa_pkcs1v15<D: sha2::Digest + rsa::pkcs8::AssociatedOid>(
    #[case] private_key: rsa::PrivateKey,
    #[case] _digest: PhantomData<D>,
    #[case] algorithm: iana::Algorithm,
) {
    let public_key = private_key.to_public_key();
    let signing_key = rsa::pkcs1v15::SigningKey::<D>::new(private_key);
    let message = ToyMessage::default();
    let cose = sign_cose(&message, algorithm, |data| signing_key.sign(&data).to_vec());

    let public_key = PublicKey::RSA(public_key.try_into().unwrap());
    verify_cose(cose, &public_key, &message);
}

#[rstest]
#[case::rsa2048(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa2048.pem")).unwrap(), PhantomData::<sha2::Sha256>, iana::Algorithm::PS256)]
#[case::rsa3072(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa3072.pem")).unwrap(), PhantomData::<sha2::Sha384>, iana::Algorithm::PS384)]
#[case::rsa4096(rsa::PrivateKey::from_pkcs8_pem(include_str!("../test/rsa4096.pem")).unwrap(), PhantomData::<sha2::Sha512>, iana::Algorithm::PS512)]
fn typed_cose_verifies_all_rsa_pss<D: sha2::digest::Digest + sha2::digest::FixedOutputReset>(
    #[case] private_key: rsa::PrivateKey,
    #[case] _digest: PhantomData<D>,
    #[case] algorithm: iana::Algorithm,
) {
    let public_key = private_key.to_public_key();
    let signing_key = rsa::pss::SigningKey::<D>::new(private_key);
    let message = ToyMessage::default();
    let cose = sign_cose(&message, algorithm, |data| {
        signing_key.sign_with_rng(&mut rand::rng(), &data).to_vec()
    });

    let public_key = PublicKey::RSA(public_key.try_into().unwrap());
    verify_cose(cose, &public_key, &message);
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
        decoded
            .verify_and_parse(&PublicKey::from(*key.verifying_key()))
            .unwrap(),
        ToyMessage::default()
    );

    // Deserialization rejects an untagged `COSE_Sign1`
    let mut untagged = Vec::new();
    ciborium::ser::into_writer(&cose, &mut untagged).unwrap();
    assert!(ciborium::de::from_reader::<TaggedCoseSign1<ToyMessage>, _>(untagged.as_slice()).is_err());
}
