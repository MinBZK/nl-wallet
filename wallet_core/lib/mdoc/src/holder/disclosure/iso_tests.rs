use futures::FutureExt;
use indexmap::IndexMap;

use attestation_types::request::NormalizedCredentialRequest;
use crypto::examples::Examples;
use crypto::mock_remote::MockRemoteKeyFactory;
use crypto::server_keys::generate::Ca;
use dcql::CredentialQueryFormat;
use utils::vec_at_least::VecNonEmpty;

use crate::examples::Example;
use crate::examples::IsoCertTimeGenerator;
use crate::examples::EXAMPLE_ATTR_NAME;
use crate::examples::EXAMPLE_ATTR_VALUE;
use crate::examples::EXAMPLE_DOC_TYPE;
use crate::examples::EXAMPLE_NAMESPACE;
use crate::holder::disclosure::credential_requests_to_mdoc_paths;
use crate::holder::Mdoc;
use crate::iso::device_retrieval::DeviceRequest;
use crate::iso::device_retrieval::ItemsRequest;
use crate::iso::device_retrieval::ReaderAuthenticationBytes;
use crate::iso::disclosure::DeviceResponse;
use crate::iso::engagement::DeviceAuthenticationBytes;
use crate::iso::engagement::SessionTranscript;
use crate::test;
use crate::test::DebugCollapseBts;
use crate::utils::serialization::CborSeq;
use crate::utils::serialization::TaggedBytes;

fn create_example_device_response(
    device_request: DeviceRequest,
    session_transcript: &SessionTranscript,
    ca: &Ca,
) -> DeviceResponse {
    let mut mdoc = Mdoc::new_example_resigned(ca).now_or_never().unwrap();

    let credential_requests: VecNonEmpty<NormalizedCredentialRequest> = device_request.into_items_requests().into();

    assert_eq!(
        match &credential_requests.as_ref().first().unwrap().format {
            CredentialQueryFormat::MsoMdoc { doctype_value } => Some(doctype_value),
            _ => None,
        },
        Some(&mdoc.mso.doc_type)
    );

    mdoc.issuer_signed = mdoc
        .issuer_signed
        .into_attribute_subset(&credential_requests_to_mdoc_paths(
            &credential_requests,
            &mdoc.mso.doc_type,
        ));

    let (device_response, _) =
        DeviceResponse::sign_from_mdocs(vec![mdoc], session_transcript, &MockRemoteKeyFactory::new_example())
            .now_or_never()
            .unwrap()
            .unwrap();

    device_response
}

/// Construct the example mdoc from the standard and disclose attributes
/// by running the example device request from the standard against it.
#[tokio::test]
async fn do_and_verify_iso_example_disclosure() {
    let device_request = DeviceRequest::example();

    // Examine some fields in the device request
    let items_request = device_request.doc_requests.first().unwrap().items_request.0.clone();
    assert_eq!(items_request.doc_type, EXAMPLE_DOC_TYPE);
    let requested_attrs = items_request.name_spaces.get(EXAMPLE_NAMESPACE).unwrap();
    let intent_to_retain = requested_attrs.get(EXAMPLE_ATTR_NAME).unwrap();
    assert!(intent_to_retain);
    println!("DeviceRequest: {:#?}", DebugCollapseBts::from(&device_request));

    // Verify reader's request
    let reader_trust_anchors = Examples::reader_trust_anchors();
    let TaggedBytes(CborSeq(example_reader_auth)) = ReaderAuthenticationBytes::example();
    let session_transcript = example_reader_auth.session_transcript.into_owned();
    let certificate = device_request
        .doc_requests
        .first()
        .unwrap()
        .verify(&session_transcript, &IsoCertTimeGenerator, reader_trust_anchors)
        .unwrap()
        .unwrap();
    let reader_x509_subject = certificate.subject();

    // The reader's certificate contains who it is
    assert_eq!(
        reader_x509_subject.as_ref().unwrap().first().unwrap(),
        (&"CN".to_string(), &"reader")
    );
    println!("Reader: {reader_x509_subject:#?}");

    // Construct a new `DeviceResponse`, based on the mdoc from the example device response in the standard.
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let resp = create_example_device_response(device_request, &session_transcript, &ca);
    println!("DeviceResponse: {:#?}", DebugCollapseBts::from(&resp));

    // Verify this second `DeviceResponse`.
    let disclosed_attrs = resp
        .verify(
            None,
            &session_transcript,
            &IsoCertTimeGenerator,
            &[ca.to_trust_anchor()],
        )
        .unwrap();
    println!("DisclosedAttributes: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

    // The first disclosed attribute is the same as we saw earlier in the DeviceRequest
    test::assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}

/// Disclose some of the attributes of the example mdoc from the spec.
#[tokio::test]
async fn iso_examples_custom_disclosure() {
    let request = DeviceRequest::from_items_requests(vec![ItemsRequest {
        doc_type: EXAMPLE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            EXAMPLE_NAMESPACE.to_string(),
            IndexMap::from([(EXAMPLE_ATTR_NAME.to_string(), false)]),
        )]),
        request_info: None,
    }]);
    println!("My Request: {:#?}", DebugCollapseBts::from(&request));

    let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
    let ca = Ca::generate_issuer_mock_ca().unwrap();
    let resp = create_example_device_response(request, &session_transcript, &ca);
    println!("My DeviceResponse: {:#?}", DebugCollapseBts::from(&resp));

    let disclosed_attrs = resp
        .verify(
            None,
            &session_transcript,
            &IsoCertTimeGenerator,
            &[ca.to_trust_anchor()],
        )
        .unwrap();
    println!("My Disclosure: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

    // The first disclosed attribute is the one we requested in our device request
    test::assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}
