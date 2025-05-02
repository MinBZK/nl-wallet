use std::collections::HashMap;

use assert_matches::assert_matches;

use attestation::x509::generate::mock::generate_issuer_mock;
use crypto::server_keys::generate::Ca;
use http_utils::urls::HttpsUri;
use issuer_settings::settings::AttestationTypeConfigSettings;
use issuer_settings::settings::IssuerSettingsError;
use pid_issuer::settings::PidIssuerSettings;
use sd_jwt_vc_metadata::UncheckedTypeMetadata;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::ServerSettings;

#[test]
fn test_settings_success() {
    let settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_issuer_trust_anchors() {
    let mut settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    settings.issuer_settings.server_settings.issuer_trust_anchors = vec![];

    assert_matches!(
        settings.validate().expect_err("should fail"),
        IssuerSettingsError::CertificateVerification(CertificateVerificationError::MissingTrustAnchors)
    );
}

#[test]
fn test_settings_no_issuer_registration() {
    let mut settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");
    let issuer_cert_no_registration =
        generate_issuer_mock(&issuer_ca, None).expect("generate issuer cert without issuer registration");

    settings.issuer_settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
    settings.issuer_settings.attestation_settings = HashMap::from([(
        "com.example.no_registration".to_string(),
        AttestationTypeConfigSettings {
            keypair: issuer_cert_no_registration.into(),
            valid_days: 365,
            copy_count: 4.try_into().unwrap(),
            attestation_qualification: Default::default(),
            certificate_san: None,
        },
    )])
    .into();

    let no_registration_metadata = UncheckedTypeMetadata {
        vct: "com.example.no_registration".to_string(),
        ..UncheckedTypeMetadata::empty_example()
    };
    let no_registration_metadata_json = serde_json::to_vec(&no_registration_metadata).unwrap();
    let pid_metadata = UncheckedTypeMetadata::pid_example();
    let pid_metadata_json = serde_json::to_vec(&pid_metadata).unwrap();

    settings.issuer_settings.metadata = HashMap::from([
        (
            no_registration_metadata.vct.clone(),
            (no_registration_metadata, no_registration_metadata_json),
        ),
        (pid_metadata.vct.clone(), (pid_metadata, pid_metadata_json)),
    ]);

    assert_matches!(
        settings.validate().expect_err("should fail"),
        IssuerSettingsError::CertificateVerification(CertificateVerificationError::IncompleteCertificateType(key))
            if key == "com.example.no_registration"
    );
}

#[test]
fn test_settings_wrong_san_field() {
    let mut settings = PidIssuerSettings::new("pid_issuer.toml", "pid_issuer").expect("default settings");

    let wrong_san: HttpsUri = "https://wrong.san.example.com".parse().unwrap();

    let (typ, attestation_settings) = settings
        .issuer_settings
        .attestation_settings
        .as_ref()
        .iter()
        .next()
        .unwrap();
    let mut attestation_settings = attestation_settings.clone();
    attestation_settings.certificate_san = Some(wrong_san.clone());
    settings.issuer_settings.attestation_settings = HashMap::from([(typ.clone(), attestation_settings)]).into();

    let error = settings.validate().expect_err("should fail");
    assert_matches!(error, IssuerSettingsError::CertificateMissingSan { san, .. } if san == wrong_san);
}
