use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;

use indexmap::IndexMap;
use itertools::Either;
use itertools::Itertools;

use attestation_types::claim_path::ClaimPath;
use utils::vec_at_least::VecNonEmpty;

use crate::iso::disclosure::IssuerSigned;
use crate::iso::mdocs::Attributes;
use crate::utils::serialization::TaggedBytes;

use super::MissingAttributesError;

/// Helper function for converting a claim path to a tuple of name space and element identifier.
/// This will return `None` if:
/// * Any of the path elements is not a key path.
/// * The claim path does not consist of two elements.
pub fn claim_path_to_mdoc_path(path: &VecNonEmpty<ClaimPath>) -> Option<(&str, &str)> {
    path.iter()
        .map(ClaimPath::try_key_path)
        .collect::<Option<Vec<_>>>()
        .and_then(|path| path.into_iter().collect_tuple())
}

impl IssuerSigned {
    /// Test if the [`IssuerSigned`] contains all of the attributes addressed by the claim paths. The resulting error,
    /// if any, will contain a list of missing attributes. Note that any claim path that is not a full key path or has
    /// anything else than two elements will lead to a mismatch.
    pub fn matches_requested_attributes<'a, 'b>(
        &'a self,
        claim_paths: impl IntoIterator<Item = &'b VecNonEmpty<ClaimPath>>,
    ) -> Result<(), MissingAttributesError> {
        let missing_attributes = claim_paths
            .into_iter()
            .flat_map(|path| {
                if let Some((name_space, element_id)) = claim_path_to_mdoc_path(path) {
                    let attribute_present = self
                        .name_spaces
                        .as_ref()
                        .and_then(|name_spaces| {
                            name_spaces.as_ref().get(name_space).map(|attributes| {
                                attributes
                                    .as_ref()
                                    .iter()
                                    .any(|TaggedBytes(signed_item)| signed_item.element_identifier == *element_id)
                            })
                        })
                        .unwrap_or(false);

                    if attribute_present {
                        return None;
                    }
                }

                Some(path.clone())
            })
            .collect::<HashSet<_>>();

        if !missing_attributes.is_empty() {
            return Err(MissingAttributesError(missing_attributes));
        }

        Ok(())
    }

