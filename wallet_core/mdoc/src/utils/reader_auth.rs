use indexmap::{IndexMap, IndexSet};
use p256::pkcs8::der::{asn1::Utf8StringRef, Decode, SliceReader};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
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

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderRegistration {
    pub id: String,
    pub purpose_statement: LocalizedStrings,
    pub retention_policy: RetentionPolicy,
    pub sharing_policy: SharingPolicy,
    pub deletion_policy: DeletionPolicy,
    pub organization: Organization,
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
    pub logo: Option<Image>,
    pub web_url: Option<Url>,
    pub kvk: Option<String>,
    pub city: Option<LocalizedStrings>,
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
