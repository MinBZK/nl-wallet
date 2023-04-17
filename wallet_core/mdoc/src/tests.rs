use anyhow::{anyhow, Result};
use ciborium::value::Value;
use core::fmt::Debug;
use indexmap::IndexMap;
use p256::pkcs8::DecodePrivateKey;
use rcgen::{BasicConstraints, Certificate, CertificateParams, DnType, IsCa};
use serde_bytes::ByteBuf;
use std::ops::Add;
use x509_parser::prelude::{FromDer, X509Certificate};

use crate::{
    basic_sa_ext::{Entry, RequestKeyGenerationMessage, UnsignedMdoc},
    examples::*,
    holder::*,
    iso::*,
    issuer::*,
};

/// Verify that the static device key example from the spec is the public key in the MSO.
#[test]
fn iso_examples_consistency() -> Result<()> {
    let static_device_key = Examples::static_device_key();

    let device_key = &DeviceResponse::example().documents.unwrap()[0]
        .issuer_signed
        .issuer_auth
        .verify_against_cert(&Examples::issuer_ca_cert())?
        .0
         .0
        .device_key_info
        .device_key;

    assert_eq!(
        static_device_key.verifying_key(),
        ecdsa::VerifyingKey::<p256::NistP256>::try_from(device_key)?,
    );

    Ok(())
}

#[test]
fn iso_examples_disclosure() -> Result<()> {
    let ca_cert = Examples::issuer_ca_cert();
    let eph_reader_key = Examples::ephemeral_reader_key();
    let device_response = DeviceResponse::example();
    println!("DeviceResponse: {:#?} ", DebugCollapseBts(&device_response));

    // Do the verification
    let disclosed_attributes = device_response.verify(
        Some(&eph_reader_key),
        &DeviceAuthenticationBytes::example_bts(), // To be signed by device key found in MSO
        &ca_cert,
    )?;
    println!("DisclosedAttributes: {:#?}", DebugCollapseBts(disclosed_attributes));

    let device_request = DeviceRequest::example();
    println!("DeviceRequest: {:#?}", DebugCollapseBts(&device_request));

    let reader_ca_cert = Examples::reader_ca_cert();
    println!(
        "Reader: {:#?}",
        device_request.verify(&reader_ca_cert, &ReaderAuthenticationBytes::example_bts())?,
    );

    let static_device_key = Examples::static_device_key();
    let cred = Credential::new(
        static_device_key,
        device_response.documents.as_ref().unwrap()[0].issuer_signed.clone(),
        &ca_cert,
    )?;

    let creds = Credentials::from([(cred)]);
    let resp = creds.disclose(&device_request, &DeviceAuthenticationBytes::example_bts())?;

    println!("DeviceResponse: {:#?}", DebugCollapseBts(&resp));
    println!(
        "Disclosure: {:#?}",
        DebugCollapseBts(resp.verify(None, &DeviceAuthenticationBytes::example_bts(), &ca_cert)),
    );

    Ok(())
}

#[test]
fn iso_examples_custom_disclosure() -> Result<()> {
    let ca_cert = Examples::issuer_ca_cert();
    let static_device_key = Examples::static_device_key();
    let device_response = DeviceResponse::example();

    let request = DeviceRequest::new(vec![ItemsRequest {
        doc_type: "org.iso.18013.5.1.mDL".to_string(),
        name_spaces: IndexMap::from([(
            "org.iso.18013.5.1".to_string(),
            IndexMap::from([("family_name".to_string(), false)]),
        )]),
        request_info: None,
    }]);
    println!("My Request: {:#?}", DebugCollapseBts(&request));

    let cred = Credential::new(
        static_device_key,
        device_response.documents.as_ref().unwrap()[0].issuer_signed.clone(),
        &ca_cert,
    )?;

    let creds = Credentials::from([(cred)]);
    let resp = creds.disclose(&request, &DeviceAuthenticationBytes::example_bts())?;

    println!("My DeviceResponse: {:#?}", DebugCollapseBts(&resp));
    println!(
        "My Disclosure: {:#?}",
        DebugCollapseBts(resp.verify(None, &DeviceAuthenticationBytes::example_bts(), &ca_cert)),
    );

    Ok(())
}

