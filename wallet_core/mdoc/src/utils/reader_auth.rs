use std::str::FromStr;

use indexmap::{IndexMap, IndexSet};
use p256::pkcs8::der::{asn1::Utf8StringRef, Decode, SliceReader};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DeserializeFromStr};
use url::Url;
use x509_parser::der_parser::Oid;

use crate::{
    identifiers::{AttributeIdentifier, AttributeIdentifierHolder},
    utils::x509::{Certificate, CertificateError},
    DeviceRequest,
};

/// oid: 2.1.123.1
/// root: {joint-iso-itu-t(2) asn1(1) examples(123)}
/// suffix: 1, unofficial id for Reader Authentication
const OID_EXT_READER_AUTH: &[u64] = &[2, 1, 123, 1];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReturnUrlPrefixError {
    #[error("scheme '{0}' is not 'https://'")]
    InvalidScheme(String),
    #[error("path '{0}' is invalid, must end with a '/'")]
    InvalidPath(String),
    #[error("query '{0}' is invalid, it must be None")]
    InvalidQuery(String),
    #[error("fragment '{0}' is invalid, it must be None")]
    InvalidFragment(String),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// URL that must match as a prefix for the return URL in a SameDevice disclosure flow. This
/// URL may only have 'https' as its scheme, a non-empty domain, a path that ends with a '/'
/// and no query or fragment. Note that the non-empty domain is handled by the `rust-url`
/// crate.
#[derive(Debug, Clone, DeserializeFromStr, Serialize, PartialEq, Eq)]
pub struct ReturnUrlPrefix(Url);

impl ReturnUrlPrefix {
    pub fn matches_url(&self, url: &Url) -> bool {
        url.authority() == self.0.authority()
            && url.path().starts_with(self.0.path())
            && url.path().len() > self.0.path().len()
    }
}

#[cfg(any(test, feature = "mock"))]
impl Default for ReturnUrlPrefix {
    fn default() -> Self {
        ReturnUrlPrefix("https://example.com/".parse().unwrap())
    }
}

impl FromStr for ReturnUrlPrefix {
    type Err = ReturnUrlPrefixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Url::parse(s)?.try_into()
    }
}

impl TryFrom<Url> for ReturnUrlPrefix {
    type Error = ReturnUrlPrefixError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        #[cfg(feature = "allow_http_return_url")]
        let allowed_schemes = ["https", "http"];
        #[cfg(not(feature = "allow_http_return_url"))]
        let allowed_schemes = ["https"];

        if !allowed_schemes.contains(&url.scheme()) {
            Err(ReturnUrlPrefixError::InvalidScheme(url.scheme().to_owned()))
        } else if !url.path().ends_with('/') {
            Err(ReturnUrlPrefixError::InvalidPath(url.path().to_owned()))
        } else if url.query().is_some() {
            Err(ReturnUrlPrefixError::InvalidQuery(url.query().unwrap().to_owned()))
        } else if url.fragment().is_some() {
            Err(ReturnUrlPrefixError::InvalidFragment(
                url.fragment().unwrap().to_owned(),
            ))
        } else {
            Ok(ReturnUrlPrefix(url))
        }
    }
}

impl From<ReturnUrlPrefix> for Url {
    fn from(value: ReturnUrlPrefix) -> Self {
        value.0
    }
}

#[skip_serializing_none]
#[cfg_attr(any(test, feature = "mock"), derive(Default))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderRegistration {
    pub id: String,
    pub purpose_statement: LocalizedStrings,
    pub retention_policy: RetentionPolicy,
    pub sharing_policy: SharingPolicy,
    pub deletion_policy: DeletionPolicy,
    pub organization: Organization,
    pub return_url_prefix: ReturnUrlPrefix,
    pub attributes: IndexMap<String, AuthorizedMdoc>,
}

impl ReaderRegistration {
    pub fn from_certificate(source: &Certificate) -> Result<Option<Self>, CertificateError> {
        // unwrap() is safe here, because we process a fixed value
        let oid = Oid::from(OID_EXT_READER_AUTH).unwrap();
        let x509_cert = source.to_x509()?;
        let ext = x509_cert.iter_extensions().find(|ext| ext.oid == oid);
        let registration = match ext {
            Some(ext) => {
                let mut reader = SliceReader::new(ext.value)?;
                let json = Utf8StringRef::decode(&mut reader)?;
                let registration = serde_json::from_str(json.as_str())?;
                Some(registration)
            }
            None => None,
        };
        Ok(registration)
    }
}

