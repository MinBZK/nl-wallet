use std::fmt;

use chrono::Duration;
use chrono::Utc;
use ciborium::Value;
use coset::CoseSign1;
use derive_more::Debug;
use indexmap::IndexMap;
use ssri::Integrity;

use crypto::CredentialEcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::server_keys::generate::Ca;
use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
use crypto::server_keys::generate::mock::RP_CERT_CN;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use http_utils::urls::HttpsUri;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::generator::mock::MockTimeGenerator;

use crate::DigestAlgorithm;
use crate::IssuerNameSpaces;
use crate::MobileSecurityObject;
use crate::MobileSecurityObjectVersion;
use crate::ValidityInfo;
use crate::holder::Mdoc;
use crate::iso::disclosure::IssuerSigned;
use crate::iso::mdocs::DataElementValue;
use crate::iso::mdocs::Entry;
use crate::utils::cose::CoseError;
use crate::utils::cose::CoseKey;
use crate::utils::cose::MdocCose;
use crate::utils::serialization::TaggedBytes;
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
/// and that the first attribute in that namespace has the specified name and value.
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

#[derive(Debug, Clone)]
pub struct TestDocument {
    pub doc_type: String,
    pub issuer_uri: HttpsUri,
    // TODO: change to: pub attributes: IndexMap<String, Attribute> in PVW-4138, or even remove TestDocument
    // altogether?
    pub namespaces: IndexMap<String, Vec<Entry>>,
    pub metadata_integrity: Integrity,
    #[debug(skip)]
    pub metadata: TypeMetadataDocuments,
}

impl TestDocument {
    pub fn new(
        doc_type: String,
        issuer_uri: HttpsUri,
        namespaces: IndexMap<String, Vec<Entry>>,
        (metadata_integrity, metadata): (Integrity, TypeMetadataDocuments),
    ) -> Self {
        Self {
            doc_type,
            issuer_uri,
            namespaces,
            metadata_integrity,
            metadata,
        }
    }

    pub fn normalized_metadata(&self) -> NormalizedTypeMetadata {
        let (normalized_metadata, _) = self.metadata.clone().into_normalized(&self.doc_type).unwrap();

        normalized_metadata
    }

    /// Signs this TestDocument into an [`Mdoc`] using `ca` and `key`.
    pub async fn sign<KEY>(self, ca: &Ca, device_key: &KEY) -> Mdoc
    where
        KEY: CredentialEcdsaKey,
    {
        let now = Utc::now();
        let issuer_signed = self.issuer_signed(ca, device_key, now).await;

        Mdoc::new::<KEY>(
            device_key.identifier().to_string(),
            issuer_signed,
            &MockTimeGenerator::new(now),
            &[ca.to_trust_anchor()],
        )
        .unwrap()
    }

    /// Generates an `IssuerSigned` for this `TestDocument`.
    pub async fn issuer_signed<KEY>(self, ca: &Ca, device_key: &KEY, now: chrono::DateTime<Utc>) -> IssuerSigned
    where
        KEY: CredentialEcdsaKey,
    {
        let name_spaces = IssuerNameSpaces::try_from(self.namespaces.clone()).unwrap();

        let mso = self.into_mobile_security_object(now, &name_spaces, device_key).await;

        let issuer_key_pair = ca
            .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
            .unwrap();

        let header = IssuerSigned::create_unprotected_header(issuer_key_pair.certificate().to_vec());

        let mso_tagged = TaggedBytes(mso);
        let issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso_tagged, header, &issuer_key_pair, true)
                .await
                .unwrap();

