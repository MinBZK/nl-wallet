use std::collections::HashSet;
use std::collections::VecDeque;

use indexmap::IndexMap;
use itertools::Itertools;

use nl_wallet_mdoc::utils::auth::Organization;
use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use sd_jwt::metadata::ClaimMetadata;
use sd_jwt::metadata::ClaimPath;
use sd_jwt::metadata::DisplayMetadata;
use sd_jwt::metadata::TypeMetadata;

use crate::attestation::AttestationError;
use crate::attestation::AttributeSelectionMode;
use crate::Attestation;
use crate::AttestationAttribute;
use crate::AttestationIdentity;
use crate::LocalizedString;

impl Attestation {
    pub(super) fn create_from_attributes(
        identity: AttestationIdentity,
        attestation_type: String,
        display_metadata: Vec<DisplayMetadata>,
        issuer: Organization,
        attributes: Vec<AttestationAttribute>,
        nested_attributes: &IndexMap<String, Attribute>,
    ) -> Result<Self, AttestationError> {
        // Check that all attributes have been processed by the metadata claims.
        let processed_key_paths = attributes
            .iter()
            .map(|attr| attr.key.iter().map(String::as_str).collect())
            .collect();
        let original_key_paths = collect_key_paths(nested_attributes);
        let difference = original_key_paths
            .difference(&processed_key_paths)
            .map(|key_path| key_path.iter().map(|key| key.to_string()).collect_vec())
            .collect::<HashSet<_>>();

        if !difference.is_empty() {
            return Err(AttestationError::AttributeNotProcessedByClaim(difference));
        }

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

enum AttributeSelectionResult<'a> {
    Found(Vec<String>, &'a AttributeValue, &'a ClaimMetadata),
    NotFound(&'a ClaimMetadata),
}

impl<'a> AttributeSelectionResult<'a> {
    fn into_attribute(self) -> Option<AttestationAttribute> {
        match self {
            AttributeSelectionResult::Found(key, attribute_value, claim) => {
                Some(Self::to_attribute(key, attribute_value, claim))
            }
            AttributeSelectionResult::NotFound(_) => None,
        }
    }

    fn try_into_attribute(self) -> Result<AttestationAttribute, AttestationError> {
        match self {
            AttributeSelectionResult::Found(key, attribute_value, claim) => {
                Ok(Self::to_attribute(key, attribute_value, claim))
            }
            AttributeSelectionResult::NotFound(claim) => {
                Err(AttestationError::AttributeNotFoundForClaim(claim.clone()))
            }
        }
    }

    fn to_attribute(
        key: Vec<String>,
        attribute_value: &'a AttributeValue,
        claim: &'a ClaimMetadata,
    ) -> AttestationAttribute {
        AttestationAttribute {
            key,
            value: attribute_value.clone(),
            labels: claim.display.clone().into_iter().map(LocalizedString::from).collect(),
        }
    }
}

impl AttestationAttribute {
    pub(super) fn from_attributes(
        attributes: &IndexMap<String, Attribute>,
        metadata: &TypeMetadata,
        selection_mode: &AttributeSelectionMode,
    ) -> Result<Vec<Self>, AttestationError> {
        let selection = metadata
            .claims
            .iter()
            .map(|claim| {
                let key = claim.path.iter().map(|cp| cp.to_string()).collect();
                let mut paths = claim.path.iter().collect::<VecDeque<_>>();
                let attribute = Self::select_attribute(&mut paths, attributes);
                match attribute {
                    Some(Attribute::Single(value)) => AttributeSelectionResult::Found(key, value, claim),
                    _ => AttributeSelectionResult::NotFound(claim),
                }
            })
            .collect::<Vec<_>>();

        let attributes = match selection_mode {
            AttributeSelectionMode::Issuance => selection
                .into_iter()
                .map(|selection| selection.try_into_attribute())
                .collect::<Result<_, _>>()?,
            AttributeSelectionMode::Disclosure => selection
                .into_iter()
                .filter_map(|selection| selection.into_attribute())
                .collect(),
        };

        Ok(attributes)
    }

    fn select_attribute<'a>(
        paths: &mut VecDeque<&ClaimPath>,
        attributes: &'a IndexMap<String, Attribute>,
    ) -> Option<&'a Attribute> {
        if let Some(path) = paths.pop_front() {
            let attribute = match path {
                ClaimPath::SelectByKey(key) => attributes.get(key),
                _ => None,
            }?;

            match attribute {
                Attribute::Single(_) if paths.is_empty() => Some(attribute),
                Attribute::Nested(nested_attrs) if !paths.is_empty() => Self::select_attribute(paths, nested_attrs),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Collect all of full key paths present in `attributes` by unrolling any nested attribute paths.
fn collect_key_paths(attributes: &IndexMap<String, Attribute>) -> HashSet<Vec<&str>> {
    collect_key_paths_recursive(attributes, &[])
}

fn collect_key_paths_recursive<'a>(
    attributes: &'a IndexMap<String, Attribute>,
    parent_path: &[&'a str],
) -> HashSet<Vec<&'a str>> {
    // The minimum amount of paths is when all of the attributes are single,
    // so this fold reserves at least that amount as the accumulator.
    attributes.iter().fold(
        HashSet::with_capacity(attributes.len()),
        |mut key_paths, (key, attribute)| {
            // Construct how we got here by appending the iteration's key to the path of our caller.
            let mut current_path = Vec::with_capacity(parent_path.len() + 1);
            current_path.extend_from_slice(parent_path);
            current_path.push(key);

            match attribute {
                // This is a leaf node, so simply add the current path to the accumulator.
                Attribute::Single(_) => {
                    key_paths.insert(current_path);
                }
                // If this is not a leaf node, prove the current path to a recursive
                // call of this function and extend the accumulator with the result.
                Attribute::Nested(nested) => key_paths.extend(collect_key_paths_recursive(nested, &current_path)),
            }

            key_paths
        },
    )
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
    use crate::AttestationAttribute;

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
    fn test_select_single_attribute_happy() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(
            &mut vec![&ClaimPath::SelectByKey(String::from("single"))].into(),
            attributes,
        );
        assert_matches!(
            result,
            Some(Attribute::Single(AttributeValue::Text(value))) if value.as_str() == "single",
            "selecting single attribute by key should find attribute"
        );
    }

    #[test]
    fn test_select_nested_attribute_for_single() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(
            &mut vec![
                &ClaimPath::SelectByKey(String::from("single")),
                &ClaimPath::SelectByKey(String::from("not_found")),
            ]
            .into(),
            attributes,
        );
        assert_matches!(
            result,
            None,
            "selecting nested attribute by key should find nothing for single attribute"
        );
    }

    #[test]
    fn test_select_nested_attribute_happy() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(
            &mut vec![
                &ClaimPath::SelectByKey(String::from("nested_1a")),
                &ClaimPath::SelectByKey(String::from("nested_1b")),
                &ClaimPath::SelectByKey(String::from("nested_1c")),
            ]
            .into(),
            attributes,
        );
        assert_matches!(
            result,
            Some(Attribute::Single(AttributeValue::Text(value))) if value.as_str() == "nested_value",
            "selecting nested attribute by keys should find attribute"
        );
    }

