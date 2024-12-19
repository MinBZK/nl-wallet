use std::fmt::Debug;
use std::num::NonZeroU8;

use ciborium::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use indexmap::IndexSet;

use wallet_common::trust_anchor::BorrowingTrustAnchor;

use crate::identifiers::AttributeIdentifier;
use crate::identifiers::AttributeIdentifierHolder;
use crate::iso::device_retrieval::DeviceRequest;
use crate::iso::device_retrieval::DocRequest;
use crate::iso::device_retrieval::ItemsRequest;
use crate::iso::mdocs::DataElementValue;
use crate::unsigned::Entry;
use crate::unsigned::UnsignedMdoc;
use crate::utils::cose::CoseError;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::TaggedBytes;
use crate::verifier::DisclosedAttributes;
use crate::verifier::DocumentDisclosedAttributes;
use crate::verifier::ItemsRequests;
use crate::MobileSecurityObject;

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
/// use nl_wallet_mdoc::test::DebugCollapseBts;
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

impl<T> Debug for DebugCollapseBts<T>
where
    T: Debug,
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

        write!(f, "{}", debugstr_collapsed)
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Assert that the specified doctype was disclosed, and that it contained the specified namespace,
/// and that the first attribute in that namespace has the specified name and value.
pub fn assert_disclosure_contains(
    disclosed_attrs: &DisclosedAttributes,
    doctype: &str,
    namespace: &str,
    name: &str,
    value: &DataElementValue,
) {
    let (disclosed_attr_name, disclosed_attr_value) = disclosed_attrs
        .get(doctype)
        .unwrap()
        .attributes
        .get(namespace)
        .unwrap()
        .first()
        .unwrap();

    assert_eq!(disclosed_attr_name, name);
    assert_eq!(disclosed_attr_value, value);
}

impl DeviceRequest {
    pub fn from_doc_requests(doc_requests: Vec<DocRequest>) -> Self {
        DeviceRequest {
            doc_requests,
            ..Default::default()
        }
    }

