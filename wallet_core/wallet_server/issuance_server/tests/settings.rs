use std::collections::HashMap;

use crypto::server_keys::generate::Ca;
use issuance_server::settings::IssuanceServerSettings;
use issuer_settings::settings::AttestationTypeConfigSettings;
use mdoc::server_keys::generate::mock::generate_issuer_mock;
use mdoc::utils::issuer_auth::IssuerRegistration;
use mdoc::AttestationQualification;
use server_utils::settings::KeyPair;
use server_utils::settings::ServerSettings;

fn mock_attestation_data(keypair: KeyPair) -> HashMap<String, AttestationTypeConfigSettings> {
    HashMap::from([(
        "com.example.pid".to_string(),
        AttestationTypeConfigSettings {
            keypair,
            valid_days: 365,
            copy_count: 4.try_into().unwrap(),
            attestation_qualification: AttestationQualification::PubEAA,
            certificate_san: None,
        },
    )])
}

#[test]
fn test_settings_success() {
    let mut settings =
        IssuanceServerSettings::new("issuance_server.toml", "issuance_server").expect("default settings");

    let issuer_ca = Ca::generate_issuer_mock_ca().expect("generate issuer CA");

    let issuer_cert_valid =
        generate_issuer_mock(&issuer_ca, IssuerRegistration::new_mock().into()).expect("generate valid issuer cert");

    settings.issuer_settings.server_settings.issuer_trust_anchors = vec![issuer_ca.as_borrowing_trust_anchor().clone()];
    settings.issuer_settings.attestation_settings = mock_attestation_data(issuer_cert_valid.into()).into();

    settings.validate().expect("should succeed");
}
