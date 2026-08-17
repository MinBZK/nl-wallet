use attestation_data::registration_certificate::UncheckedRegistrationCertificate;
use attestation_data::x509::RelyingParty;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use cose::wrprc_cwt::SignedWrprcCwt;
use cose::wrprc_cwt::UnverifiedWrprcCwt;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::DistinguishedName;
use jwt::DEFAULT_VALIDATION;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::jades_b_b::JadesbbHeader;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use utils::generator::mock::MockTimeGenerator;

const ANNEX_C_EXAMPLE: &str = include_str!("../examples/spec/registration_certificate_annex_c.json");
const STATUS_LIST_URI: &str = "https://example.com/statuslists/1";

#[derive(Serialize)]
#[serde(transparent)]
struct RegistrationCertificateFixture(Value);

impl jwt::JwtTyp for RegistrationCertificateFixture {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

fn registration_certificate_payload() -> RegistrationCertificateFixture {
    let mut payload: Value = serde_json::from_str(ANNEX_C_EXAMPLE).unwrap();
    payload["id"] = json!("wrprc-example-1");
    payload["status"] = json!({
        "idx": "0",
        "uri": STATUS_LIST_URI,
    });

    RegistrationCertificateFixture(payload)
}

fn access_certificate_subject() -> RelyingParty {
    RelyingParty::try_from(DistinguishedName::new_legal_person(
        "Example Company".to_string(),
        "DE".to_string(),
        "Example Company GmbH".to_string(),
        "LEIXG-529900T8BM49AURSDO55".to_string(),
    ))
    .unwrap()
}

fn validation_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2023, 5, 3, 0, 0, 0).unwrap()
}

#[tokio::test]
async fn verify_and_validate_registration_certificate_from_jades_jwt() {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let signer = ca.generate_wrpac_verifier_mock().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let signed = SignedJwt::<_, JadesbbHeader>::sign_with_iat(&registration_certificate_payload(), &signer, &time)
        .await
        .unwrap();

    let encoded = signed.to_string();
    let unverified: UnverifiedJwt<UncheckedRegistrationCertificate, JadesbbHeader> = encoded.parse().unwrap();
    let (_, payload) = unverified
        .parse_and_verify_against_trust_anchors(&TrustAnchors::from(&ca), &time, None, DEFAULT_VALIDATION.to_owned())
        .unwrap();
    let certificate = payload
        .validate_structure(&access_certificate_subject(), validation_time())
        .unwrap();

    assert_eq!(certificate.payload().id.as_deref(), Some("wrprc-example-1"));
}

#[tokio::test]
async fn verify_and_validate_registration_certificate_from_wrprc_cwt() {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let signer = ca.generate_wrpac_verifier_mock().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let signed = SignedWrprcCwt::sign_with_certificate(&registration_certificate_payload(), &signer, &time)
        .await
        .unwrap();

    let encoded = signed.to_vec().unwrap();
    let unverified = UnverifiedWrprcCwt::<UncheckedRegistrationCertificate>::from_slice(&encoded).unwrap();
    let payload = unverified
        .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &time, None)
        .unwrap()
        .into_payload();
    let certificate = payload
        .validate_structure(&access_certificate_subject(), validation_time())
        .unwrap();

    assert_eq!(certificate.payload().id.as_deref(), Some("wrprc-example-1"));
}
