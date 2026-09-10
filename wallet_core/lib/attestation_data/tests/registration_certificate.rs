use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::mock::ANNEX_C_EXAMPLE;
use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
use attestation_data::registration_certificate::mock::STATUS_LIST_URI;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::RelyingParty;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use cose::wrprc_cwt::SignedWrprcCwt;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::DistinguishedName;
use jwt::SignedJwt;
use jwt::jades_b_b::JadesbbHeader;
use rstest::rstest;
use serde_json::Value;
use serde_json::json;
use utils::generator::mock::MockTimeGenerator;

#[derive(Clone, Copy)]
enum EnvelopeFormat {
    Jwt,
    Cwt,
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

async fn signed_registration_certificate(format: EnvelopeFormat, ca: &Ca, time: &MockTimeGenerator) -> Vec<u8> {
    let signer = ca.generate_wrpac_verifier_mock().unwrap();

    match format {
        EnvelopeFormat::Jwt => {
            SignedJwt::<_, JadesbbHeader>::sign_with_iat(&registration_certificate_payload(), &signer, time)
                .await
                .unwrap()
                .to_string()
                .into_bytes()
        }
        EnvelopeFormat::Cwt => {
            SignedWrprcCwt::sign_with_certificate(&registration_certificate_payload(), &signer, time)
                .await
                .unwrap()
                .to_vec()
                .unwrap()
        }
    }
}

#[rstest]
#[case::jwt(EnvelopeFormat::Jwt)]
#[case::cwt(EnvelopeFormat::Cwt)]
#[tokio::test]
async fn parse_registration_certificate_without_verifying(#[case] format: EnvelopeFormat) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let encoded = signed_registration_certificate(format, &ca, &time).await;

    let envelope = RegistrationCertificateEnvelope::try_from(encoded.as_slice()).unwrap();

    assert!(matches!(
        (format, &envelope),
        (EnvelopeFormat::Jwt, RegistrationCertificateEnvelope::Jwt(_))
            | (EnvelopeFormat::Cwt, RegistrationCertificateEnvelope::Cwt(_))
    ));
    assert_eq!(envelope.to_vec().unwrap(), encoded);
}

#[rstest]
#[case::jwt(EnvelopeFormat::Jwt)]
#[case::cwt(EnvelopeFormat::Cwt)]
#[tokio::test]
async fn reject_untrusted_registration_certificate_after_parsing(#[case] format: EnvelopeFormat) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let encoded = signed_registration_certificate(format, &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::try_from(encoded.as_slice()).unwrap();
    let untrusted_ca = Ca::generate_wrpac_mock_ca().unwrap();

    let error = verify_registration_certificate_envelope(&envelope, &TrustAnchors::from(&untrusted_ca), &time)
        .err()
        .expect("an envelope signed by an untrusted CA should be rejected");

    assert!(matches!(
        (format, error),
        (
            EnvelopeFormat::Jwt,
            attestation_data::registration_certificate::RegistrationCertificateEnvelopeError::Jwt(_)
        ) | (
            EnvelopeFormat::Cwt,
            attestation_data::registration_certificate::RegistrationCertificateEnvelopeError::Cwt(_)
        )
    ));
}

#[rstest]
#[case::jwt(EnvelopeFormat::Jwt)]
#[case::cwt(EnvelopeFormat::Cwt)]
#[tokio::test]
async fn verify_and_validate_registration_certificate(#[case] format: EnvelopeFormat) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let encoded = signed_registration_certificate(format, &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::try_from(encoded.as_slice()).unwrap();
    let payload = verify_registration_certificate_envelope(&envelope, &TrustAnchors::from(&ca), &time)
        .unwrap()
        .into_payload();
    let certificate = payload
        .validate_structure(&access_certificate_subject(), validation_time())
        .unwrap();

    assert_eq!(certificate.payload().id.as_deref(), Some("wrprc-example-1"));
}