    #[test]
    fn test_select_nested_attribute_unknown_key() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(
            &mut vec![
                &ClaimPath::SelectByKey(String::from("nested_1a")),
                &ClaimPath::SelectByKey(String::from("nested_1b")),
                &ClaimPath::SelectByKey(String::from("not_found")),
            ]
            .into(),
            attributes,
        );
        assert_matches!(
            result,
            None,
            "selecting nested attribute by key should find nothing for unknown key"
        );
    }

    #[test]
    fn test_select_nested_attribute_too_deep() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(
            &mut vec![
                &ClaimPath::SelectByKey(String::from("nested_1a")),
                &ClaimPath::SelectByKey(String::from("nested_1b")),
                &ClaimPath::SelectByKey(String::from("nested_1c")),
                &ClaimPath::SelectByKey(String::from("nested_1d")),
            ]
            .into(),
            attributes,
        );
        assert_matches!(
            result,
            None,
            "selecting by more keys than attributes are nested should find nothing"
        );
    }

    #[test]
    fn test_select_attribute_with_empty_paths() {
        let attributes = &*ATTRIBUTES;

        let result = AttestationAttribute::select_attribute(&mut vec![].into(), attributes);
        assert_matches!(result, None, "selecting nothing should find nothing");
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
                vec!["birthdate"],
                vec!["place_of_birth", "locality"],
                vec!["place_of_birth", "country", "name"],
                vec!["place_of_birth", "country", "area_code"],
            ])
        );
    }
}
