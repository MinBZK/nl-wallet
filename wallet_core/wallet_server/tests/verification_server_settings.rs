#![cfg(all(feature = "disclosure", not(feature = "issuance")))]

use std::collections::HashMap;

use assert_matches::assert_matches;

use nl_wallet_mdoc::{
    server_keys::KeyPair,
    utils::{reader_auth::ReaderRegistration, x509::CertificateError},
};
use openid4vc::verifier::SessionTypeReturnUrl;
use wallet_server::settings::{CertificateVerificationError, Settings, VerifierUseCase};

fn to_use_case(key_pair: KeyPair) -> VerifierUseCase {
    VerifierUseCase {
        session_type_return_url: SessionTypeReturnUrl::Both,
        key_pair: key_pair.try_into().unwrap(),
    }
}

#[test]
fn test_settings_no_registration() {
    let mut settings =
        Settings::new_custom("verification_server.toml", "verification_server").expect("default settings");

    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_cert_no_registration = reader_ca
        .generate_reader_mock(None)
        .expect("generate reader cert without reader registration");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("no_registration".to_string(), to_use_case(reader_cert_no_registration));

    settings.verifier.usecases = usecases.into();
    settings.verifier.reader_trust_anchors = Some(vec![reader_ca.certificate().clone()]);

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::IncompleteCertificateType(key) if key == "no_registration");
}

#[test]
fn test_settings_wrong_ca() {
    let mut settings =
        Settings::new_custom("verification_server.toml", "verification_server").expect("default settings");

    let issuer_ca = KeyPair::generate_issuer_mock_ca().expect("generate issuer CA");
    let reader_ca = KeyPair::generate_reader_mock_ca().expect("generate reader CA");

    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_cert_wrong_ca = issuer_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate reader cert on issuer CA");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("wrong_ca".to_string(), to_use_case(reader_cert_wrong_ca));

    settings.verifier.usecases = usecases.into();
    settings.verifier.reader_trust_anchors = Some(vec![reader_ca.certificate().clone()]);

    let error = settings.verify_key_pairs().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "wrong_ca");
}
