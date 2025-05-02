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

    use attestation::auth::reader_auth::mock::create_registration;
    use attestation::auth::reader_auth::mock::create_some_registration;
    use attestation::auth::reader_auth::ValidationError;
    use attestation::identifiers::mock::MockAttributeIdentifierHolder;

    #[test]
    fn verify_requested_attributes_in_device_request() {
        let device_request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
            "some_doctype/another_namespace/some_attribute".parse().unwrap(),
            "some_doctype/another_namespace/another_attribute".parse().unwrap(),
            "another_doctype/some_namespace/some_attribute".parse().unwrap(),
            "another_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        registration.verify_requested_attributes(&device_request).unwrap();
    }

    #[test]
    fn verify_requested_attributes_in_device_request_missing() {
        let device_request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        let result = registration.verify_requested_attributes(&device_request);
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
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/some_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_some_registration();
        registration.verify_requested_attributes(&request).unwrap();
    }

    #[test]
    fn validate_items_request_missing_attribute() {
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
            "some_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/some_namespace/missing_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_namespace() {
        let request: MockAttributeIdentifierHolder = vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "some_doctype/missing_namespace/some_attribute".parse().unwrap(),
            "some_doctype/missing_namespace/another_attribute".parse().unwrap(),
        ]);
    }

    #[test]
    fn validate_items_request_missing_doctype() {
        let request: MockAttributeIdentifierHolder = vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]
        .into();
        let registration = create_registration(vec![(
            "some_doctype",
            vec![("some_namespace", vec!["some_attribute", "another_attribute"])],
        )]);

        let result = registration.verify_requested_attributes(&request);
        assert_matches!(result, Err(ValidationError::UnregisteredAttributes(attrs)) if attrs == vec![
            "missing_doctype/some_namespace/some_attribute".parse().unwrap(),
            "missing_doctype/some_namespace/another_attribute".parse().unwrap(),
        ]);
    }
}
