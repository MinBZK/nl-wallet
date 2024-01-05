use std::{fmt::Debug, iter};

use futures::{executor, future};
use indexmap::IndexMap;
use p256::ecdsa::{Signature, VerifyingKey};
use webpki::TrustAnchor;

use wallet_common::{
    keys::{software::SoftwareEcdsaKey, ConstructibleWithIdentifier, EcdsaKey, SecureEcdsaKey, WithIdentifier},
    utils,
};

use crate::{
    examples::{Example, Examples, IsoCertTimeGenerator},
    holder::Mdoc,
    identifiers::AttributeIdentifier,
    iso::{disclosure::DeviceResponse, mdocs::DataElementValue},
    server_keys::PrivateKey,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey, MdocKeyType},
        reader_auth::{AuthorizedAttribute, AuthorizedMdoc, AuthorizedNamespace},
        x509::{Certificate, CertificateError, CertificateType},
    },
    verifier::DisclosedAttributes,
};

/// Out of the example data structures in the standard, assemble an mdoc.
/// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the originating
/// mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some [`IssuerSignedItem`]
/// instances, i.e. attributes.
/// This is because the other attributes are actually nowhere present in the standard so it is impossible to construct
/// the example mdoc with all attributes present.
///
/// Using tests should not rely on all attributes being present.
pub fn mdoc_from_example_device_response(trust_anchors: &[TrustAnchor<'_>]) -> Mdoc {
    // Prepare the mdoc's private key
    let static_device_key = Examples::static_device_key();
    SoftwareEcdsaKey::insert("example_static_device_key", static_device_key);

    let issuer_signed = DeviceResponse::example().documents.as_ref().unwrap()[0]
        .issuer_signed
        .clone();

    Mdoc::new::<SoftwareEcdsaKey>(
        "example_static_device_key".to_string(),
        issuer_signed,
        &IsoCertTimeGenerator,
        trust_anchors,
    )
    .unwrap()
}

const ISSUANCE_CA_CN: &str = "ca.issuer.example.com";
const ISSUANCE_CERT_CN: &str = "cert.issuer.example.com";

pub fn generate_issuance_key_and_ca() -> Result<(PrivateKey, Certificate), CertificateError> {
    // Issuer CA certificate and normal certificate
    let (ca, ca_privkey) = Certificate::new_ca(ISSUANCE_CA_CN)?;
    let (issuer_cert, issuer_privkey) = Certificate::new(&ca, &ca_privkey, ISSUANCE_CERT_CN, CertificateType::Mdl)?;
    let issuance_key = PrivateKey::new(issuer_privkey, issuer_cert);

    Ok((issuance_key, ca))
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AttributeIdParsingError {
    #[error("Expected string with 3 parts separated by '/', got {0} parts")]
    InvalidPartsCount(usize),
}

// This implementation is solely intended for unit testing purposes to easily construct AttributeIdentifiers.
// This implementation should never end up in production code, because the use of '/' is officially allowed in the
// various parts.
impl std::str::FromStr for AttributeIdentifier {
    type Err = AttributeIdParsingError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let parts = source.split('/').collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(AttributeIdParsingError::InvalidPartsCount(parts.len()));
        }
        let result = Self {
            doc_type: parts[0].to_owned(),
            namespace: parts[1].to_owned(),
            attribute: parts[2].to_owned(),
        };
        Ok(result)
    }
}

/// The [`FactorySoftwareEcdsaKey`] type wraps [`SoftwareEcdsaKey`] and has
/// the possibility of returning [`FactorySoftwareEcdsaKeyError`] when signing.
pub struct FactorySoftwareEcdsaKey {
    key: SoftwareEcdsaKey,
    has_signing_error: bool,
}

#[derive(Debug, Default, thiserror::Error)]
#[error("FactorySoftwareEcdsaKeyError")]
pub struct FactorySoftwareEcdsaKeyError {}

impl MdocEcdsaKey for FactorySoftwareEcdsaKey {
    const KEY_TYPE: MdocKeyType = MdocKeyType::Software;
}
impl SecureEcdsaKey for FactorySoftwareEcdsaKey {}
impl EcdsaKey for FactorySoftwareEcdsaKey {
    type Error = FactorySoftwareEcdsaKeyError;

