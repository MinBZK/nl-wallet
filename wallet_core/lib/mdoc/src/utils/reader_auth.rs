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
