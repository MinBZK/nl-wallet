use assert_matches::assert_matches;

use nl_wallet_mdoc::server_keys::generate::Ca;
use nl_wallet_mdoc::utils::issuer_auth::IssuerRegistration;
use pid_issuer::settings::IssuerAttestationData;
use pid_issuer::settings::IssuerSettings;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::ServerSettings;

fn mock_attestation_data(keypair: KeyPair) -> IssuerAttestationData {
    IssuerAttestationData {
        attestation_type: "com.example.valid".to_string(),
        keypair,
        valid_days: 365,
        copy_count: 4.try_into().unwrap(),
    }
}

#[test]
fn test_settings_success() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
    settings.attestation_settings = vec![mock_attestation_data(issuer_cert_valid.into())].into();

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
    settings.attestation_settings = vec![mock_attestation_data(issuer_cert_valid.into())].into();

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

    settings.attestation_settings = vec![
        mock_attestation_data(issuer_cert_valid.into()),
        IssuerAttestationData {
            attestation_type: "com.example.no_registration".to_string(),
            keypair: issuer_cert_no_registration.into(),
            valid_days: 365,
            copy_count: 4.try_into().unwrap(),
        },
    ]
    .into();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        CertificateVerificationError::IncompleteCertificateType(key) if key == "com.example.no_registration"
    );
}
