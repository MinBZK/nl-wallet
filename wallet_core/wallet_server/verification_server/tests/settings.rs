use std::assert_matches;
use std::collections::HashMap;
use std::num::NonZeroU64;

use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::x509::RelyingParty;
use chrono::Duration;
use chrono::Utc;
use cose::wrprc_cwt::SignedWrprcCwt;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateError;
use dcql::Query;
use jwt::SignedJwt;
use jwt::jades_b_b::JadesbbHeader;
use openid4vc::verifier::SessionTypeReturnUrl;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use server_utils::settings::CertificateVerificationError;
use server_utils::settings::Server;
use server_utils::settings::ServerAuth;
use server_utils::settings::ServerSettings;
use server_utils::settings::Settings;
use server_utils::settings::Storage;
use server_utils::settings::VerifierUseCasesValidationError;
use server_utils::status_list_token_cache_settings::StatusListTokenCacheSettings;
use utils::generator::TimeGenerator;
use verification_server::settings::EphemeralIdSecret;
use verification_server::settings::UseCaseSettings;
use verification_server::settings::VerifierSettings;

const ANNEX_C_EXAMPLE: &str =
    include_str!("../../../lib/attestation_data/examples/spec/registration_certificate_annex_c.json");

#[derive(Clone, Copy)]
enum RegistrationCertificateFormat {
    Jwt,
    Cwt,
}

#[derive(Serialize)]
#[serde(transparent)]
struct RegistrationCertificateFixture(Value);

impl jwt::JwtTyp for RegistrationCertificateFixture {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

fn registration_certificate(
    access_key_pair: &KeyPair,
    wrprc_ca: &Ca,
    format: RegistrationCertificateFormat,
    subject_override: Option<&str>,
) -> Vec<u8> {
    let access_subject =
        RelyingParty::try_from(access_key_pair.certificate().to_distinguished_name().unwrap()).unwrap();
    let RelyingParty::LegalPerson {
        country_name,
        organization_name,
        organization_identifier,
        ..
    } = access_subject
    else {
        panic!("test WRPAC should represent a legal person")
    };

    let now = Utc::now();
    let mut payload: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
    payload["id"] = json!("wrprc-settings-test");
    payload["sub"] = json!(subject_override.unwrap_or(&organization_identifier));
    payload["sub_ln"] = json!(organization_name);
    payload["country"] = json!(country_name);
    payload["iat"] = json!(now.timestamp());
    payload["exp"] = json!((now + Duration::days(30)).timestamp());
    payload["status"] = json!({
        "idx": "0",
        "uri": "https://example.com/statuslists/1",
    });

    let signer = wrprc_ca.generate_wrpac_verifier_mock().unwrap();
    let payload = RegistrationCertificateFixture(payload);
    match format {
        RegistrationCertificateFormat::Jwt => futures::executor::block_on(
            SignedJwt::<_, JadesbbHeader>::sign_with_iat(&payload, &signer, &TimeGenerator),
        )
        .unwrap()
        .to_string()
        .into_bytes(),
        RegistrationCertificateFormat::Cwt => {
            futures::executor::block_on(SignedWrprcCwt::sign_with_certificate(&payload, &signer, &TimeGenerator))
                .unwrap()
                .to_vec()
                .unwrap()
        }
    }
}

fn to_use_case(key_pair: KeyPair, registration_certificate: &[u8]) -> UseCaseSettings {
    let registration_certificate =
        RegistrationCertificateEnvelope::try_from(registration_certificate).expect("valid registration certificate");

    UseCaseSettings {
        session_type_return_url: SessionTypeReturnUrl::Both,
        key_pair: key_pair.into(),
        registration_certificate,
        dcql_query: None,
        return_url_template: None,
        disclosure_base_deep_link: None,
        accept_undetermined_revocation_status: false,
    }
}

fn unauthorized_query() -> Query {
    serde_json::from_value(json!({
        "credentials": [{
            "id": "unauthorized",
            "format": "mso_mdoc",
            "meta": { "doctype_value": "com.example.unauthorized" },
            "claims": [{ "path": ["com.example.unauthorized", "claim"] }]
        }]
    }))
    .unwrap()
}

fn default_settings() -> VerifierSettings {
    let server_settings = Settings {
        wallet_server: Server {
            ip: "127.0.0.1".parse().unwrap(),
            port: 8001,
        },
        internal_server: ServerAuth::InternalEndpoint(Server {
            ip: "127.0.0.1".parse().unwrap(),
            port: 8002,
        }),
        log_requests: false,
        structured_logging: false,
        storage: Storage {
            url: "memory://".parse().unwrap(),
            expiration_minutes: NonZeroU64::new(10).unwrap(),
            successful_deletion_minutes: NonZeroU64::new(10).unwrap(),
            failed_deletion_minutes: NonZeroU64::new(10).unwrap(),
        },
        issuer_trust_anchors: TrustAnchors::empty(),
        wrpac_trust_anchors: TrustAnchors::empty(),
        wrprc_trust_anchors: TrustAnchors::empty(),
        hsm: None,
    };

    VerifierSettings {
        usecases: HashMap::new().into(),
        ephemeral_id_secret: EphemeralIdSecret::try_from(vec![0; 32]).unwrap(),
        allow_origins: None,
        public_url: "https://verification.example.com/".parse().unwrap(),
        universal_link_base_url: "walletdebuginteraction://wallet.example.com/".parse().unwrap(),
        wallet_client_ids: vec!["https://wallet.example.com".to_string()],
        extending_vct_values: None,
        status_list_token_cache_settings: StatusListTokenCacheSettings::default(),
        server_settings,
    }
}

#[test]
fn test_settings_success() {
    let mut settings = default_settings();

    let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate WRPAC CA");
    let wrpac_cert_valid = wrpac_ca
        .generate_wrpac_verifier_mock()
        .expect("generate valid wrpac cert");
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate =
        registration_certificate(&wrpac_cert_valid, &wrprc_ca, RegistrationCertificateFormat::Jwt, None);

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert(
        "valid".to_string(),
        to_use_case(wrpac_cert_valid, &registration_certificate),
    );

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca);
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_no_wrpac_trust_anchors() {
    let mut settings = default_settings();

    let wrpac_ca = Ca::generate_wrpac_mock_ca().expect("generate WRPAC CA");
    let wrpac_cert_valid = wrpac_ca
        .generate_wrpac_verifier_mock()
        .expect("generate valid wrpac cert");
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate =
        registration_certificate(&wrpac_cert_valid, &wrprc_ca, RegistrationCertificateFormat::Jwt, None);

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert(
        "valid".to_string(),
        to_use_case(wrpac_cert_valid, &registration_certificate),
    );

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::empty();
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        VerifierUseCasesValidationError::Certificate(CertificateVerificationError::MissingTrustAnchors)
    );
}

