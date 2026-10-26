use std::sync::Arc;

use attestation_data::registration_certificate::RegistrationCertificateEnvelope;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use crypto::server_keys::generate::Ca;
use mdoc::CipherSuiteIdentifier;
use mdoc::DeviceAuth;
use mdoc::DeviceEngagement;
use mdoc::DeviceRequest;
use mdoc::DeviceResponse;
use mdoc::Engagement;
use mdoc::EngagementVersion;
use mdoc::SecurityKeyed;
use mdoc::utils::cose::CoseKey;
use mdoc::utils::serialization::cbor_deserialize;
use mdoc::utils::serialization::cbor_serialize;
use p256::ecdsa::SigningKey;
use p256::elliptic_curve::Generate;
use parking_lot::Mutex;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureError;
use platform_support::close_proximity_disclosure::CloseProximityDisclosureUpdate;
use platform_support::close_proximity_disclosure::MockCloseProximityDisclosureClient;
use rstest::rstest;
use serial_test::serial;
use tokio::sync::mpsc;
use wallet::hardware_test::ResponseArtifact;
use wallet::hardware_test::Verification;
use wallet::hardware_test::run_disclosure;
use wallet::hardware_test::run_disclosure_with_jwt;
use wallet::hardware_test::run_registration_certificate_rejection;
use wallet::hardware_test::verify_response;

/// Runs the same public wallet API and real signatures as the device tests, replacing only BLE.
async fn round_trip(
    rejection: Option<RegistrationCertificateCase>,
    jwt_registration_certificate: bool,
) -> (Verification, ResponseArtifact) {
    let (updates, receiver) = mpsc::channel(128);
    let start = MockCloseProximityDisclosureClient::start_qr_handover_context();
    start.expect().once().return_once(|| {
        let engagement = DeviceEngagement::from(Engagement {
            version: EngagementVersion::V1_0,
            security: SecurityKeyed {
                cipher_suite_identifier: CipherSuiteIdentifier::P256,
                e_device_key_bytes: CoseKey::try_from(SigningKey::generate().verifying_key())
                    .unwrap()
                    .into(),
            }
            .into(),
            device_retrieval_methods: None,
        });
        Ok((
            BASE64_URL_SAFE_NO_PAD.encode(cbor_serialize(&engagement).unwrap()),
            receiver,
        ))
    });
    let captured = Arc::new(Mutex::new(None));
    let captured_response = Arc::clone(&captured);
    let close_updates = updates.clone();
    let send = MockCloseProximityDisclosureClient::send_device_response_context();
    send.expect().once().return_once(move |response| {
        *captured_response.lock() = Some(response);
        close_updates.try_send(CloseProximityDisclosureUpdate::Closed).unwrap();
        Ok(())
    });
    let stop = MockCloseProximityDisclosureClient::stop_ble_server_context();
    stop.expect().once().returning(|| Ok(()));
    let expected = Arc::new(Mutex::new(None));
    let expected_ready = Arc::clone(&expected);
    let reader_ready = move |ready: wallet::hardware_test::ReaderReady| {
        let device_request = hex::decode(ready.device_request_hex).unwrap();
        if jwt_registration_certificate {
            let request: DeviceRequest = cbor_deserialize(device_request.as_slice()).unwrap();
            let doc_request = request.doc_requests.iter().next().unwrap();
            assert!(
                matches!(
                    RegistrationCertificateEnvelope::from_request_info(&doc_request.items_request.0).unwrap(),
                    RegistrationCertificateEnvelope::Jwt(_)
                ),
                "the reader must send a JWT registration certificate"
            );
        }
        let verification = serde_json::to_value(&ready.wallet_verification).unwrap();
        let transcript = verification["session_transcript_hex"].as_str().unwrap();
        updates.try_send(CloseProximityDisclosureUpdate::Connected).unwrap();
        updates
            .try_send(CloseProximityDisclosureUpdate::SessionEstablished {
                session_transcript: hex::decode(transcript).unwrap(),
                device_request,
            })
            .unwrap();
        *expected_ready.lock() = Some(ready.wallet_verification);
    };
    match rejection {
        Some(case) => {
            run_registration_certificate_rejection::<MockCloseProximityDisclosureClient>(case, reader_ready).await
        }
        None if jwt_registration_certificate => {
            run_disclosure_with_jwt::<MockCloseProximityDisclosureClient>(reader_ready).await
        }
        None => run_disclosure::<MockCloseProximityDisclosureClient>(reader_ready).await,
    }
    .expect("wallet scenario should produce its expected outcome and clean up");

    let expected = expected.lock().take().unwrap();
    let expected_json = serde_json::to_value(&expected).unwrap();
    let received = ResponseArtifact {
        device_response_hex: hex::encode(captured.lock().take().unwrap()),
        session_transcript_hex: expected_json["session_transcript_hex"].as_str().unwrap().to_owned(),
    };
    (expected, received)
}

