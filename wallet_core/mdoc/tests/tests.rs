use core::fmt::Debug;
use std::ops::Add;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use ciborium::value::Value;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use p256::ecdsa::VerifyingKey;
use serde::{de::DeserializeOwned, Serialize};

use url::Url;
use wallet_common::{generator::Generator, keys::software::SoftwareEcdsaKey};

use nl_wallet_mdoc::{
    basic_sa_ext::{Entry, UnsignedMdoc},
    holder::*,
    iso::*,
    issuer::*,
    utils::{
        mdocs_map::MdocsMap,
        serialization::{cbor_deserialize, cbor_serialize},
        x509::{Certificate, CertificateUsage},
    },
    verifier::DisclosedAttributes,
    Error,
};

mod examples;
use examples::*;

const EXAMPLE_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";
const EXAMPLE_NAMESPACE: &str = "org.iso.18013.5.1";
const EXAMPLE_ATTR_NAME: &str = "family_name";
static EXAMPLE_ATTR_VALUE: Lazy<Value> = Lazy::new(|| Value::Text("Doe".to_string())); // Lazy since can't have a const String

/// Verify the example disclosure from the standard.
#[test]
fn verify_iso_example_disclosure() {
    let device_response = DeviceResponse::example();
    println!("DeviceResponse: {:#?} ", DebugCollapseBts(&device_response));

    // Examine the first attribute in the device response
    let document = device_response.documents.as_ref().unwrap()[0].clone();
    assert_eq!(document.doc_type, EXAMPLE_DOC_TYPE);
    let namespaces = document.issuer_signed.name_spaces.as_ref().unwrap();
    let attrs = namespaces.get(EXAMPLE_NAMESPACE).unwrap();
    let issuer_signed_attr = attrs.0.first().unwrap().0.clone();
    assert_eq!(issuer_signed_attr.element_identifier, EXAMPLE_ATTR_NAME);
    assert_eq!(issuer_signed_attr.element_value, *EXAMPLE_ATTR_VALUE);
    println!("issuer_signed_attr: {:#?}", DebugCollapseBts(&issuer_signed_attr));

    // Do the verification
    let eph_reader_key = Examples::ephemeral_reader_key();
    let trust_anchors = Examples::iaca_trust_anchors();
    let disclosed_attrs = device_response
        .verify(
            Some(&eph_reader_key),
            &DeviceAuthenticationBytes::example_bts(), // To be signed by device key found in MSO
            &IsoCertTimeGenerator,
            trust_anchors,
        )
        .unwrap();
    println!("DisclosedAttributes: {:#?}", DebugCollapseBts(&disclosed_attrs));

    // The first disclosed attribute is the same as we saw earlier in the DeviceResponse
    assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}

/// Construct the example mdoc from the standard and disclose attributes
/// by running the example device request from the standard against it.
#[test]
fn do_and_verify_iso_example_disclosure() {
    let device_request = DeviceRequest::example();

    // Examine some fields in the device request
    let items_request = device_request.doc_requests.first().unwrap().items_request.0.clone();
    assert_eq!(items_request.doc_type, EXAMPLE_DOC_TYPE);
    let requested_attrs = items_request.name_spaces.get(EXAMPLE_NAMESPACE).unwrap();
    let intent_to_retain = requested_attrs.get(EXAMPLE_ATTR_NAME).unwrap();
    assert!(intent_to_retain);
    println!("DeviceRequest: {:#?}", DebugCollapseBts(&device_request));

    // Verify reader's request
    let reader_trust_anchors = Examples::reader_trust_anchors();
    let reader_x509_subject = device_request
        .verify(
            &ReaderAuthenticationBytes::example_bts(),
            &IsoCertTimeGenerator,
            reader_trust_anchors,
        )
        .unwrap();

    // The reader's certificate contains who it is
    assert_eq!(
        reader_x509_subject.as_ref().unwrap().first().unwrap(),
        (&"CN".to_string(), &"reader".to_string())
    );
    println!("Reader: {:#?}", reader_x509_subject);

    // Construct the mdoc from the example device response in the standard
    let trust_anchors = Examples::iaca_trust_anchors();
    let mdoc = mdoc_from_example_device_response(trust_anchors);

    // Do the disclosure and verify it
    let wallet = Wallet::new(MdocsMap::try_from([mdoc]).unwrap());
    let resp = wallet
        .disclose::<SoftwareEcdsaKey>(&device_request, &DeviceAuthenticationBytes::example_bts())
        .unwrap();
    println!("DeviceResponse: {:#?}", DebugCollapseBts(&resp));
    let disclosed_attrs = resp
        .verify(
            None,
            &DeviceAuthenticationBytes::example_bts(),
            &IsoCertTimeGenerator,
            trust_anchors,
        )
        .unwrap();
    println!("DisclosedAttributes: {:#?}", DebugCollapseBts(&disclosed_attrs));

    // The first disclosed attribute is the same as we saw earlier in the DeviceRequest
    assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}

