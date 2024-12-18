use assert_matches::assert_matches;
use std::collections::HashMap;

use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::x509::Certificate;
use nl_wallet_mdoc::utils::x509::CertificateError;
use openid4vc::verifier::SessionTypeReturnUrl;
use wallet_common::trust_anchor::DerTrustAnchor;
use wallet_server::settings::CertificateVerificationError;
use wallet_server::settings::Settings;
use wallet_server::settings::VerifierUseCase;

fn to_use_case(key_pair: KeyPair) -> VerifierUseCase {
    VerifierUseCase {
        session_type_return_url: SessionTypeReturnUrl::Both,
        key_pair: key_pair.try_into().unwrap(),
    }
}

fn certificates_to_trust_anchors(trust_anchors: &[Certificate]) -> Result<Vec<DerTrustAnchor>, CertificateError> {
    trust_anchors
        .iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, CertificateError>>()
}

#[test]
fn test_settings_success() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    settings.verify_key_pairs().expect("should succeed");
}

#[test]
fn test_settings_no_issuer_trust_anchors() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let issuer_trust_anchors = vec![];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::MissingTrustAnchors);
}

#[test]
fn test_settings_no_reader_trust_anchors() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::MissingTrustAnchors);
}

#[test]
fn test_settings_no_reader_registration() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_cert_no_registration = reader_ca
        .generate_reader_mock(None)
        .expect("generate reader cert without reader registration");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("no_registration".to_string(), to_use_case(reader_cert_no_registration));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::IncompleteCertificateType(key) if key == "no_registration");
}

#[test]
fn test_settings_wrong_reader_ca() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_cert_wrong_ca = issuer_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate reader cert on issuer CA");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("wrong_ca".to_string(), to_use_case(reader_cert_wrong_ca));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "wrong_ca");
}

#[test]
fn test_settings_no_issuer_registration() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_no_registration = issuer_ca
        .generate_issuer_mock(None)
        .expect("generate issuer cert without issuer registration");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());
    settings.issuer.private_keys.insert(
        "com.example.no_registration".to_string(),
        issuer_cert_no_registration.try_into().unwrap(),
    );

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::IncompleteCertificateType(key) if key == "com.example.no_registration");
}

#[test]
fn test_settings_wrong_issuer_ca() {
    let mut settings =
        Settings::new_custom("ws_integration_test.toml", "ws_integration_test").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_wrong_ca = reader_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate issuer cert on reader CA");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer_trust_anchors = certificates_to_trust_anchors(&issuer_trust_anchors).unwrap();

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());
    settings.issuer.private_keys.insert(
        "com.example.wrong_ca".to_string(),
        issuer_cert_wrong_ca.try_into().unwrap(),
    );

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.verifier.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.certificate().try_into().unwrap()];

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "com.example.wrong_ca");
}
