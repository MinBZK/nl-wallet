use std::collections::HashMap;

use assert_matches::assert_matches;

use mdoc::server_keys::generate::Ca;
use mdoc::utils::issuer_auth::IssuerRegistration;
use pid_issuer::settings::AttestationTypeConfigSettings;
use pid_issuer::settings::IssuerSettings;
use pid_issuer::settings::IssuerSettingsError;
use sd_jwt::metadata::TypeMetadata;
use sd_jwt::metadata::UncheckedTypeMetadata;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::KeyPair;
use server_utils::settings::ServerSettings;
use wallet_common::urls::HttpsUri;

fn mock_attestation_data(keypair: KeyPair) -> HashMap<String, AttestationTypeConfigSettings> {
    HashMap::from([(
        "com.example.pid".to_string(),
        AttestationTypeConfigSettings {
            keypair,
            valid_days: 365,
            copy_count: 4.try_into().unwrap(),
            certificate_san: None,
        },
    )])
}

#[test]
fn test_settings_success() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
    settings.attestation_settings = mock_attestation_data(issuer_cert_valid.into()).into();

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_issuer_trust_anchors() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");

    settings.server_settings.issuer_trust_anchors = vec![];
    settings.attestation_settings = mock_attestation_data(issuer_cert_valid.into()).into();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        IssuerSettingsError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
    );
}

#[test]
fn test_settings_no_issuer_registration() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_valid = issuer_ca
        .generate_issuer_mock(IssuerRegistration::new_mock().into())
        .expect("generate valid issuer cert");
    let issuer_cert_no_registration = issuer_ca
        .generate_issuer_mock(None)
        .expect("generate issuer cert without issuer registration");

    settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];

    let mut attestation_settings = mock_attestation_data(issuer_cert_valid.into());
    attestation_settings.insert(
        "com.example.no_registration".to_string(),
        AttestationTypeConfigSettings {
            keypair: issuer_cert_no_registration.into(),
            valid_days: 365,
            copy_count: 4.try_into().unwrap(),
            certificate_san: None,
        },
    );
    settings.attestation_settings = attestation_settings.into();

    let no_registration_metadata = UncheckedTypeMetadata {
        vct: "com.example.no_registration".to_string(),
        ..UncheckedTypeMetadata::empty_example()
    };
    let pid_metadata = TypeMetadata::pid_example();

    settings.metadata = HashMap::from([
        (
            no_registration_metadata.vct.clone(),
            serde_json::to_vec(&no_registration_metadata).unwrap(),
        ),
        (
            pid_metadata.as_ref().vct.clone(),
            serde_json::to_vec(&pid_metadata).unwrap(),
        ),
    ]);

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        IssuerSettingsError::CertificateVerification(CertificateVerificationError::IncompleteCertificateType(key))
            if key == "com.example.no_registration"
    );
}

#[test]
fn test_settings_missing_metadata() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    settings.metadata.clear();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, IssuerSettingsError::MissingMetadata { .. });
}

#[test]
fn test_settings_wrong_san_field() {
    let mut settings = IssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let wrong_san: HttpsUri = "https://wrong.san.example.com".parse().unwrap();

    let (typ, attestation_settings) = settings.attestation_settings.as_ref().iter().next().unwrap();
    let mut attestation_settings = attestation_settings.clone();
    attestation_settings.certificate_san = Some(wrong_san.clone());
    settings.attestation_settings = HashMap::from([(typ.clone(), attestation_settings)]).into();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, IssuerSettingsError::CertificateMissingSan { san, .. } if san == wrong_san);
}