/// Disclose some of the attributes of the example mdoc from the spec.
#[test]
fn iso_examples_custom_disclosure() {
    let request = DeviceRequest::new(vec![ItemsRequest {
        doc_type: EXAMPLE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            EXAMPLE_NAMESPACE.to_string(),
            IndexMap::from([(EXAMPLE_ATTR_NAME.to_string(), false)]),
        )]),
        request_info: None,
    }]);
    println!("My Request: {:#?}", DebugCollapseBts(&request));

    let trust_anchors = Examples::iaca_trust_anchors();
    let mdoc = mdoc_from_example_device_response(trust_anchors);

    let wallet = Wallet::new(MdocsMap::try_from([mdoc]).unwrap());
    let resp = wallet
        .disclose::<SoftwareEcdsaKey>(&request, &DeviceAuthenticationBytes::example_bts())
        .unwrap();

    println!("My DeviceResponse: {:#?}", DebugCollapseBts(&resp));
    let disclosed_attrs = resp
        .verify(
            None,
            &DeviceAuthenticationBytes::example_bts(),
            &IsoCertTimeGenerator,
            trust_anchors,
        )
        .unwrap();
    println!("My Disclosure: {:#?}", DebugCollapseBts(&disclosed_attrs));

    // The first disclosed attribute is the one we requested in our device request
    assert_disclosure_contains(
        &disclosed_attrs,
        EXAMPLE_DOC_TYPE,
        EXAMPLE_NAMESPACE,
        EXAMPLE_ATTR_NAME,
        &EXAMPLE_ATTR_VALUE,
    );
}

/// Out of the example data structures in the standard, assemble an mdoc.
/// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the originating
/// mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some [`IssuerSignedItem`]
/// instances, i.e. attributes.
/// This is because the other attributes are actually nowhere present in the standard so it is impossible to construct
/// the example mdoc with all attributes present.
///
/// Using tests should not rely on all attributes being present.
fn mdoc_from_example_device_response(trust_anchors: &[TrustAnchor]) -> Mdoc<SoftwareEcdsaKey> {
    // Prepare the mdoc's private key
    let static_device_key = Examples::static_device_key();
    SoftwareEcdsaKey::insert("example_static_device_key", static_device_key);

    let issuer_signed = DeviceResponse::example().documents.as_ref().unwrap()[0]
        .issuer_signed
        .clone();

    Mdoc::<SoftwareEcdsaKey>::new(
        "example_static_device_key".to_string(),
        issuer_signed,
        &IsoCertTimeGenerator,
        trust_anchors,
    )
    .unwrap()
}

/// Assert that the specified doctype was disclosed, and that it contained the specified namespace,
/// and that the first attribute in that namespace has the specified name and value.
fn assert_disclosure_contains(
    disclosed_attrs: &DisclosedAttributes,
    doctype: &str,
    namespace: &str,
    name: &str,
    value: &DataElementValue,
) {
    let disclosed_attr = disclosed_attrs
        .get(doctype)
        .unwrap()
        .get(namespace)
        .unwrap()
        .first()
        .unwrap();
    assert_eq!(disclosed_attr.name, *name);
    assert_eq!(disclosed_attr.value, *value);
}

