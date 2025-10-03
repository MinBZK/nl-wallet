use std::fmt;

use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
use crypto::server_keys::generate::mock::RP_CERT_CN;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;

use crate::iso::mdocs::DataElementValue;
use crate::verifier::DisclosedDocument;

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
/// use mdoc::test::DebugCollapseBts;
///
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
/// println!("{:#?}", DebugCollapseBts::from(test));
/// ```
pub struct DebugCollapseBts<T>(T);

impl<T> From<T> for DebugCollapseBts<T> {
    fn from(value: T) -> Self {
        DebugCollapseBts(value)
    }
}

impl<T> fmt::Debug for DebugCollapseBts<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Match numbers within square brackets, e.g.: [1, 2, 3]
        let debugstr = format!("{:#?}", self.0);
        let debugstr_collapsed =
            regex::Regex::new(r"\[\s*(\d,?\s*)+]")
                .unwrap()
                .replace_all(debugstr.as_str(), |caps: &regex::Captures| {
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
                });

        write!(f, "{debugstr_collapsed}")
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Assert that the specified doctype was disclosed, and that it contained the specified namespace,
/// and that the namespace contains the specified name and value.
pub fn assert_disclosure_contains(
    disclosed_documents: &[DisclosedDocument],
    doctype: &str,
    namespace: &str,
    name: &str,
    value: &DataElementValue,
) {
    let contains_attribute = disclosed_documents
        .iter()
        .filter(|document| document.doc_type == *doctype)
        .flat_map(|document| document.attributes.get(namespace))
        .any(|attributes| attributes.get(name) == Some(value));

    assert!(contains_attribute)
}

pub fn generate_issuer_mock(ca: &Ca) -> Result<KeyPair, CertificateError> {
    ca.generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
}

pub fn generate_reader_mock(ca: &Ca) -> Result<KeyPair, CertificateError> {
    ca.generate_key_pair(RP_CERT_CN, CertificateUsage::ReaderAuth, Default::default())
}
