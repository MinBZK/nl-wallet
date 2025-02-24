use std::collections::HashSet;

use indexmap::IndexMap;
use itertools::Itertools;

use nl_wallet_mdoc::utils::auth::Organization;
use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use sd_jwt::metadata::ClaimPath;
use sd_jwt::metadata::TypeMetadata;
use wallet_common::vec_at_least::VecNonEmpty;

use super::Attestation;
use super::AttestationAttribute;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttributeSelectionMode;
use super::LocalizedString;

impl Attestation {
    // Construct a new `Attestation` from a combination of metadata and nested attributes.
    // This method has different behaviour depending on the `selection_mode` parameter.
    pub(super) fn create_from_attributes(
        identity: AttestationIdentity,
        attestation_type: String,
        metadata: TypeMetadata,
        issuer: Organization,
        mut nested_attributes: IndexMap<String, Attribute>,
        selection_mode: AttributeSelectionMode,
    ) -> Result<Self, AttestationError> {
        // For every claim in the metadata, traverse the nested attributes to find it,
        // then convert it to a `AttestationAttribute` value.
        let attributes_iter = metadata.claims.into_iter().map(|claim| {
            match take_attribute_value_at_key_path(&mut nested_attributes, claim.path.clone()) {
                Some((path, value)) => Ok(AttestationAttribute {
                    key: path,
                    value,
                    labels: claim.display.into_iter().map(LocalizedString::from).collect(),
                }),
                None => Err(AttestationError::AttributeNotFoundForClaim(claim.path)),
            }
        });

        let attributes = match selection_mode {
            // During issuance, an attribute that is in the metadata but not in the nested
            // tree of attributes received from the issuer will result in an error.
            AttributeSelectionMode::Issuance => attributes_iter.try_collect()?,
            // Because of selective disclosure, an attribute that is in the metadata but
            // not in the proposal for which attributes to disclose can simply be ignored.
            AttributeSelectionMode::Disclosure => attributes_iter.flatten().collect(),
        };

        // The nested attributes should now be fully drained of any attribute values.
        // If this is not the case, we were provided an attribute
        // that is not covered by the metadata, which is an error.
        let remaining_key_paths = collect_key_paths(&nested_attributes);

        if !remaining_key_paths.is_empty() {
            return Err(AttestationError::AttributeNotProcessedByClaim(remaining_key_paths));
        }

        // Finally, construct the `Attestation` type.
        let display_metadata = metadata
            .display
            .into_iter()
            .map(|metadata| (metadata.lang.clone(), metadata))
            .collect();
        let attestation = Attestation {
            identity,
            display_metadata,
            attestation_type,
            issuer,
            attributes,
        };

        Ok(attestation)
    }
}