    async fn verifying_key(&self) -> Result<VerifyingKey, Self::Error> {
        let verifying_key = self.key.verifying_key().await.unwrap();

        Ok(verifying_key)
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Signature, Self::Error> {
        if self.has_signing_error {
            return Err(FactorySoftwareEcdsaKeyError::default());
        }

        let signature = self.key.try_sign(msg).await.unwrap();

        Ok(signature)
    }
}
impl WithIdentifier for FactorySoftwareEcdsaKey {
    fn identifier(&self) -> &str {
        self.key.identifier()
    }
}

/// The [`SoftwareKeyFactory`] type implements [`KeyFactory`] and has the option
/// of returning [`SoftwareKeyFactoryError`] when generating keys, as well as generating
/// [`FactorySoftwareEcdsaKey`] that return [`FactorySoftwareEcdsaKeyError`] when signing.
#[derive(Debug, Default)]
pub struct SoftwareKeyFactory {
    pub has_generating_error: bool,
    pub has_key_signing_error: bool,
}

#[derive(Debug, Default, thiserror::Error)]
#[error("SoftwareKeyFactoryError")]
pub struct SoftwareKeyFactoryError {}

impl SoftwareKeyFactory {
    fn new_key(&self, identifier: &str) -> FactorySoftwareEcdsaKey {
        FactorySoftwareEcdsaKey {
            key: SoftwareEcdsaKey::new(identifier),
            has_signing_error: self.has_key_signing_error,
        }
    }
}

impl KeyFactory for SoftwareKeyFactory {
    type Key = FactorySoftwareEcdsaKey;
    type Error = SoftwareKeyFactoryError;

    async fn generate_new_multiple(&self, count: u64) -> Result<Vec<Self::Key>, Self::Error> {
        if self.has_generating_error {
            return Err(SoftwareKeyFactoryError::default());
        }

        let keys = iter::repeat_with(|| self.new_key(&utils::random_string(32)))
            .take(count as usize)
            .collect();

        Ok(keys)
    }

    fn generate_existing<I: Into<String> + Send>(&self, identifier: I, public_key: VerifyingKey) -> Self::Key {
        let key = self.new_key(&identifier.into());

        // If the provided public key does not match the key fetched
        // using the identifier, this is programmer error.
        assert_eq!(executor::block_on(key.verifying_key()).unwrap(), public_key);

        key
    }

    async fn sign_with_new_keys<T: Into<Vec<u8>> + Send>(
        &self,
        msg: T,
        number_of_keys: u64,
    ) -> Result<Vec<(Self::Key, Signature)>, Self::Error> {
        let keys = self.generate_new_multiple(number_of_keys).await?;
        let bytes = msg.into();

        let signatures_by_identifier = future::join_all(keys.into_iter().map(|key| async {
            let signature = SoftwareEcdsaKey::new(key.identifier())
                .try_sign(bytes.as_slice())
                .await
                .unwrap();

            (key, signature)
        }))
        .await
        .into_iter()
        .collect();

        Ok(signatures_by_identifier)
    }
}

/// Build attributes for [`ReaderRegistration`] from a list of attributes.
pub fn reader_registration_attributes(
    doc_type: String,
    name_space: String,
    attributes: impl Iterator<Item = impl Into<String>>,
) -> IndexMap<String, AuthorizedMdoc> {
    [(
        doc_type,
        AuthorizedMdoc(
            [(
                name_space,
                AuthorizedNamespace(
                    attributes
                        .map(|attribute| (attribute.into(), AuthorizedAttribute {}))
                        .collect(),
                ),
            )]
            .into(),
        ),
    )]
    .into()
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
/// use nl_wallet_mdoc::mock::DebugCollapseBts;
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

/// Assert that the specified doctype was disclosed, and that it contained the specified namespace,
/// and that the first attribute in that namespace has the specified name and value.
pub fn assert_disclosure_contains(
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
