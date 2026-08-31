use std::assert_matches;

use dcql::Query;
use issuance_server::settings::IssuanceServerSettings;
use issuance_server::settings::IssuanceServerSettingsValidationError;
use issuance_server::settings::VerifierSettingsValidationError;
use serde_json::json;
use server_utils::settings::ServerSettings;

const UNIVERSITY_USE_CASE_ID: &str = "university_mdoc";

fn settings() -> IssuanceServerSettings {
    IssuanceServerSettings::new("issuance_server.toml", "issuance_server").expect("default settings")
}

#[test]
fn test_settings_success() {
    let settings = settings();

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_requires_registration_certificate() {
    let mut settings = settings();
    settings
        .verifier_settings
        .disclosure_settings
        .retain(|use_case_id, _| use_case_id == UNIVERSITY_USE_CASE_ID);
    settings
        .verifier_settings
        .disclosure_settings
        .get_mut(UNIVERSITY_USE_CASE_ID)
        .unwrap()
        .registration_certificate = None;

    assert_matches!(
        settings.validate(),
        Err(IssuanceServerSettingsValidationError::Verifier(
            VerifierSettingsValidationError::MissingRegistrationCertificate { use_case_id }
        )) if use_case_id == UNIVERSITY_USE_CASE_ID
    );
}

#[test]
fn test_settings_rejects_dcql_query_not_authorized_by_registration_certificate() {
    let mut settings = settings();
    settings
        .verifier_settings
        .disclosure_settings
        .retain(|use_case_id, _| use_case_id == UNIVERSITY_USE_CASE_ID);
    settings
        .verifier_settings
        .disclosure_settings
        .get_mut(UNIVERSITY_USE_CASE_ID)
        .unwrap()
        .dcql_query = serde_json::from_value::<Query>(json!({
        "credentials": [{
            "id": "unauthorized",
            "format": "mso_mdoc",
            "meta": { "doctype_value": "com.example.unauthorized" },
            "claims": [{ "path": ["com.example.unauthorized", "claim"] }]
        }]
    }))
    .unwrap();

    assert_matches!(
        settings.validate(),
        Err(IssuanceServerSettingsValidationError::Verifier(
            VerifierSettingsValidationError::UnauthorizedDcqlQuery { use_case_id, .. }
        )) if use_case_id == UNIVERSITY_USE_CASE_ID
    );
}
