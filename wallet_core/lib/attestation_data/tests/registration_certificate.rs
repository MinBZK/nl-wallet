use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
use attestation_data::registration_certificate::RegistrationCertificateValidationError;
use attestation_data::registration_certificate::mock::ANNEX_C_EXAMPLE;
use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::RelyingParty;
use chrono::DateTime;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use cose::wrprc_cwt::SignedWrprcCwt;
use crypto::server_keys::generate::Ca;
use crypto::x509::DistinguishedName;
use jwt::SignedJwt;
use jwt::jades_b_b::JadesbbHeader;
use rstest::rstest;
use utils::generator::mock::MockTimeGenerator;

#[derive(Clone, Copy)]
enum EnvelopeFormat {
    Jwt,
    Cwt,
}

fn registration_certificate_payload() -> RegistrationCertificateFixture {
    RegistrationCertificateFixture(serde_json::from_str(ANNEX_C_EXAMPLE).unwrap())
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

async fn signed_registration_certificate(
    format: EnvelopeFormat,
    payload: &RegistrationCertificateFixture,
    ca: &Ca,
    time: &MockTimeGenerator,
) -> Vec<u8> {
    let signer = ca.generate_wrpac_verifier_mock().unwrap();

    match format {
        EnvelopeFormat::Jwt => SignedJwt::<_, JadesbbHeader>::sign_with_iat(payload, &signer, time)
            .await
            .unwrap()
            .to_string()
            .into_bytes(),
        EnvelopeFormat::Cwt => SignedWrprcCwt::sign_with_certificate(payload, &signer, time)
            .await
            .unwrap()
            .to_vec()
            .unwrap(),
    }
}

#[rstest]
#[case::jwt(EnvelopeFormat::Jwt)]
#[case::cwt(EnvelopeFormat::Cwt)]
#[tokio::test]
async fn parse_registration_certificate_without_verifying(#[case] format: EnvelopeFormat) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let encoded = signed_registration_certificate(format, &registration_certificate_payload(), &ca, &time).await;

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
    let encoded = signed_registration_certificate(format, &registration_certificate_payload(), &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::try_from(encoded.as_slice()).unwrap();
    let untrusted_ca = Ca::generate_wrpac_mock_ca().unwrap();

    let error = verify_registration_certificate_envelope(&envelope, &untrusted_ca.to_trust_anchors(), &time)
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
#[case::no_expiry(None)]
#[case::before_expiry(Some(1))]
#[case::at_expiry(Some(0))]
#[case::after_expiry(Some(-1))]
#[tokio::test]
async fn verify_and_validate_registration_certificate(
    #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    #[case] expires_in_seconds: Option<i64>,
) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    // Keep exp in the system clock's future so the JWT library's own expiry check passes. This exercises the
    // shared envelope check against the supplied clock, including the exact expiry boundary, for both formats.
    let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap() + Duration::days(1);
    let time = MockTimeGenerator::new(now);
    let mut payload = registration_certificate_payload();
    payload.0["iat"] = json!((now - Duration::minutes(2)).timestamp());
    if let Some(seconds) = expires_in_seconds {
        payload.0["exp"] = json!((now + Duration::seconds(seconds)).timestamp());
    }
    let encoded = signed_registration_certificate(format, &payload, &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::try_from(encoded.as_slice()).unwrap();
    let verified = verify_registration_certificate_envelope(&envelope, &ca.to_trust_anchors(), &time);
    if let Some(seconds) = expires_in_seconds.filter(|seconds| *seconds <= 0) {
        assert!(matches!(
            verified,
            Err(RegistrationCertificateEnvelopeError::Validity(
                RegistrationCertificateValidationError::Expired { expiration }
            )) if expiration == now + Duration::seconds(seconds)
        ));
        return;
    }

    let payload = verified.unwrap().into_payload();
    let certificate = payload.validate_binding(&access_certificate_subject()).unwrap();

    assert_eq!(certificate.payload().sub, "LEIXG-529900T8BM49AURSDO55");
}