        IssuerSigned {
            name_spaces: name_spaces.into(),
            issuer_auth,
        }
    }

    async fn into_mobile_security_object<KEY>(
        self,
        now: chrono::DateTime<Utc>,
        name_spaces: &IssuerNameSpaces,
        device_key: &KEY,
    ) -> MobileSecurityObject
    where
        KEY: CredentialEcdsaKey,
    {
        let device_public_key = &device_key.verifying_key().await.unwrap();
        let cose_pubkey: CoseKey = device_public_key.try_into().unwrap();

        MobileSecurityObject {
            version: MobileSecurityObjectVersion::V1_0,
            digest_algorithm: DigestAlgorithm::SHA256,
            doc_type: self.doc_type,
            value_digests: (name_spaces).try_into().unwrap(),
            device_key_info: cose_pubkey.into(),
            validity_info: ValidityInfo {
                signed: now.into(),
                valid_from: now.into(),
                valid_until: (now + Duration::days(365)).into(),
                expected_update: None,
            },
            issuer_uri: Some(self.issuer_uri),
            attestation_qualification: Some(Default::default()),
            type_metadata_integrity: Some(self.metadata_integrity),
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn assert_matches(&self, disclosed_documents: &[DisclosedDocument]) {
        // verify the number of documents
        assert_eq!(disclosed_documents.len(), self.len());
        for (
            TestDocument {
                doc_type: expected_doc_type,
                namespaces: expected_namespaces,
                issuer_uri: expected_issuer,
                ..
            },
            disclosed_document,
        ) in self.0.iter().zip(disclosed_documents)
        {
            // verify the disclosed attributes
            assert_eq!(disclosed_document.doc_type, *expected_doc_type);
            // verify the issuer
            assert_eq!(disclosed_document.issuer_uri, *expected_issuer);
            // verify the number of namespaces in this document
            assert_eq!(disclosed_document.attributes.len(), expected_namespaces.len());
            for (expected_namespace, expected_attributes) in expected_namespaces {
                let disclosed_attributes = disclosed_document
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

impl std::ops::Add for TestDocuments {
    type Output = TestDocuments;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

impl MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> {
    pub fn doc_type(&self) -> Result<String, CoseError> {
        Ok(self.dangerous_parse_unverified()?.0.doc_type)
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod data {
    use attestation_types::claim_path::ClaimPath;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use dcql::ClaimsQuery;
    use dcql::ClaimsSelection;
    use dcql::CredentialQuery;
    use dcql::CredentialQueryFormat;
    use dcql::Query;
    use dcql::normalized::AttributeRequest;
    use dcql::normalized::NormalizedCredentialRequest;
    use utils::vec_at_least::VecNonEmpty;

    use super::*;

    pub const PID: &str = "urn:eudi:pid:nl:1";
    pub const ADDR: &str = "urn:eudi:pid-address:nl:1";
    pub const ADDR_NS: &str = "urn:eudi:pid-address:nl:1.address";

    pub fn empty() -> TestDocuments {
        vec![].into()
    }

    pub fn pid_example() -> TestDocuments {
        vec![TestDocument::new(
            PID.to_owned(),
            format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
            IndexMap::from_iter(vec![(
                PID.to_string(),
                vec![
                    Entry {
                        name: "bsn".to_string(),
                        value: Value::Text("999999999".to_string()),
                    },
                    Entry {
                        name: "given_name".to_string(),
                        value: Value::Text("Willeke Liselotte".to_string()),
                    },
                    Entry {
                        name: "family_name".to_string(),
                        value: Value::Text("De Bruijn".to_string()),
                    },
                ],
            )]),
            TypeMetadataDocuments::nl_pid_example(),
        )]
        .into()
    }

    pub fn pid_given_name() -> TestDocuments {
        vec![TestDocument::new(
            PID.to_owned(),
            format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
            IndexMap::from_iter(vec![(
                PID.to_string(),
                vec![Entry {
                    name: "given_name".to_string(),
                    value: Value::Text("Willeke Liselotte".to_string()),
                }],
            )]),
            TypeMetadataDocuments::nl_pid_example(),
        )]
        .into()
    }

    pub fn pid_family_name() -> TestDocuments {
        vec![TestDocument::new(
            PID.to_owned(),
            format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
            IndexMap::from_iter(vec![(
                PID.to_string(),
                vec![Entry {
                    name: "family_name".to_string(),
                    value: Value::Text("De Bruijn".to_string()),
                }],
            )]),
            TypeMetadataDocuments::nl_pid_example(),
        )]
        .into()
    }

    pub fn pid_full_name() -> TestDocuments {
        vec![TestDocument::new(
            PID.to_owned(),
            format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
            IndexMap::from_iter(vec![(
                PID.to_string(),
                vec![
                    Entry {
                        name: "family_name".to_string(),
                        value: Value::Text("De Bruijn".to_string()),
                    },
                    Entry {
                        name: "given_name".to_string(),
                        value: Value::Text("Willeke Liselotte".to_string()),
                    },
                ],
            )]),
            TypeMetadataDocuments::nl_pid_example(),
        )]
        .into()
    }

    pub fn addr_street() -> TestDocuments {
        vec![TestDocument::new(
            ADDR.to_owned(),
            format!("https://{ISSUANCE_CERT_CN}").parse().unwrap(),
            IndexMap::from_iter(vec![(
                ADDR_NS.to_string(),
                vec![Entry {
                    name: "street_address".to_string(),
                    value: Value::Text("Turfmarkt".to_string()),
                }],
            )]),
            TypeMetadataDocuments::address_example(),
        )]
        .into()
    }

    impl From<TestDocument> for NormalizedCredentialRequest {
        fn from(source: TestDocument) -> Self {
            let format = CredentialQueryFormat::MsoMdoc {
                doctype_value: source.doc_type,
            };

            // unwrap below is safe because claims path is not empty
            let claims = source
                .namespaces
                .into_iter()
                .flat_map(|(namespace, attrs)| {
                    attrs.into_iter().map(move |entry| AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(namespace.clone()),
                            ClaimPath::SelectByKey(entry.name),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain: true,
                    })
                })
                .collect();

            NormalizedCredentialRequest { format, claims }
        }
    }

    impl From<TestDocuments> for VecNonEmpty<NormalizedCredentialRequest> {
        fn from(source: TestDocuments) -> Self {
            source
                .0
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        }
    }

    fn credential_query_from((id, source): (usize, TestDocument)) -> CredentialQuery {
        CredentialQuery {
            id: format!("id-{id}"),
            format: CredentialQueryFormat::MsoMdoc {
                doctype_value: source.doc_type,
            },
            multiple: false,
            trusted_authorities: vec![],
            require_cryptographic_holder_binding: true,
            claims_selection: ClaimsSelection::All {
                claims: source
                    .namespaces
                    .into_iter()
                    .flat_map(|(ns, entries)| {
                        entries.into_iter().map(move |attr| ClaimsQuery {
                            id: None,
                            path: vec![ClaimPath::SelectByKey(ns.clone()), ClaimPath::SelectByKey(attr.name)]
                                .try_into()
                                .unwrap(),
                            values: vec![],
                            intent_to_retain: Some(true),
                        })
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            },
        }
    }

    impl From<TestDocuments> for Query {
        fn from(source: TestDocuments) -> Self {
            let credentials = source
                .0
                .into_iter()
                .enumerate()
                .map(credential_query_from)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            Self {
                credentials,
                credential_sets: vec![],
            }
        }
    }
}