/// Verify that the static device key example from the spec is the public key in the MSO.
#[test]
fn iso_examples_consistency() {
    let static_device_key = Examples::static_device_key();

    let device_key = &DeviceResponse::example().documents.unwrap()[0]
        .issuer_signed
        .issuer_auth
        .verify_against_trust_anchors(
            CertificateUsage::Mdl,
            &IsoCertTimeGenerator,
            Examples::iaca_trust_anchors(),
        )
        .unwrap()
        .0
        .device_key_info
        .device_key;

    assert_eq!(
        *static_device_key.verifying_key(),
        VerifyingKey::try_from(device_key).unwrap(),
    );
}

const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";
const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

fn new_issuance_request() -> Vec<UnsignedMdoc> {
    let now = chrono::Utc::now();
    vec![UnsignedMdoc {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        copy_count: 2,
        valid_from: now.into(),
        valid_until: chrono::Utc::now().add(chrono::Duration::days(365)).into(),
        attributes: IndexMap::from([(
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS
                .iter()
                .map(|(key, val)| Entry {
                    name: key.to_string(),
                    value: Value::Text(val.to_string()),
                })
                .collect(),
        )]),
    }]
}

struct MockHttpClient<'a, K, S> {
    issuance_server: &'a Server<K, S>,
}

#[async_trait]
impl<K, S> HttpClient for MockHttpClient<'_, K, S>
where
    K: KeyRing + Sync,
    S: SessionStore + Send + Sync + 'static,
{
    async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R, Error>
    where
        V: Serialize + Sync,
        R: DeserializeOwned,
    {
        let session_token = url.path_segments().unwrap().last().unwrap().to_string();
        let val = &cbor_serialize(val).unwrap();
        let response = self.issuance_server.process_message(session_token.into(), val).unwrap();
        let response = cbor_deserialize(response.as_slice()).unwrap();
        Ok(response)
    }
}

struct MockIssuanceKeyring {
    issuance_key: PrivateKey,
}
impl KeyRing for MockIssuanceKeyring {
    fn private_key(&self, _: &DocType) -> Option<&PrivateKey> {
        Some(&self.issuance_key)
    }
}

fn user_consent<const CONSENT: bool>() -> impl IssuanceUserConsent {
    struct MockUserConsent<const CONSENT: bool>;
    #[async_trait]
    impl<const CONSENT: bool> IssuanceUserConsent for MockUserConsent<CONSENT> {
        async fn ask(&self, _: &[UnsignedMdoc]) -> bool {
            CONSENT
        }
    }
    MockUserConsent::<CONSENT>
}

#[tokio::test]
async fn issuance_and_disclosure() {
    // Agree with issuance
    let (wallet, ca) = issuance_using_consent(user_consent::<true>(), new_issuance_request()).await;
    assert_eq!(1, wallet.list_mdocs::<SoftwareEcdsaKey>().len());

    // We can disclose the mdoc that was just issued to us
    custom_disclosure(wallet, ca);

    // Decline issuance
    let (wallet, _) = issuance_using_consent(user_consent::<false>(), new_issuance_request()).await;
    assert!(wallet.list_mdocs::<SoftwareEcdsaKey>().is_empty());

    // Issue not-yet-valid mdocs
    let now = Utc::now();
    let mut request = new_issuance_request();
    request
        .iter_mut()
        .for_each(|r| r.valid_from = now.add(Duration::days(132)).into());
    assert!(request[0].valid_from.0 .0.parse::<DateTime<Utc>>().unwrap() > now);
    let (wallet, _) = issuance_using_consent(user_consent::<true>(), request).await;
    assert_eq!(1, wallet.list_mdocs::<SoftwareEcdsaKey>().len());
}

async fn issuance_using_consent<T: IssuanceUserConsent>(
    user_consent: T,
    request: Vec<UnsignedMdoc>,
) -> (Wallet<MdocsMap>, Certificate) {
    // Issuer CA certificate and normal certificate
    let (ca, ca_privkey) = Certificate::new_ca(ISSUANCE_CA_CN).unwrap();
    let (issuer_cert, issuer_privkey) =
        Certificate::new(&ca, &ca_privkey, ISSUANCE_CERT_CN, CertificateUsage::Mdl).unwrap();
    let issuance_key = PrivateKey::new(issuer_privkey, issuer_cert.as_bytes().into());

    // Setup session and issuer
    let issuance_server = Server::new(
        "http://example.com".parse().unwrap(),
        MockIssuanceKeyring { issuance_key },
        MemorySessionStore::new(),
    );
    let service_engagement = issuance_server.new_session(request).unwrap();

    // Setup holder
    let wallet = Wallet::new(MdocsMap::new());
    assert!(wallet.list_mdocs::<SoftwareEcdsaKey>().is_empty());

    // Do issuance
    let client = MockHttpClient {
        issuance_server: &issuance_server,
    };
    wallet
        .do_issuance::<SoftwareEcdsaKey>(service_engagement, &user_consent, &client, &[(&ca).try_into().unwrap()])
        .await
        .unwrap();

    (wallet, ca)
}

