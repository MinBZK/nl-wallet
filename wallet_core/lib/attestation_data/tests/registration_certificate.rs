use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use attestation_data::registration_certificate::RegistrationCertificateEnvelopeError;
use attestation_data::registration_certificate::mock::ANNEX_C_EXAMPLE;
use attestation_data::registration_certificate::mock::RegistrationCertificateFixture;
use attestation_data::registration_certificate::verify_registration_certificate_envelope;
use attestation_data::x509::RelyingParty;
use chrono::DateTime;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use cose::wrprc_cwt::SignedWrprcCwt;
use cose::wrprc_cwt::WrprcCwtError;
use crypto::server_keys::generate::Ca;
use crypto::x509::DistinguishedName;
use jwt::SignedJwt;
use jwt::error::JwtVerifyError;
use jwt::error::JwtX5cVerifyError;
use jwt::error::ValidationErrorKind;
use jwt::jades_b_b::JadesbbHeader;
use rstest::rstest;
use serde_json::json;
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

    let envelope = RegistrationCertificateEnvelope::from_slice(encoded.as_slice()).unwrap();

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
    let envelope = RegistrationCertificateEnvelope::from_slice(encoded.as_slice()).unwrap();
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
#[case::no_expiry(None, false)]
#[case::before_expiry(Some(3600), false)]
#[case::at_expiry(Some(0), false)]
#[case::within_leeway(Some(-30), false)]
#[case::outside_leeway(Some(-120), true)]
#[tokio::test]
async fn verify_and_validate_registration_certificate(
    #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
    #[case] expires_in_seconds: Option<i64>,
    #[case] expired: bool,
) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    // jsonwebtoken checks the system clock, so keep JWT cases away from the 60-second leeway boundary.
    // The CWT verifier's unit test covers that boundary using a fixed clock.
    let now = match format {
        EnvelopeFormat::Jwt => DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap(),
        EnvelopeFormat::Cwt => validation_time(),
    };
    let time = MockTimeGenerator::new(now);
    let mut payload = registration_certificate_payload();
    payload.0["iat"] = json!((now - Duration::minutes(2)).timestamp());
    if let Some(seconds) = expires_in_seconds {
        payload.0["exp"] = json!((now + Duration::seconds(seconds)).timestamp());
    }
    let encoded = signed_registration_certificate(format, &payload, &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::from_slice(encoded.as_slice()).unwrap();
    let verified = verify_registration_certificate_envelope(&envelope, &ca.to_trust_anchors(), &time);
    if expired {
        match format {
            EnvelopeFormat::Jwt => assert!(matches!(
                verified,
                Err(RegistrationCertificateEnvelopeError::Jwt(JwtX5cVerifyError::JwtVerify(
                    JwtVerifyError::Validation(error)
                ))) if *error.kind() == ValidationErrorKind::ExpiredSignature
            )),
            EnvelopeFormat::Cwt => assert!(matches!(
                verified,
                Err(RegistrationCertificateEnvelopeError::Cwt(WrprcCwtError::Expired))
            )),
        }
        return;
    }

    let payload = verified.unwrap().into_payload();
    let certificate = payload.validate_binding(&access_certificate_subject()).unwrap();

    assert_eq!(certificate.payload().sub, "LEIXG-529900T8BM49AURSDO55");
}

#[rstest]
#[tokio::test]
async fn reject_null_registration_certificate_expiry(
    #[values(EnvelopeFormat::Jwt, EnvelopeFormat::Cwt)] format: EnvelopeFormat,
) {
    let ca = Ca::generate_wrpac_mock_ca().unwrap();
    let time = MockTimeGenerator::new(validation_time());
    let mut payload = registration_certificate_payload();
    payload.0["exp"] = json!(null);
    let encoded = signed_registration_certificate(format, &payload, &ca, &time).await;
    let envelope = RegistrationCertificateEnvelope::from_slice(encoded.as_slice()).unwrap();

    let result = verify_registration_certificate_envelope(&envelope, &ca.to_trust_anchors(), &time);

    assert!(matches!(
        (format, result),
        (EnvelopeFormat::Jwt, Err(RegistrationCertificateEnvelopeError::Jwt(_)))
            | (EnvelopeFormat::Cwt, Err(RegistrationCertificateEnvelopeError::Cwt(_)))
    ));
}
