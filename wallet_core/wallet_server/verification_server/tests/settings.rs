use std::collections::HashMap;

use assert_matches::assert_matches;

use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::server_keys::KeyPair;
use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
use nl_wallet_mdoc::utils::x509::CertificateError;
use openid4vc::verifier::SessionTypeReturnUrl;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::ServerSettings;
use verification_server::settings::VerifierSettings;
use verification_server::settings::VerifierUseCase;

fn to_use_case(key_pair: KeyPair) -> VerifierUseCase {
    VerifierUseCase {
        session_type_return_url: SessionTypeReturnUrl::Both,
        key_pair: key_pair.into(),
    }
}

#[test]
fn test_settings_success() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let reader_ca = Ca::generate_reader_mock_ca().expect("generate reader CA");
    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.as_borrowing_trust_anchor().clone()];

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_reader_trust_anchors() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let reader_ca = Ca::generate_reader_mock_ca().expect("generate reader CA");
    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));

    settings.usecases = usecases.into();
    settings.reader_trust_anchors = vec![];

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::MissingTrustAnchors);
}

#[test]
fn test_settings_no_reader_registration() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let reader_ca = Ca::generate_reader_mock_ca().expect("generate reader CA");
    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_cert_no_registration = reader_ca
        .generate_reader_mock(None)
        .expect("generate reader cert without reader registration");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("no_registration".to_string(), to_use_case(reader_cert_no_registration));

    settings.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.as_borrowing_trust_anchor().clone()];

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::IncompleteCertificateType(key) if key == "no_registration");
}

#[test]
fn test_settings_wrong_reader_ca() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let reader_ca = Ca::generate_reader_mock_ca().expect("generate reader CA");
    let reader_cert_valid = reader_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate valid reader cert");
    let reader_wrong_ca = Ca::generate_reader_mock_ca().expect("generate wrong reader CA");
    let reader_cert_wrong_ca = reader_wrong_ca
        .generate_reader_mock(ReaderRegistration::new_mock().into())
        .expect("generate reader cert on wrong CA");

    let mut usecases: HashMap<String, VerifierUseCase> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(reader_cert_valid));
    usecases.insert("wrong_ca".to_string(), to_use_case(reader_cert_wrong_ca));

    settings.usecases = usecases.into();
    settings.reader_trust_anchors = vec![reader_ca.as_borrowing_trust_anchor().clone()];

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "wrong_ca"
    );
}
