use attestation::auth::reader_auth::ReaderRegistration;
use attestation::identifiers::AttributeIdentifier;
use attestation::identifiers::AttributeIdentifierHolder;
use error_category::ErrorCategory;

use crate::ItemsRequest;

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum ValidationError {
    #[error("requested unregistered attributes: {0:?}")]
    #[category(critical)] // RP data, no user data
    UnregisteredAttributes(Vec<AttributeIdentifier>),
}

pub trait VerifyRequestedAttributesExt {
    fn verify_requested_attributes<'a, R>(&self, requested_attributes: &R) -> Result<(), ValidationError>
    where
        R: IntoIterator<Item = &'a ItemsRequest> + Clone;
}

impl VerifyRequestedAttributesExt for ReaderRegistration {
    /// Verify whether all requested attributes exist in the registration.
    fn verify_requested_attributes<'a, R>(&self, requested_attributes: &R) -> Result<(), ValidationError>
    where
        R: IntoIterator<Item = &'a ItemsRequest> + Clone,
    {
        let difference: Vec<AttributeIdentifier> = requested_attributes.difference(self).into_iter().collect();

        if !difference.is_empty() {
            return Err(ValidationError::UnregisteredAttributes(difference));
        }

        Ok(())
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use indexmap::IndexMap;

    use attestation::auth::reader_auth::AuthorizedAttribute;
    use attestation::auth::reader_auth::AuthorizedMdoc;
    use attestation::auth::reader_auth::AuthorizedNamespace;
    use attestation::auth::reader_auth::ReaderRegistration;

    use crate::verifier::ItemsRequests;

    pub fn reader_registration_mock_from_requests(authorized_requests: &ItemsRequests) -> ReaderRegistration {
        let attributes = authorized_requests
            .0
            .iter()
            .map(|items_request| {
                let namespaces: IndexMap<_, _> = items_request
                    .name_spaces
                    .iter()
                    .map(|(namespace, attributes)| {
                        let authorized_attributes = attributes
                            .iter()
                            .map(|attribute| (attribute.0.clone(), AuthorizedAttribute {}))
                            .collect();
                        (namespace.clone(), AuthorizedNamespace(authorized_attributes))
                    })
                    .collect();
                (items_request.doc_type.clone(), AuthorizedMdoc(namespaces))
            })
            .collect();
        ReaderRegistration {
            attributes,
            ..ReaderRegistration::new_mock()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use attestation::auth::reader_auth::test::create_registration;
    use attestation::auth::reader_auth::test::create_some_registration;
    use attestation::auth::reader_auth::test::DocTypes;

    use crate::DeviceRequest;
    use crate::ItemsRequest;

    use super::*;

    #[test]
    fn verify_requested_attributes_in_device_request() {
        let device_request = DeviceRequest::from_items_requests(vec![
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
        let registration = create_some_registration();
        registration
            .verify_requested_attributes(&device_request.items_requests())
            .unwrap();
    }

    #[test]
    fn verify_requested_attributes_in_device_request_missing() {
        let device_request = DeviceRequest::from_items_requests(vec![
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
        let registration = create_some_registration();
        let result = registration.verify_requested_attributes(&device_request.items_requests());
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
        let request = DeviceRequest::from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_some_registration();
        registration
            .verify_requested_attributes(&request.items_requests())
            .unwrap();
    }

    #[test]
    fn validate_items_request_missing_attribute() {
        let request = DeviceRequest::from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["missing_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request.items_requests());
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_namespace() {
        let request = DeviceRequest::from_items_requests(vec![create_items_request(vec![(
            "some_doctype",
            vec![("missing_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request.items_requests());
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_doctype() {
        let request = DeviceRequest::from_items_requests(vec![create_items_request(vec![(
            "missing_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )])]);
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request.items_requests());
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    // Utility function to easily create [`ItemsRequest`]
    fn create_items_request(mut request_doctypes: DocTypes) -> ItemsRequest {
        // An [`ItemRequest`] can only contain 1 doctype
        assert_eq!(request_doctypes.len(), 1);
        let (doc_type, namespaces) = request_doctypes.remove(0);

        let mut name_spaces = IndexMap::new();
        for (namespace, attrs) in namespaces {
            let mut attribute_map = IndexMap::new();
            for attr in attrs {
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
}
