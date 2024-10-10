#![cfg(all(feature = "issuance", not(feature = "disclosure")))]

use assert_matches::assert_matches;

use nl_wallet_mdoc::{
    server_keys::KeyPair,
    utils::{issuer_auth::IssuerRegistration, x509::CertificateError},
};
use wallet_server::settings::{CertificateVerificationError, Settings};

#[test]
fn test_settings_cert_no_registration() {
    let mut settings = Settings::new_custom("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_no_registration = issuer_ca
        .generate_issuer_mock(None)
        .expect("generate issuer cert without issuer registration");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer.issuer_trust_anchors = Some(issuer_trust_anchors);

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());
    settings.issuer.private_keys.insert(
        "com.example.no_registration".to_string(),
        issuer_cert_no_registration.try_into().unwrap(),
    );

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::IncompleteCertificateType(key) if key == "com.example.no_registration");
}

#[test]
fn test_settings_cert_wrong_ca() {
    let mut settings = Settings::new_custom("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_wrong_ca = reader_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate issuer cert on reader CA");

    let issuer_trust_anchors = vec![issuer_ca.certificate().clone()];
    settings.issuer.issuer_trust_anchors = Some(issuer_trust_anchors);

    settings.issuer.private_keys.clear();
    settings
        .issuer
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.try_into().unwrap());
    settings.issuer.private_keys.insert(
        "com.example.wrong_ca".to_string(),
        issuer_cert_wrong_ca.try_into().unwrap(),
    );

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "com.example.wrong_ca");
}