#[test]
fn test_settings_wrong_wrpac_ca() {
    let mut settings = default_settings();

    let wrpac_ca_trusted = Ca::generate_wrpac_mock_ca().expect("generate trusted WRPAC CA");
    let wrpac_ca_wrong = Ca::generate_wrpac_mock_ca().expect("generate wrong WRPAC CA");
    let wrpac_cert_wrong = wrpac_ca_wrong
        .generate_wrpac_verifier_mock()
        .expect("generate wrong WRPAC cert");
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate =
        registration_certificate(&wrpac_cert_wrong, &wrprc_ca, RegistrationCertificateFormat::Jwt, None);

    let mut usecases: HashMap<String, UseCaseSettings> = HashMap::new();
    usecases.insert(
        "wrong_ca".to_string(),
        to_use_case(wrpac_cert_wrong, &registration_certificate),
    );

    settings.usecases = usecases.into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca_trusted);
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    let error = settings.validate().expect_err("should fail");
    assert_matches!(
        error,
        VerifierUseCasesValidationError::Certificate(CertificateVerificationError::InvalidCertificate(
            CertificateError::Verification(_), key
        )) if key == "wrong_ca"
    );
}

#[test]
fn test_settings_accepts_cwt_registration_certificate() {
    let mut settings = default_settings();
    let wrpac_ca = Ca::generate_wrpac_mock_ca().unwrap();
    let wrpac = wrpac_ca.generate_wrpac_verifier_mock().unwrap();
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate =
        registration_certificate(&wrpac, &wrprc_ca, RegistrationCertificateFormat::Cwt, None);

    settings.usecases = HashMap::from([("valid".to_string(), to_use_case(wrpac, &registration_certificate))]).into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca);
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    settings.validate().expect("should succeed");
}

#[test]
fn test_settings_rejects_registration_certificate_for_other_access_certificate() {
    let mut settings = default_settings();
    let wrpac_ca = Ca::generate_wrpac_mock_ca().unwrap();
    let wrpac = wrpac_ca.generate_wrpac_verifier_mock().unwrap();
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate = registration_certificate(
        &wrpac,
        &wrprc_ca,
        RegistrationCertificateFormat::Jwt,
        Some("NTRNL-00000000"),
    );

    settings.usecases = HashMap::from([("mismatch".to_string(), to_use_case(wrpac, &registration_certificate))]).into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca);
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    assert_matches!(
        settings.validate(),
        Err(VerifierUseCasesValidationError::InvalidRegistrationCertificate { use_case_id, .. })
            if use_case_id == "mismatch"
    );
}

#[test]
fn test_settings_rejects_dcql_query_not_authorized_by_registration_certificate() {
    let mut settings = default_settings();
    let wrpac_ca = Ca::generate_wrpac_mock_ca().unwrap();
    let wrpac = wrpac_ca.generate_wrpac_verifier_mock().unwrap();
    let wrprc_ca = Ca::generate_mock();
    let registration_certificate =
        registration_certificate(&wrpac, &wrprc_ca, RegistrationCertificateFormat::Jwt, None);
    let mut use_case = to_use_case(wrpac, &registration_certificate);
    use_case.dcql_query = Some(unauthorized_query());

    settings.usecases = HashMap::from([("unauthorized".to_string(), use_case)]).into();
    settings.server_settings.wrpac_trust_anchors = TrustAnchors::from(&wrpac_ca);
    settings.server_settings.wrprc_trust_anchors = TrustAnchors::from(&wrprc_ca);

    assert_matches!(
        settings.validate(),
        Err(VerifierUseCasesValidationError::UnauthorizedDcqlQuery { use_case_id, .. })
            if use_case_id == "unauthorized"
    );
}