    pub fn from_items_requests(items_requests: Vec<ItemsRequest>) -> Self {
        Self::from_doc_requests(
            items_requests
                .into_iter()
                .map(|items_request| DocRequest {
                    items_request: items_request.into(),
                    reader_auth: None,
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDocument {
    pub doc_type: String,
    pub namespaces: IndexMap<String, Vec<Entry>>,
}

impl TestDocument {
    fn new(doc_type: String, namespaces: IndexMap<String, Vec<Entry>>) -> Self {
        Self { doc_type, namespaces }
    }

    #[cfg(any(test, all(feature = "generate", feature = "mock")))]
    /// Converts `self` into an [`UnsignedMdoc`] and signs it into an [`Mdoc`] using `ca` and `key_factory`.
    pub async fn sign<KF>(
        self,
        ca: &crate::server_keys::KeyPair,
        key_factory: &KF,
        copy_count: NonZeroU8,
    ) -> crate::holder::Mdoc
    where
        KF: wallet_common::keys::factory::KeyFactory,
    {
        use wallet_common::generator::TimeGenerator;
        use wallet_common::keys::WithIdentifier;

        use crate::holder::Mdoc;

        let (issuer_signed, mdoc_key) = self.issuer_signed(ca, key_factory, copy_count).await;

        let borrowing_trust_anchor = ca.to_trust_anchor().unwrap();
        Mdoc::new::<KF::Key>(
            mdoc_key.identifier().to_string(),
            issuer_signed,
            &TimeGenerator,
            &[(&borrowing_trust_anchor).into()],
        )
        .unwrap()
    }

    #[cfg(any(test, all(feature = "generate", feature = "mock")))]
    /// Converts `self` into an [`UnsignedMdoc`] and signs it into an [`Mdoc`] using `ca` and `key_factory`.
    pub async fn issuer_signed<KF>(
        self,
        ca: &crate::server_keys::KeyPair,
        key_factory: &KF,
        copy_count: NonZeroU8,
    ) -> (crate::IssuerSigned, KF::Key)
    where
        KF: wallet_common::keys::factory::KeyFactory,
    {
        use wallet_common::keys::EcdsaKey;

        use crate::iso::disclosure::IssuerSigned;
        use crate::utils::issuer_auth::IssuerRegistration;

        let unsigned = {
            let mut unsigned = UnsignedMdoc::from(self);
            unsigned.copy_count = copy_count;
            unsigned
        };
        let issuance_key = ca.generate_issuer_mock(IssuerRegistration::new_mock().into()).unwrap();

        let mdoc_key = key_factory.generate_new().await.unwrap();
        let mdoc_public_key = (&mdoc_key.verifying_key().await.unwrap()).try_into().unwrap();
        let issuer_signed = IssuerSigned::sign(unsigned, mdoc_public_key, &issuance_key)
            .await
            .unwrap();

        (issuer_signed, mdoc_key)
    }
}

impl From<(&'static str, &'static str, Vec<(&'static str, Value)>)> for TestDocument {
    fn from((doc_type, namespace, attributes): (&'static str, &'static str, Vec<(&'static str, Value)>)) -> Self {
        Self::new(
            doc_type.to_string(),
            IndexMap::from_iter(vec![(
                namespace.to_string(),
                attributes
                    .into_iter()
                    .map(|(name, value)| Entry {
                        name: name.into(),
                        value,
                    })
                    .collect(),
            )]),
        )
    }
}

impl From<TestDocument> for UnsignedMdoc {
    fn from(value: TestDocument) -> Self {
        Self {
            doc_type: value.doc_type,
            copy_count: NonZeroU8::new(1).unwrap(),
            valid_from: chrono::Utc::now().into(),
            valid_until: (chrono::Utc::now() + chrono::Duration::days(365)).into(),
            attributes: value.namespaces.try_into().unwrap(),
        }
    }
}

impl From<TestDocument> for ItemsRequest {
    fn from(value: TestDocument) -> Self {
        Self {
            doc_type: value.doc_type,
            name_spaces: IndexMap::from_iter(value.namespaces.into_iter().map(|(namespace, attributes)| {
                (
                    namespace,
                    IndexMap::from_iter(attributes.into_iter().map(|attribute| (attribute.name, true))),
                )
            })),
            request_info: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDocuments(pub Vec<TestDocument>);
impl TestDocuments {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_first(self) -> Option<TestDocument> {
        self.0.into_iter().next()
    }

    pub fn assert_matches(&self, disclosed_documents: &IndexMap<String, DocumentDisclosedAttributes>) {
        // verify the number of documents
        assert_eq!(disclosed_documents.len(), self.len());
        for TestDocument {
            doc_type: expected_doc_type,
            namespaces: expected_namespaces,
        } in self.0.iter()
        {
            // verify the disclosed attributes
            let disclosed_namespaces = disclosed_documents
                .get(expected_doc_type)
                .expect("expected doc_type not received");
            // verify the number of namespaces in this document
            assert_eq!(disclosed_namespaces.attributes.len(), expected_namespaces.len());
            for (expected_namespace, expected_attributes) in expected_namespaces {
                let disclosed_attributes = disclosed_namespaces
                    .attributes
                    .get(expected_namespace)
                    .expect("expected namespace not received");
                // verify the number of the attributes in this namespace
                assert_eq!(disclosed_attributes.len(), expected_attributes.len());
                // verify whether all expected attributes exist in this namespace
                for expected_attribute in expected_attributes {
                    assert_eq!(
                        disclosed_attributes.get(&expected_attribute.name),
                        Some(&expected_attribute.value)
                    );
                }
            }
        }
    }
}

impl From<Vec<TestDocument>> for TestDocuments {
    fn from(value: Vec<TestDocument>) -> Self {
        Self(value)
    }
}

impl IntoIterator for TestDocuments {
    type Item = TestDocument;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<TestDocuments> for ItemsRequests {
    fn from(value: TestDocuments) -> Self {
        let requests: Vec<_> = value.into_iter().map(ItemsRequest::from).collect();
        Self::from(requests)
    }
}

impl std::ops::Add for TestDocuments {
    type Output = TestDocuments;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

impl From<TestDocuments> for DeviceRequest {
    fn from(value: TestDocuments) -> Self {
        let items_requests = ItemsRequests::from(value);
        Self::from_items_requests(items_requests.0)
    }
}

impl AttributeIdentifierHolder for TestDocuments {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.0
            .iter()
            .flat_map(|document| {
                document.namespaces.iter().flat_map(|(namespace, attributes)| {
                    attributes.iter().map(|attribute| AttributeIdentifier {
                        credential_type: document.doc_type.clone(),
                        namespace: namespace.clone(),
                        attribute: attribute.name.clone(),
                    })
                })
            })
            .collect()
    }
}

impl MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> {
    pub fn doc_type(&self) -> Result<String, CoseError> {
        Ok(self.dangerous_parse_unverified()?.0.doc_type)
    }
}

pub mod data {
    use super::*;

    const PID: &str = "com.example.pid";
    const ADDR: &str = "com.example.address";

    pub fn empty() -> TestDocuments {
        vec![].into()
    }

    pub fn pid_given_name() -> TestDocuments {
        vec![(PID, PID, vec![("given_name", "Willeke Liselotte".into())]).into()].into()
    }

    pub fn pid_family_name() -> TestDocuments {
        vec![(PID, PID, vec![("family_name", "De Bruijn".into())]).into()].into()
    }

    pub fn pid_full_name() -> TestDocuments {
        vec![(
            PID,
            PID,
            vec![
                ("given_name", "Willeke Liselotte".into()),
                ("family_name", "De Bruijn".into()),
            ],
        )
            .into()]
        .into()
    }

    pub fn addr_street() -> TestDocuments {
        vec![(ADDR, ADDR, vec![("resident_street", "Turfmarkt".into())]).into()].into()
    }
}