    /// Prune the [`IssuerSigned`] of any attributes that are not covered by the claim paths. This may result in its
    /// `name_spaces` field to be set to `None`. Note that claim paths that are not full key paths or do not consist of
    /// two elements are unsupported and will result in an error.
    pub fn into_attribute_subset<'a>(
        self,
        claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>,
    ) -> Result<Self, MissingAttributesError> {
        // Define a newtype to wrap the the keys for the mdoc_paths `HashMap` down below. Unfortunately, the requirement
        // that mutable references be invariant over their type parameters means that the lifetime of any reference used
        // when calling `HashMap::remove()` needs to live as long as the lifetime of the key typeembedded in the type.
        // In our case that means that any key passed to `remove()` requires a lifetime that is at least as long as the
        // `'a` lifetime of the claim_path iterator argument of this method. Implementing `Borrow` on the newtype which
        // allows for the key to be borrowed with a shorter lifetime skirts around that requirement.
        #[derive(PartialEq, Eq, Hash)]
        struct MdocPathKey<'a>((&'a str, &'a str));

        impl<'a: 'b, 'b> Borrow<(&'b str, &'b str)> for MdocPathKey<'a> {
            fn borrow(&self) -> &(&'b str, &'b str) {
                let Self(inner) = self;

                inner
            }
        }

        let Some(name_spaces) = self.name_spaces else {
            return Err(MissingAttributesError(claim_paths.into_iter().cloned().collect()));
        };

        let (mut mdoc_paths, mut missing_paths): (HashMap<_, _>, HashSet<_>) =
            claim_paths.into_iter().partition_map(|claim_path| {
                if let Some(mdoc_path) = claim_path_to_mdoc_path(claim_path) {
                    Either::Left((MdocPathKey(mdoc_path), claim_path))
                } else {
                    Either::Right(claim_path.clone())
                }
            });

        // Remove all of the attributes that are not listed in mdoc_paths,
        // which may cause name_spaces to be returned as None.
        let name_spaces = name_spaces
            .into_iter()
            .flat_map(|(name_space, attributes)| {
                let attributes = attributes
                    .into_inner()
                    .into_iter()
                    .filter(|TaggedBytes(signed_item)| {
                        mdoc_paths
                            .remove(&(name_space.as_str(), signed_item.element_identifier.as_str()))
                            .is_some()
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

        missing_paths.extend(mdoc_paths.into_values().cloned());

        if !missing_paths.is_empty() {
            return Err(MissingAttributesError(missing_paths));
        }

        Ok(Self { name_spaces, ..self })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use rstest::rstest;

    use attestation_types::claim_path::ClaimPath;
    use utils::vec_at_least::VecNonEmpty;

    use crate::IssuerNameSpaces;
    use crate::examples::Example;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::disclosure::IssuerSigned;
    use crate::utils::serialization::TaggedBytes;

    use super::super::MissingAttributesError;
    use super::claim_path_to_mdoc_path;

    fn issuer_signed_example() -> IssuerSigned {
        DeviceResponse::example()
            .documents
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .issuer_signed
    }

    fn claim_path(elements: &[&str]) -> VecNonEmpty<ClaimPath> {
        elements
            .iter()
            .map(|key| ClaimPath::SelectByKey(key.to_string()))
            .collect_vec()
            .try_into()
            .unwrap()
    }

    #[rstest]
    #[case(vec![], None)]
    #[case(vec![claim_path(&["org.iso.18013.5.1", "family_name"])], None)]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "family_name"]),
            claim_path(&["org.iso.18013.5.1", "issue_date"]),
            claim_path(&["org.iso.18013.5.1", "expiry_date"]),
            claim_path(&["org.iso.18013.5.1", "document_number"]),
            claim_path(&["org.iso.18013.5.1", "portrait"]),
            claim_path(&["org.iso.18013.5.1", "driving_privileges"]),
        ],
        None,
    )]
    #[case(
        vec![claim_path(&["org.iso.18013.5.1", "is_rich"])],
        Some(vec![claim_path(&["org.iso.18013.5.1", "is_rich"])].into_iter().collect()),
    )]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "family_name"]),
            claim_path(&["org.iso.18013.5.1", "is_rich"]),
        ],
        Some(vec![claim_path(&["org.iso.18013.5.1", "is_rich"])].into_iter().collect()),
    )]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "family_name"]),
            claim_path(&["vroom", "driving_privileges"]),
        ],
        Some(vec![claim_path(&["vroom", "driving_privileges"])].into_iter().collect()),
    )]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "portrait"]),
            claim_path(&["foobar"]),
            claim_path(&["org.iso.18013.5.1", "driving_privileges"]),
            claim_path(&["foobar", "bleh", "blah"]),
        ],
        Some(vec![
            claim_path(&["foobar", "bleh", "blah"]),
            claim_path(&["foobar"]),
        ].into_iter().collect()),
    )]
    #[case(
        vec![vec![ClaimPath::SelectAll].try_into().unwrap()],
        Some(HashSet::from([vec![ClaimPath::SelectAll].try_into().unwrap()]))
    )]
    fn test_issuer_signed_matches_requested_attributes(
        #[case] claim_paths: Vec<VecNonEmpty<ClaimPath>>,
        #[case] expected_missing_attributes: Option<HashSet<VecNonEmpty<ClaimPath>>>,
    ) {
        let matches = issuer_signed_example().matches_requested_attributes(&claim_paths);

        match expected_missing_attributes {
            None => matches.expect("should match requested attributes"),
            Some(attributes) => {
                assert_matches!(
                    matches.expect_err("should match not requested attributes"),
                    MissingAttributesError(missing_attributes)
                        if missing_attributes == attributes
                );
            }
        }
    }

    #[derive(Debug)]
    enum IntoAttributeSubsetResult<'a> {
        Success(HashSet<(&'a str, &'a str)>),
        Failure(HashSet<VecNonEmpty<ClaimPath>>),
    }

    #[rstest]
    #[case(vec![], IntoAttributeSubsetResult::Success(HashSet::new()))]
    #[case(
        vec![claim_path(&["foo", "bar"]), claim_path(&["bleh", "blah"])],
        IntoAttributeSubsetResult::Failure(HashSet::from([claim_path(&["foo", "bar"]), claim_path(&["bleh", "blah"])])),
    )]
    #[case(
        vec![claim_path(&["foo"]), claim_path(&["bar"]), claim_path(&["bleh"]), claim_path(&["blah"])],
        IntoAttributeSubsetResult::Failure(HashSet::from([
            claim_path(&["foo"]),
            claim_path(&["bar"]),
            claim_path(&["bleh"]),
            claim_path(&["blah"])
        ])),
    )]
    #[case(
        vec![claim_path(&["foo", "bar", "bleh"])],
        IntoAttributeSubsetResult::Failure(HashSet::from([claim_path(&["foo", "bar", "bleh"])])),
    )]
    #[case(
        vec![claim_path(&["foo", "bar", "bleh", "blah"])],
        IntoAttributeSubsetResult::Failure(HashSet::from([claim_path(&["foo", "bar", "bleh", "blah"])])),
    )]
    #[case(
        vec![claim_path(&["org.iso.18013.5.1", "family_name"])],
        IntoAttributeSubsetResult::Success(HashSet::from([("org.iso.18013.5.1", "family_name")])),
    )]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "family_name"]),
            claim_path(&["org.iso.18013.5.1", "issue_date"]),
            claim_path(&["org.iso.18013.5.1", "expiry_date"]),
            claim_path(&["org.iso.18013.5.1", "document_number"]),
            claim_path(&["org.iso.18013.5.1", "portrait"]),
            claim_path(&["org.iso.18013.5.1", "driving_privileges"]),
        ],
        IntoAttributeSubsetResult::Success(HashSet::from([
            ("org.iso.18013.5.1", "family_name"),
            ("org.iso.18013.5.1", "issue_date"),
            ("org.iso.18013.5.1", "expiry_date"),
            ("org.iso.18013.5.1", "document_number"),
            ("org.iso.18013.5.1", "portrait"),
            ("org.iso.18013.5.1", "driving_privileges"),
        ])),
    )]
    #[case(
        vec![claim_path(&["org.iso.18013.5.1", "family_name"]), claim_path(&["foo", "bar"])],
        IntoAttributeSubsetResult::Failure(HashSet::from([claim_path(&["foo", "bar"])])),
    )]
    #[case(
        vec![
            claim_path(&["org.iso.18013.5.1", "portrait"]),
            claim_path(&["foo"]),
            claim_path(&["foo", "bar", "bleh"]),
            claim_path(&["org.iso.18013.5.1", "driving_privileges"]),
        ],
        IntoAttributeSubsetResult::Failure(HashSet::from([
            claim_path(&["foo"]),
            claim_path(&["foo", "bar", "bleh"]),
        ])),
    )]
    fn test_issuer_signed_into_attribute_subset(
        #[case] claim_paths: Vec<VecNonEmpty<ClaimPath>>,
        #[case] expected_result: IntoAttributeSubsetResult,
    ) {
        let source_issuer_signed = issuer_signed_example();
        let dest_issuer_signed_result = source_issuer_signed.clone().into_attribute_subset(&claim_paths);

        let expected_attribute_paths = match expected_result {
            IntoAttributeSubsetResult::Success(expected_attribute_paths) => expected_attribute_paths,
            IntoAttributeSubsetResult::Failure(expected_missing_paths) => {
                let MissingAttributesError(missing_paths) =
                    dest_issuer_signed_result.expect_err("getting IssuerSigned attribute subset should not succeed");

                assert_eq!(missing_paths, expected_missing_paths);

                return;
            }
        };

        let dest_issuer_signed =
            dest_issuer_signed_result.expect("getting IssuerSigned attribute subset should succeed");

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

        let mdoc_paths = claim_paths
            .iter()
            .flat_map(claim_path_to_mdoc_path)
            .collect::<HashSet<_>>();

        // Check that all paths present in the destination were actually
        // requested and that this matches the expected paths.
        assert!(dest_attribute_paths.is_subset(&mdoc_paths));
        assert_eq!(dest_attribute_paths, expected_attribute_paths);

        // Of all the items that were not moved from the source to the destination,
        // check that they were indeed not present in the source.
        let contains_unmoved_item = mdoc_paths
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