#[tokio::test]
#[serial(MockCloseProximityDisclosureClient)]
async fn signed_disclosure_round_trip_and_verifier_rejections() {
    let (expected, mut received) = round_trip(None, false).await;
    let expected_json = serde_json::to_value(&expected).unwrap();
    verify_response(&expected, &received)
        .await
        .expect("the actual response signature must verify");

    let mut untrusted = expected_json.clone();
    untrusted["issuer_trust_anchors"] =
        serde_json::to_value(Ca::generate_issuer_mock_ca().unwrap().to_trust_anchors()).unwrap();
    let untrusted: Verification = serde_json::from_value(untrusted).unwrap();
    assert!(
        verify_response(&untrusted, &received).await.is_err(),
        "untrusted issuer must fail"
    );

    // A transcript mismatch must fail even when the response itself is otherwise valid.
    let real_transcript = received.session_transcript_hex.clone();
    received.session_transcript_hex = "83f6f6f6".to_owned();
    assert!(verify_response(&expected, &received).await.is_err());
    // If the reader and expected metadata agree on a different transcript, device authentication must still fail.
    let mut wrong_session = expected_json;
    wrong_session["session_transcript_hex"] = serde_json::json!(received.session_transcript_hex);
    let wrong_session: Verification = serde_json::from_value(wrong_session).unwrap();
    assert!(
        verify_response(&wrong_session, &received).await.is_err(),
        "device signature must bind the actual session"
    );
    received.session_transcript_hex = real_transcript;

    let mut response: DeviceResponse =
        cbor_deserialize(hex::decode(&received.device_response_hex).unwrap().as_slice()).unwrap();
    let mut tampered = response.clone();
    let mut documents = tampered.documents.take().unwrap().into_inner();
    let DeviceAuth::DeviceSignature(signature) = &mut documents[0].device_signed.device_auth else {
        panic!("expected device signature");
    };
    signature.as_mut().signature[0] ^= 1;
    tampered.documents = Some(documents.try_into().unwrap());
    received.device_response_hex = hex::encode(cbor_serialize(&tampered).unwrap());
    assert!(
        verify_response(&expected, &received).await.is_err(),
        "modified signature must fail"
    );
    response.status = mdoc::DeviceResponseStatus::GeneralError;
    received.device_response_hex = hex::encode(cbor_serialize(&response).unwrap());
    assert!(
        verify_response(&expected, &received).await.is_err(),
        "error response must not pass"
    );
}

