use std::collections::HashMap;
use std::collections::HashSet;

use attestation_types::attribute_paths::AttestationAttributePaths;
use attestation_types::attribute_paths::AttestationAttributePathsError;
use utils::vec_at_least::VecNonEmpty;

use crate::verifier::ItemsRequests;

impl ItemsRequests {
    pub fn try_into_attribute_paths(self) -> Result<AttestationAttributePaths, AttestationAttributePathsError> {
        let Self(requests) = self;

        let paths = requests
            .into_iter()
            .fold(HashMap::<_, HashSet<_>>::new(), |mut paths, request| {
                // For an mdoc items request, simply make a path of length 2,
                // consisting of the name space and element identifier.
                let attributes = request
                    .name_spaces
                    .into_iter()
                    .flat_map(|(namespace, elements)| {
                        let element_count = elements.len();

                        elements
                            .into_keys()
                            .zip(itertools::repeat_n(namespace, element_count))
                            .map(|(element_id, namespace)| VecNonEmpty::try_from(vec![namespace, element_id]).unwrap())
                    })
                    .collect::<HashSet<_>>();

                // In case a doc type occurs multiple times, merge the paths.
                paths.entry(request.doc_type).or_default().extend(attributes);

                paths
            });

        AttestationAttributePaths::try_new(paths)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use assert_matches::assert_matches;

    use attestation_types::attribute_paths::AttestationAttributePathsError;

    use crate::iso::device_retrieval::ItemsRequest;
    use crate::verifier::ItemsRequests;

    #[test]
    fn test_items_requests_try_into_attribute_paths_ok() {
        let items_requests = ItemsRequests::from(vec![ItemsRequest::new_example()]);

        let attribute_paths = items_requests
            .try_into_attribute_paths()
            .expect("converting ItemsRequests into AttestationAttributePaths should succeed");

        let expected_attribute_paths = HashMap::from([(
            "org.iso.18013.5.1.mDL".to_string(),
            HashSet::from([
                vec!["org.iso.18013.5.1".to_string(), "family_name".to_string()]
                    .try_into()
                    .unwrap(),
                vec!["org.iso.18013.5.1".to_string(), "issue_date".to_string()]
                    .try_into()
                    .unwrap(),
                vec!["org.iso.18013.5.1".to_string(), "expiry_date".to_string()]
                    .try_into()
                    .unwrap(),
                vec!["org.iso.18013.5.1".to_string(), "document_number".to_string()]
                    .try_into()
                    .unwrap(),
                vec!["org.iso.18013.5.1".to_string(), "driving_privileges".to_string()]
                    .try_into()
                    .unwrap(),
            ]),
        )]);

        assert_eq!(attribute_paths.as_ref(), &expected_attribute_paths);
    }

    #[test]
    fn test_items_requests_try_into_attribute_paths_error() {
        let items_requests = ItemsRequests::from(vec![ItemsRequest::new_example_empty()]);

        let error = items_requests
            .try_into_attribute_paths()
            .expect_err("converting ItemsRequests into AttestationAttributePaths not should succeed");

        assert_matches!(
            error,
            AttestationAttributePathsError::EmptyAttributes(attestation_type)
                if attestation_type == HashSet::from(["org.iso.18013.5.1.mDL".to_string()])
        );
    }
}
