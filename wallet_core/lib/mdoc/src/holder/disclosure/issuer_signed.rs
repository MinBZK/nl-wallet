use std::collections::HashSet;

use indexmap::IndexMap;
use itertools::Itertools;

use crate::iso::disclosure::IssuerSigned;
use crate::iso::mdocs::Attributes;
use crate::utils::serialization::TaggedBytes;

impl IssuerSigned {
    pub fn matches_attribute_paths(&self, attribute_paths: &HashSet<(&str, &str)>) -> bool {
        self.name_spaces
            .as_ref()
            .map(|name_spaces| {
                attribute_paths.iter().all(|(name_space, element_id)| {
                    name_spaces
                        .as_ref()
                        .get(*name_space)
                        .map(|attributes| {
                            attributes
                                .as_ref()
                                .iter()
                                .any(|TaggedBytes(signed_item)| signed_item.element_identifier == *element_id)
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    pub fn into_attribute_subset(self, attribute_paths: &HashSet<(&str, &str)>) -> Self {
        let Some(name_spaces) = self.name_spaces else {
            return self;
        };

        // Remove all of the attributes that are not listed in attribute_paths,
        // which may cause name_spaces to be returned as None.
        let name_spaces = name_spaces
            .into_iter()
            .flat_map(|(name_space, attributes)| {
                let attributes = attributes
                    .into_inner()
                    .into_iter()
                    .filter(|TaggedBytes(signed_item)| {
                        attribute_paths.contains(&(name_space.as_str(), signed_item.element_identifier.as_str()))
                    })
                    .collect_vec();

                // This will return None if the attributes are empty and will subsequently be filtered out.
                Attributes::try_from(attributes)
                    .ok()
                    .map(|attributes| (name_space, attributes))
            })
            .collect::<IndexMap<_, _>>()
            .try_into()
            .ok();

        Self { name_spaces, ..self }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use rstest::rstest;

    use crate::examples::Example;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::disclosure::IssuerSigned;
    use crate::utils::serialization::TaggedBytes;
    use crate::IssuerNameSpaces;

    fn issuer_signed_example() -> IssuerSigned {
        DeviceResponse::example()
            .documents
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .issuer_signed
    }

    #[rstest]
    #[case(HashSet::new(), true)]
    #[case(HashSet::from([("org.iso.18013.5.1", "family_name")]), true)]
    #[case(
        HashSet::from([
            ("org.iso.18013.5.1", "family_name"),
            ("org.iso.18013.5.1", "issue_date"),
            ("org.iso.18013.5.1", "expiry_date"),
            ("org.iso.18013.5.1", "document_number"),
            ("org.iso.18013.5.1", "portrait"),
            ("org.iso.18013.5.1", "driving_privileges"),
        ]),
        true
    )]
    #[case(HashSet::from([("org.iso.18013.5.1", "is_rich")]), false)]
    #[case(HashSet::from([("org.iso.18013.5.1", "family_name"), ("org.iso.18013.5.1", "is_rich")]), false)]
    #[case(HashSet::from([("org.iso.18013.5.1", "family_name"), ("vroom", "driving_privileges")]), false)]
    fn test_issuer_signed_matches_attribute_paths(
        #[case] attribute_paths: HashSet<(&str, &str)>,
        #[case] expected_matches: bool,
    ) {
        let matches = issuer_signed_example().matches_attribute_paths(&attribute_paths);

        assert_eq!(matches, expected_matches);
    }

    #[rstest]
    #[case(HashSet::new(), HashSet::new())]
    #[case(HashSet::from([("foo", "bar"), ("bleh", "blah")]), HashSet::new())]
    #[case(
        HashSet::from([("org.iso.18013.5.1", "family_name")]),
        HashSet::from([("org.iso.18013.5.1", "family_name")]),
    )]
    #[case(
        HashSet::from([
            ("org.iso.18013.5.1", "family_name"),
            ("org.iso.18013.5.1", "issue_date"),
            ("org.iso.18013.5.1", "expiry_date"),
            ("org.iso.18013.5.1", "document_number"),
            ("org.iso.18013.5.1", "portrait"),
            ("org.iso.18013.5.1", "driving_privileges")
        ]),
        HashSet::from([
            ("org.iso.18013.5.1", "family_name"),
            ("org.iso.18013.5.1", "issue_date"),
            ("org.iso.18013.5.1", "expiry_date"),
            ("org.iso.18013.5.1", "document_number"),
            ("org.iso.18013.5.1", "portrait"),
            ("org.iso.18013.5.1", "driving_privileges"),
        ]),
    )]
    #[case(
        HashSet::from([("org.iso.18013.5.1", "family_name"), ("foo", "bar")]),
        HashSet::from([("org.iso.18013.5.1", "family_name")]),
    )]
    fn test_issuer_signed_into_attribute_subset(
        #[case] attribute_paths: HashSet<(&str, &str)>,
        #[case] expected_attribute_paths: HashSet<(&str, &str)>,
    ) {
        let source_issuer_signed = issuer_signed_example();
        let dest_issuer_signed = source_issuer_signed.clone().into_attribute_subset(&attribute_paths);

        assert_eq!(source_issuer_signed.issuer_auth, dest_issuer_signed.issuer_auth);

        let (source_name_spaces, dest_name_spaces) = [source_issuer_signed, dest_issuer_signed]
            .into_iter()
            .map(|issuer_signed| {
                issuer_signed
                    .name_spaces
                    .map(IssuerNameSpaces::into_inner)
                    .unwrap_or_default()
            })
            .collect_tuple()
            .unwrap();

        // Determine the set of paths present in the destination, while checking
        // that each item is present in the source and matches that item exactly.
        let dest_attribute_paths = dest_name_spaces
            .iter()
            .flat_map(|(name_space, attributes)| {
                attributes.as_ref().iter().map(|TaggedBytes(item)| {
                    let path = (name_space.as_str(), item.element_identifier.as_str());

                    let source_item = source_name_spaces.get(name_space.as_str()).and_then(|attributes| {
                        attributes.as_ref().iter().find_map(|TaggedBytes(source_item)| {
                            (source_item.element_identifier == item.element_identifier).then_some(source_item)
                        })
                    });

                    assert_eq!(source_item, Some(item));

                    path
                })
            })
            .collect::<HashSet<_>>();

        // Check that all paths present in the destination were actually
        // requested and that this matches the expected paths.
        assert!(dest_attribute_paths.is_subset(&attribute_paths));
        assert_eq!(dest_attribute_paths, expected_attribute_paths);

        // Of all the items that were not moved from the source to the destination,
        // check that they were indeed not present in the source.
        let contains_unmoved_item =
            attribute_paths
                .difference(&dest_attribute_paths)
                .any(|(name_space, element_id)| {
                    source_name_spaces
                        .get(*name_space)
                        .map(|attributes| {
                            attributes
                                .as_ref()
                                .iter()
                                .any(|TaggedBytes(item)| item.element_identifier == *element_id)
                        })
                        .unwrap_or(false)
                });

        assert!(!contains_unmoved_item);
    }
}