#[rstest]
#[case::missing(RegistrationCertificateCase::Missing)]
#[case::wrong_type(RegistrationCertificateCase::WrongType)]
#[case::malformed(RegistrationCertificateCase::Malformed)]
#[case::tampered_signature(RegistrationCertificateCase::TamperedSignature)]
#[case::expired(RegistrationCertificateCase::Expired)]
#[case::untrusted(RegistrationCertificateCase::Untrusted)]
#[case::mismatched_reader(RegistrationCertificateCase::MismatchedReader)]
#[case::mismatched_subject_type(RegistrationCertificateCase::MismatchedSubjectType)]
#[case::revoked(RegistrationCertificateCase::Revoked)]
#[case::suspended(RegistrationCertificateCase::Suspended)]
#[case::status_unavailable(RegistrationCertificateCase::StatusUnavailable)]
#[case::untrusted_status_list(RegistrationCertificateCase::UntrustedStatusList)]
#[case::status_index_out_of_bounds(RegistrationCertificateCase::StatusIndexOutOfBounds)]
#[case::unauthorized_document_type(RegistrationCertificateCase::UnauthorizedDocumentType)]
#[case::unauthorized_namespace(RegistrationCertificateCase::UnauthorizedNamespace)]
#[case::unauthorized_attribute(RegistrationCertificateCase::UnauthorizedAttribute)]
#[case::unauthorized_second_document(RegistrationCertificateCase::UnauthorizedSecondDocument)]
#[tokio::test]
#[serial(MockCloseProximityDisclosureClient)]
async fn registration_certificate_rejection(#[case] case: RegistrationCertificateCase) {
    let (expected, mut received) = round_trip(Some(case), false).await;
    verify_response(&expected, &received).await.unwrap();

    let mut response: DeviceResponse =
        cbor_deserialize(hex::decode(&received.device_response_hex).unwrap().as_slice()).unwrap();
    // An unrelated error or an empty success response must not satisfy a rejection test.
    for status in [
        mdoc::DeviceResponseStatus::CborDecodingError,
        mdoc::DeviceResponseStatus::Ok,
    ] {
        response.status = status;
        received.device_response_hex = hex::encode(cbor_serialize(&response).unwrap());
        assert!(verify_response(&expected, &received).await.is_err());
    }
}

#[tokio::test]
#[serial(MockCloseProximityDisclosureClient)]
async fn rejection_verifier_rejects_disclosed_documents() {
    let (expected, mut received) = round_trip(None, false).await;
    let mut expected = serde_json::to_value(expected).unwrap();
    expected["expected_error_status"] = serde_json::json!(10);
    let expected: Verification = serde_json::from_value(expected).unwrap();
    let mut response: DeviceResponse =
        cbor_deserialize(hex::decode(&received.device_response_hex).unwrap().as_slice()).unwrap();
    response.status = mdoc::DeviceResponseStatus::GeneralError;
    received.device_response_hex = hex::encode(cbor_serialize(&response).unwrap());
    assert!(
        verify_response(&expected, &received).await.is_err(),
        "error status must not conceal disclosed documents"
    );
    response.documents = None;
    response.document_errors = Some(
        vec![indexmap::IndexMap::from([("example.document".to_owned(), 0)])]
            .try_into()
            .unwrap(),
    );
    received.device_response_hex = hex::encode(cbor_serialize(&response).unwrap());
    assert!(
        verify_response(&expected, &received).await.is_err(),
        "rejection must not include per-document results"
    );
}

#[tokio::test]
#[serial(MockCloseProximityDisclosureClient)]
async fn jwt_registration_certificate_disclosure_round_trip() {
    let (expected, received) = round_trip(None, true).await;
    verify_response(&expected, &received).await.unwrap();
}

#[tokio::test]
#[serial(MockCloseProximityDisclosureClient)]
async fn transport_failure_preserves_error_and_cleans_up() {
    let start = MockCloseProximityDisclosureClient::start_qr_handover_context();
    start.expect().once().returning(|| {
        Err(CloseProximityDisclosureError::PlatformError {
            reason: "synthetic BLE start failure".to_owned(),
        })
    });
    let stop = MockCloseProximityDisclosureClient::stop_ble_server_context();
    stop.expect().once().returning(|| Ok(()));
    let error = run_disclosure::<MockCloseProximityDisclosureClient>(|_| panic!("reader must not start"))
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("synthetic BLE start failure"));
}
use wallet::hardware_test::RegistrationCertificateCase;