/// Look for an [`AttributeValue`] within a nested tree of attributes by traversing a particular key path within that
/// tree. If this value is found, remove it from its `IndexMap` and return both the exact path and the value itself.
fn take_attribute_value_at_key_path(
    attributes: &mut IndexMap<String, Attribute>,
    path: VecNonEmpty<ClaimPath>,
) -> Option<(Vec<String>, AttributeValue)> {
    // First, confirm that the path is made up of key entries by converting to a `Vec<String>`.
    // This will return `None` if any of the elements of the path is not an index.
    let key_path = path
        .into_iter()
        .map(|path| match path {
            ClaimPath::SelectByKey(key) => Some(key),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;

    // Iterate over the path to first find the correct `IndexMap` and then look for the value in it.
    key_path
        .iter()
        // This is guaranteed not to underflow because the key path has at least one entry.
        .take(key_path.len() - 1)
        // For all entries in the path but the last, start traversing the path
        // and expect to find another nested IndexMap every step along the way.
        // Note that for a path length of 1, this will result in the input IndexMap.
        .try_fold(attributes, |attributes, key| match attributes.get_mut(key) {
            Some(Attribute::Nested(nested_attributes)) => Some(nested_attributes),
            _ => None,
        })
        // For the last entry in the path, if the IndexMap found in the last step
        // contains a value for that key, remove it and return it.
        .and_then(|attributes| match attributes.swap_remove(key_path.last().unwrap()) {
            Some(Attribute::Single(value)) => Some(value),
            _ => None,
        })
        // Combine the resulting value with the full path created earlier.
        .map(|value| (key_path, value))
}

/// Collect all full key paths present in `attributes` by unrolling any nested attribute paths.
fn collect_key_paths(attributes: &IndexMap<String, Attribute>) -> HashSet<Vec<String>> {
    collect_key_paths_recursive(attributes, &[])
}

fn collect_key_paths_recursive(attributes: &IndexMap<String, Attribute>, parent_path: &[&str]) -> HashSet<Vec<String>> {
    // Prepare a `Vec`` with the current path by prepending it with the parent path.
    let mut current_path = Vec::with_capacity(parent_path.len() + 1);
    current_path.extend_from_slice(parent_path);

    // Collect all of the leaf nodes encountered into a `HashSet`` accumulator, while recursing all nested values.
    attributes
        .iter()
        .fold(HashSet::new(), |mut key_paths, (key, attribute)| {
            // Construct how we got here by appending the iteration's key to the path of our caller.
            current_path.push(key);

            match attribute {
                // This is a leaf node, so add allocated strings of the current path to the accumulator.
                Attribute::Single(_) => {
                    key_paths.insert(current_path.iter().map(|key| key.to_string()).collect());
                }
                // If this is not a leaf node, prove the current path to a recursive
                // call of this function and extend the accumulator with the result.
                Attribute::Nested(nested) => key_paths.extend(collect_key_paths_recursive(nested, &current_path)),
            }

            current_path.pop();

            key_paths
        })
}

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;
    use serde_json::json;

    use openid4vc::attributes::Attribute;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::ClaimMetadata;
    use sd_jwt::metadata::ClaimPath;
    use sd_jwt::metadata::ClaimSelectiveDisclosureMetadata;

    use crate::attestation::attribute::collect_key_paths;
    use crate::attestation::attribute::take_attribute_value_at_key_path;

    pub static ATTRIBUTES: LazyLock<IndexMap<String, Attribute>> = LazyLock::new(|| {
        IndexMap::from([
            (
                String::from("single"),
                Attribute::Single(AttributeValue::Text(String::from("single"))),
            ),
            (
                String::from("nested_1a"),
                Attribute::Nested(IndexMap::from([(
                    String::from("nested_1b"),
                    Attribute::Nested(IndexMap::from([(
                        String::from("nested_1c"),
                        Attribute::Single(AttributeValue::Text(String::from("nested_value"))),
                    )])),
                )])),
            ),
        ])
    });

    pub fn claim_metadata(keys: &[&str]) -> ClaimMetadata {
        ClaimMetadata {
            path: keys
                .iter()
                .map(|key| ClaimPath::SelectByKey(String::from(*key)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            display: vec![],
            sd: ClaimSelectiveDisclosureMetadata::Always,
            svg_id: None,
        }
    }

    #[test]
    fn test_take_attribute_value_at_path_single() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![ClaimPath::SelectByKey(String::from("single"))].try_into().unwrap(),
        );

        assert_matches!(
            result,
            Some((path, AttributeValue::Text(value))) if path == vec!["single"] && value == "single",
            "selecting single attribute by key should find attribute"
        );
    }

    #[test]
    fn test_take_attribute_value_at_path_single_not_found() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![
                ClaimPath::SelectByKey(String::from("single")),
                ClaimPath::SelectByKey(String::from("not_found")),
            ]
            .try_into()
            .unwrap(),
        );

        assert_matches!(
            result,
            None,
            "selecting nested attribute by key should find nothing for single attribute"
        );
    }

    #[test]
    fn test_take_attribute_value_at_path_single_nested() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![
                ClaimPath::SelectByKey(String::from("nested_1a")),
                ClaimPath::SelectByKey(String::from("nested_1b")),
                ClaimPath::SelectByKey(String::from("nested_1c")),
            ]
            .try_into()
            .unwrap(),
        );

        assert_matches!(
            result,
            Some((path, AttributeValue::Text(value)))
                if path == vec!["nested_1a", "nested_1b", "nested_1c"] && value == "nested_value",
            "selecting nested attribute by keys should find attribute"
        );
    }

    #[test]
    fn test_take_attribute_value_at_path_single_nested_not_found() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![
                ClaimPath::SelectByKey(String::from("nested_1a")),
                ClaimPath::SelectByKey(String::from("nested_1b")),
                ClaimPath::SelectByKey(String::from("not_found")),
            ]
            .try_into()
            .unwrap(),
        );

        assert_matches!(
            result,
            None,
            "selecting nested attribute by key should find nothing for unknown key"
        );
    }

    #[test]
    fn test_take_attribute_value_at_path_too_deep() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![
                ClaimPath::SelectByKey(String::from("nested_1a")),
                ClaimPath::SelectByKey(String::from("nested_1b")),
                ClaimPath::SelectByKey(String::from("nested_1c")),
                ClaimPath::SelectByKey(String::from("nested_1d")),
            ]
            .try_into()
            .unwrap(),
        );

        assert_matches!(
            result,
            None,
            "selecting by more keys than attributes are nested should find nothing"
        );
    }

    #[test]
    fn test_collect_key_paths() {
        let json = json!({
            "birthdate": "1963-08-12",
            "place_of_birth": {
                "locality": "The Hague",
                "country": {
                    "name": "The Netherlands",
                    "area_code": 33
                }
            }
        });
        let attributes: IndexMap<String, Attribute> = serde_json::from_value(json).unwrap();
        assert_eq!(
            collect_key_paths(&attributes),
            HashSet::from([
                vec!["birthdate".to_string()],
                vec!["place_of_birth".to_string(), "locality".to_string()],
                vec!["place_of_birth".to_string(), "country".to_string(), "name".to_string()],
                vec![
                    "place_of_birth".to_string(),
                    "country".to_string(),
                    "area_code".to_string()
                ],
            ])
        );
    }
}