fn custom_disclosure(wallet: Wallet<MdocsMap>, ca: Certificate) {
    assert!(!wallet.list_mdocs::<SoftwareEcdsaKey>().is_empty());

    // Create a request asking for one attribute
    let request = DeviceRequest::new(vec![ItemsRequest {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS.iter().map(|(key, _)| (key.to_string(), false)).collect(),
        )]),
        request_info: None,
    }]);

    // Do the disclosure and verify it
    let challenge = DeviceAuthenticationBytes::example_bts();
    let disclosed = wallet
        .disclose::<SoftwareEcdsaKey>(&request, challenge.as_ref())
        .unwrap();
    let disclosed_attrs = disclosed
        .verify(None, &challenge, &TimeGenerator, &[(&ca).try_into().unwrap()])
        .unwrap();
    println!("Disclosure: {:#?}", DebugCollapseBts(&disclosed_attrs));

    // Check the first disclosed attribute has the expected name and value
    let attr = ISSUANCE_ATTRS.first().unwrap();
    assert_disclosure_contains(
        &disclosed_attrs,
        ISSUANCE_DOC_TYPE,
        ISSUANCE_NAME_SPACE,
        attr.0,
        &Value::Text(attr.1.to_string()),
    );
}

/// Some of the certificates in the ISO examples are valid from Oct 1, 2020 to Oct 1, 2021.
/// This generator returns a time in that window.
struct IsoCertTimeGenerator;
impl Generator<DateTime<Utc>> for IsoCertTimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()
    }
}

struct TimeGenerator;
impl Generator<DateTime<Utc>> for TimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Wrapper around `T` that implements `Debug` by using `T`'s implementation,
/// but with byte sequences (which can take a lot of vertical space) replaced with
/// a CBOR diagnostic-like notation.
///
/// Example output:
///
/// ```text
/// Test {
///     a_string: "Hello, World",
///     an_int: 42,
///     a_byte_sequence: h'00012AFF',
/// }
/// ```
///
/// Example code:
/// ```rust
/// #[derive(Debug)]
/// struct Test {
///     a_string: String,
///     an_int: u64,
///     a_byte_sequence: Vec<u8>,
/// }
///
/// let test = Test {
///     a_string: "Hello, World".to_string(),
///     an_int: 42,
///     a_byte_sequence: vec![0, 1, 42, 255],
/// };
///
/// println!("{:#?}", DebugCollapseBts(test));
/// ```
struct DebugCollapseBts<T>(T);

impl<T> Debug for DebugCollapseBts<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Match numbers within square brackets, e.g.: [1, 2, 3]
        let debugstr = format!("{:#?}", self.0);
        let debugstr_collapsed = regex::Regex::new(r"\[\s*(\d,?\s*)+\]").unwrap().replace_all(
            debugstr.as_str(),
            |caps: &regex::Captures| {
                let no_whitespace = remove_whitespace(&caps[0]);
                let trimmed = no_whitespace[1..no_whitespace.len() - 2].to_string(); // Remove square brackets
                if trimmed.split(',').any(|r| r.parse::<u8>().is_err()) {
                    // If any of the numbers don't fit in a u8, just return the numbers without whitespace
                    no_whitespace
                } else {
                    format!(
                        "h'{}'", // CBOR diagnostic-like notation
                        hex::encode(
                            trimmed
                                .split(',')
                                .map(|i| i.parse::<u8>().unwrap())
                                .collect::<Vec<u8>>(),
                        )
                        .to_uppercase()
                    )
                }
            },
        );

        write!(f, "{}", debugstr_collapsed)
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}