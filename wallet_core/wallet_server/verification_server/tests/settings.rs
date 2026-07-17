use std::assert_matches;
use std::collections::HashMap;

use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateError;
use openid4vc::verifier::SessionTypeReturnUrl;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::ServerSettings;
use verification_server::settings::UseCaseSettings;
use verification_server::settings::VerifierSettings;

fn to_use_case(key_pair: KeyPair) -> UseCaseSettings {
    UseCaseSettings {
        session_type_return_url: SessionTypeReturnUrl::Both,
        key_pair: key_pair.into(),
        dcql_query: None,
        return_url_template: None,
        disclosure_base_deep_link: None,
        accept_undetermined_revocation_status: false,
    }
}

#[test]
fn test_settings_success() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate WRPAC CA");
    let wrpac_cert_valid = wrpac_ca
        .generate_wrpac_verifier_mock()
        .expect("generate valid wrpac cert");

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(wrpac_cert_valid));

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca);

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_wrpac_trust_anchors() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate WRPAC CA");
    let wrpac_cert_valid = wrpac_ca
        .generate_wrpac_verifier_mock()
        .expect("generate valid wrpac cert");

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert("valid".to_string(), to_use_case(wrpac_cert_valid));

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::empty();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::MissingTrustAnchors);
}

#[test]
fn test_settings_wrong_wrpac_ca() {
    let mut settings =
        VerifierSettings::new("verification_server.toml", "verification_server").expect("default settings");

    let wrpac_ca_trusted = Ca::generate_wrpac_mock_ca().expect("generate trusted WRPAC CA");
    let wrpac_ca_wrong = Ca::generate_wrpac_mock_ca().expect("generate wrong WRPAC CA");
    let wrpac_cert_wrong = wrpac_ca_wrong
        .generate_wrpac_verifier_mock()
        .expect("generate wrong WRPAC cert");

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert("wrong_ca".to_string(), to_use_case(wrpac_cert_wrong));

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca_trusted);

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        CertificateVerificationError::InvalidCertificate(CertificateError::Verification(_), key) if key == "wrong_ca"
    );
}