const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";
const ISSUANCE_DOC_TYPE: &str = "example_doctype";
const ISSUANCE_NAME_SPACE: &str = "example_namespace";
const ISSUANCE_ATTRS: [(&str, &str); 2] = [("first_name", "John"), ("family_name", "Doe")];

fn new_issuance_request() -> RequestKeyGenerationMessage {
    RequestKeyGenerationMessage {
        e_session_id: ByteBuf::from("e_session_id"),
        challenge: ByteBuf::from("challenge"),
        unsigned_mdocs: IndexMap::from([(
            ISSUANCE_DOC_TYPE.to_string(),
            UnsignedMdoc {
                count: 2,
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
            },
        )]),
    }
}

#[test]
fn issuance_and_disclosure() -> Result<()> {
    // Issuer data
    let ca = new_ca(ISSUANCE_CA_CN)?;
    let (privkey, cert_bts) = new_certificate(&ca, ISSUANCE_CERT_CN)?;
    let ca_bts = ca.serialize_der()?;
    let ca_cert = X509Certificate::from_der(ca_bts.as_slice())?.1;

    let request = new_issuance_request();
    let issuer = Issuer::new(request.clone(), privkey, cert_bts);

    // User data
    let mut wallet = Credentials::new();

    // Do issuance
    let wallet_issuance_state = Credentials::issuance_start(&request)?;
    println!(
        "wallet response: {:#?}",
        DebugCollapseBts(&wallet_issuance_state.response)
    );
    let issuer_response = issuer.issue(&wallet_issuance_state.response)?;
    println!("issuer response: {:#?}", DebugCollapseBts(&issuer_response));
    let creds = Credentials::issuance_finish(wallet_issuance_state, issuer_response, &ca_cert)?;
    wallet.add(creds);

    // Disclose some attributes from our cred
    let request = DeviceRequest::new(vec![ItemsRequest {
        doc_type: ISSUANCE_DOC_TYPE.to_string(),
        name_spaces: IndexMap::from([(
            ISSUANCE_NAME_SPACE.to_string(),
            ISSUANCE_ATTRS.iter().map(|(key, _)| (key.to_string(), false)).collect(),
        )]),
        request_info: None,
    }]);

    let challenge = DeviceAuthenticationBytes::example_bts();
    let disclosed = wallet.disclose(&request, challenge.as_ref())?;
    println!(
        "Disclosure: {:#?}",
        DebugCollapseBts(disclosed.verify(None, &challenge, &ca_cert)?)
    );

    Ok(())
}

pub fn new_ca(common_name: &str) -> Result<Certificate, rcgen::RcgenError> {
    let mut ca_params = CertificateParams::new(vec![]);
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    ca_params.distinguished_name.push(DnType::CommonName, common_name);
    Certificate::from_params(ca_params)
}

pub fn new_certificate(ca: &Certificate, common_name: &str) -> Result<(ecdsa::SigningKey<p256::NistP256>, Vec<u8>)> {
    let mut cert_params = CertificateParams::new(vec![]);
    cert_params.is_ca = IsCa::NoCa;
    cert_params.distinguished_name.push(DnType::CommonName, common_name);
    // TODO: X509v3 Extended Key Usage: critical / 1.0.18013.5.1.2
    let cert = Certificate::from_params(cert_params)?;
    let cert_bts = cert.serialize_der_with_signer(ca)?;

    let cert_privkey: ecdsa::SigningKey<p256::NistP256> =
        ecdsa::SigningKey::from_pkcs8_der(cert.get_key_pair().serialized_der()).map_err(|e| anyhow!(e))?;

    Ok((cert_privkey, cert_bts))
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