impl AttributeIdentifierHolder for ReaderRegistration {
    fn attribute_identifiers(&self) -> IndexSet<AttributeIdentifier> {
        self.attributes
            .iter()
            .flat_map(|(doc_type, AuthorizedMdoc(namespaces))| {
                namespaces
                    .into_iter()
                    .flat_map(|(namespace, AuthorizedNamespace(attributes))| {
                        attributes.into_iter().map(|(attribute, _)| AttributeIdentifier {
                            doc_type: doc_type.to_owned(),
                            namespace: namespace.to_owned(),
                            attribute: attribute.to_owned(),
                        })
                    })
            })
            .collect()
    }
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub display_name: LocalizedStrings,
    pub legal_name: LocalizedStrings,
    pub description: LocalizedStrings,
    pub category: LocalizedStrings,
    pub logo: Option<Image>,
    pub web_url: Option<Url>,
    pub kvk: Option<String>,
    pub city: Option<LocalizedStrings>,
    pub department: Option<LocalizedStrings>,
    pub country_code: Option<String>,
    pub privacy_policy_url: Option<Url>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageType {
    #[default]
    #[serde(rename = "image/svg+xml")]
    Svg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

/// Encapsulates an image.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// Media Type of the image, expected to start with: `image/`.
    pub mime_type: ImageType,
    /// String encoded data of the image, f.e. XML text for `image/xml+svg`, or Base64 encoded binary data for
    /// `image/png`.
    pub image_data: String,
}

type Language = String;

/// Holds multiple translations of the same field
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedStrings(pub IndexMap<Language, String>);

/// Allows convenient definitions of [`LocalizedStrings`] in Rust code.
impl From<Vec<(&str, &str)>> for LocalizedStrings {
    fn from(source: Vec<(&str, &str)>) -> Self {
        let mut map = IndexMap::new();
        for (language, value) in source.into_iter() {
            map.insert(language.to_owned(), value.to_owned());
        }
        LocalizedStrings(map)
    }
}

#[serde_as]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub intent_to_retain: bool,
    pub max_duration_in_minutes: Option<u64>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharingPolicy {
    pub intent_to_share: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionPolicy {
    pub deleteable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedMdoc(pub IndexMap<String, AuthorizedNamespace>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedNamespace(pub IndexMap<String, AuthorizedAttribute>);

// This struct could be extended in the future for attribute specific policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedAttribute {}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("requested unregistered attributes: {0:?}")]
    UnregisteredAttributes(Vec<AttributeIdentifier>),
}

impl DeviceRequest {
    /// Verify whether all requested attributes exist in the `registration`.
    pub fn verify_requested_attributes(&self, reader_registration: &ReaderRegistration) -> Result<(), ValidationError> {
        let difference: Vec<AttributeIdentifier> = self.difference(reader_registration).into_iter().collect();

        if !difference.is_empty() {
            return Err(ValidationError::UnregisteredAttributes(difference));
        }

        Ok(())
    }
}

#[cfg(feature = "generate")]
mod generate {
    use p256::pkcs8::der::{asn1::Utf8StringRef, Encode};
    use rcgen::CustomExtension;

    use crate::utils::{
        reader_auth::{ReaderRegistration, OID_EXT_READER_AUTH},
        x509::CertificateError,
    };

    impl ReaderRegistration {
        pub fn to_custom_ext(&self) -> Result<CustomExtension, CertificateError> {
            let json_string = serde_json::to_string(self)?;
            let string = Utf8StringRef::new(&json_string)?;
            let ext = CustomExtension::from_oid_content(OID_EXT_READER_AUTH, string.to_der()?);
            Ok(ext)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{utils::serialization::TaggedBytes, DeviceRequestVersion, DocRequest, ItemsRequest};

    use super::*;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use rstest::rstest;

    #[rstest]
    #[case("https://example/", Ok(()))]
    #[case("https://example.com/", Ok(()))]
    #[case(
        "https://example.com/path/",
        Ok(())
    )]
    #[case(
        "https://example.com/some/path/",
        Ok(())
    )]
    #[case("https://example", Ok(()))] // `"https://example".parse().unwrap().to_string() == "https://example.com/"`
    #[case("https://example.com", Ok(()))] // `"https://example.com".parse().unwrap().to_string() == "https://example.com/"`
    #[cfg_attr(
        feature = "allow_http_return_url",
        case("http://example.com", Ok(()))
    )]
    #[case("file://etc/passwd", Err(ReturnUrlPrefixError::InvalidScheme("file".to_owned())))]
    #[cfg_attr(
        not(feature = "allow_http_return_url"),
        case("http://example.com", Err(ReturnUrlPrefixError::InvalidScheme("http".to_owned())))
    )]
    #[case("https://", Err(ReturnUrlPrefixError::UrlParse(url::ParseError::EmptyHost)))] // test for non-empty domain clause
    #[case("https://etc/passwd", Err(ReturnUrlPrefixError::InvalidPath("/passwd".to_owned())))]
    #[case("https://example.com/path/?", Err(ReturnUrlPrefixError::InvalidQuery("".to_owned())))]
    #[case("https://example.com/path/?hello", Err(ReturnUrlPrefixError::InvalidQuery("hello".to_owned())))]
    #[case(
        "https://example.com/path/?hello=world",
        Err(ReturnUrlPrefixError::InvalidQuery("hello=world".to_owned()))
    )]
    #[case("https://example.com/path/#", Err(ReturnUrlPrefixError::InvalidFragment("".to_owned())))]
    #[case(
        "https://example.com/path/#hello",
        Err(ReturnUrlPrefixError::InvalidFragment("hello".to_owned()))
    )]
    #[case("", Err(ReturnUrlPrefixError::UrlParse(url::ParseError::RelativeUrlWithoutBase)))]
    fn test_return_url_prefix_parse(#[case] value: &str, #[case] expected_err: Result<(), ReturnUrlPrefixError>) {
        assert_eq!(value.parse::<ReturnUrlPrefix>().map(|_| ()), expected_err);

        let result = serde_json::from_str::<ReturnUrlPrefix>(&format!("\"{}\"", value));
        assert!((result.is_ok() && expected_err.is_ok()) || (result.is_err() && expected_err.is_err()));

        // if it doesn't parse as a URL, this check doesn't make sense
        if let Ok(url) = value.parse::<Url>() {
            assert_eq!(
                std::convert::TryInto::<ReturnUrlPrefix>::try_into(url).map(|_| ()),
                expected_err
            );
        }
    }

    #[rstest]
    #[case("https://example.com/", "https://example.com/session", true)]
    #[case("https://example.com/", "https://example.com/session/more/path", true)]
    #[case("https://example.com/", "https://example.com/session?query=foo&query2=bar", true)]
    #[case("https://example.com/", "https://example.com/session#fragment", true)]
    #[case(
        "https://example.com/",
        "https://example.com/session?query=foo&query2=bar#fragment",
        true
    )]
    #[case("https://example.com/path/", "https://example.com/path/session", true)]
    #[case("https://example.com/path/", "https://example.com/path/session/more/path", true)]
    #[case(
        "https://user:password@example.com/",
        "https://user:password@example.com/session",
        true
    )]
    #[case(
        "https://user:password@example.com/",
        "https://user:password@example.com/session/more/path",
        true
    )]
    #[case("https://example.com:8080/", "https://example.com:8080/session", true)]
    #[case("https://example.com:8080/", "https://example.com:8080/session/more/path", true)]
    #[case("https://example.com/", "https://example.com/", false)]
    #[case("https://example.com/path/", "https://example.com/path/", false)]
    #[case("https://example.com/path/", "https://example.com/session", false)]
    #[case("https://example.com/path/more/", "https://example.com/path/session", false)]
    #[case("https://user:password@example.com/", "https://example.com/session", false)]
    #[case("https://example.com:8080/", "https://example.com:8443/session", false)]
    #[case("https://example.com:80/", "https://example.com/session", false)]
    #[case("https://example.com/", "https://example.com:80/session", false)]
    fn test_return_url_prefix_matches_url(
        #[case] return_url_prefix: ReturnUrlPrefix,
        #[case] url: Url,
        #[case] expected_match: bool,
    ) {
        assert_eq!(return_url_prefix.matches_url(&url), expected_match);
    }

    #[test]
    fn verify_requested_attributes_in_device_request() {
        let device_request = device_request_from_items_requests(vec![
            create_items_request(vec![(
                "some_doctype",
                vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
            )]),
            create_items_request(vec![(
                "some_doctype",
                vec![("another_namespace", vec!["some_attribute", "another_attribute"])],
            )]),
            create_items_request(vec![(
                "another_doctype",
                vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
            )]),
        ]);
        let registration = create_registration(vec![
            (
                "some_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
            (
                "another_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
        ]);
        device_request.verify_requested_attributes(&registration).unwrap();
    }

    #[test]
    fn verify_requested_attributes_in_device_request_missing() {
        let device_request = device_request_from_items_requests(vec![
            create_items_request(vec![(
                "some_doctype",
                vec![("some_namespace", vec!["some_attribute", "missing_attribute"])],
            )]),
            create_items_request(vec![(
                "some_doctype",
                vec![("missing_namespace", vec!["some_attribute", "another_attribute"])],
            )]),
            create_items_request(vec![(
                "missing_doctype",
                vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
            )]),
        ]);
        let registration = create_registration(vec![
            (
                "some_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
            (
                "another_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
        ]);
        let result = device_request.verify_requested_attributes(&registration);
        assert_matches!(
            result,
            Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
                "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
                "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
                "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
                "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
                "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn validate_items_request() {
        let request = device_request_from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![
            (
                "some_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
            (
                "another_doctype",
                vec![
                    ("some_namespace", vec!["some_attribute", "another_attribute"]),
                    ("another_namespace", vec!["some_attribute", "another_attribute"]),
                ],
            ),
        ]);
        request.verify_requested_attributes(&registration).unwrap();
    }

    #[test]
    fn validate_items_request_missing_attribute() {
        let request = device_request_from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["missing_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = request.verify_requested_attributes(&registration);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_namespace() {
        let request = device_request_from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("missing_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = request.verify_requested_attributes(&registration);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_doctype() {
        let request = device_request_from_items_requests(vec![create_items_request(vec![(
            "missing_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = request.verify_requested_attributes(&registration);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    type Attributes<'a> = Vec<&'a str>;
    type Namespaces<'a> = Vec<(&'a str, Attributes<'a>)>;
    type DocTypes<'a> = Vec<(&'a str, Namespaces<'a>)>;

    // Utility function to easily create [`ItemsRequest`]
    fn create_items_request(mut request_doctypes: DocTypes) -> ItemsRequest {
        // An [`ItemRequest`] can only contain 1 doctype
        assert_eq!(request_doctypes.len(), 1);
        let (doc_type, namespaces) = request_doctypes.remove(0);

        let mut name_spaces = IndexMap::new();
        for (namespace, attrs) in namespaces.into_iter() {
            let mut attribute_map = IndexMap::new();
            for attr in attrs.into_iter() {
                attribute_map.insert(attr.to_owned(), true);
            }
            name_spaces.insert(namespace.to_owned(), attribute_map);
        }

        ItemsRequest {
            doc_type: doc_type.to_owned(),
            name_spaces,
            request_info: None,
        }
    }

    // Utility function to easily create [`ReaderRegistration`]
    fn create_registration(registered_doctypes: DocTypes) -> ReaderRegistration {
        let mut attributes = IndexMap::new();
        for (doc_type, namespaces) in registered_doctypes.into_iter() {
            let mut namespace_map = IndexMap::new();
            for (ns, attrs) in namespaces.into_iter() {
                let mut attribute_map = IndexMap::new();
                for attr in attrs.into_iter() {
                    attribute_map.insert(attr.to_owned(), AuthorizedAttribute {});
                }
                namespace_map.insert(ns.to_owned(), AuthorizedNamespace(attribute_map));
            }
            attributes.insert(doc_type.to_owned(), AuthorizedMdoc(namespace_map));
        }

        ReaderRegistration {
            attributes,
            ..Default::default()
        }
    }

    fn doc_request_from_items_request(items_request: ItemsRequest) -> DocRequest {
        DocRequest {
            items_request: TaggedBytes(items_request),
            reader_auth: None,
        }
    }

    fn device_request_from_items_requests(items_requests: Vec<ItemsRequest>) -> DeviceRequest {
        DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests: items_requests.into_iter().map(doc_request_from_items_request).collect(),
        }
    }
}
