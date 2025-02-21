use assert_matches::assert_matches;

use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use pid_issuer::settings::IssuerSettings;
use wallet_server::settings::CertificateVerificationError;
use wallet_server::settings::ServerSettings;

#[test]
fn test_settings_success() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
    settings.private_keys.clear();
    settings
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.into());

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_issuer_trust_anchors() {
    use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;

    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    settings.server_settings.issuer_trust_anchors = vec![];
    settings.private_keys.clear();
    settings
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.into());

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, CertificateVerificationError::MissingTrustAnchors);
}

#[test]
fn test_settings_no_issuer_registration() {
    use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;

    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_no_registration = issuer_ca
        .generate_issuer_mock(None)
        .expect("generate issuer cert without issuer registration");

    settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];

    settings.private_keys.clear();
    settings
        .private_keys
        .insert("com.example.valid".to_string(), issuer_cert_valid.into());
    settings.private_keys.insert(
        "com.example.no_registration".to_string(),
        issuer_cert_no_registration.into(),
    );

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        CertificateVerificationError::IncompleteCertificateType(key) if key == "com.example.no_registration"
    );
}
