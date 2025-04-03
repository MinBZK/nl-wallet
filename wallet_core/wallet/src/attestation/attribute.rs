use std::collections::HashMap;
use std::collections::HashSet;

use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;

use mdoc::utils::auth::Organization;
use openid4vc::attributes::Attribute;
use openid4vc::attributes::AttributeValue;
use sd_jwt_vc_metadata::ClaimPath;
use sd_jwt_vc_metadata::JsonSchemaProperty;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use wallet_common::vec_at_least::VecNonEmpty;

use super::Attestation;
use super::AttestationAttribute;
use super::AttestationAttributeValue;
use super::AttestationError;
use super::AttestationIdentity;

impl Attestation {
    // Construct a new `Attestation` from a combination of metadata and nested attributes.
    // This method has different behaviour depending on the `selection_mode` parameter.
    pub(super) fn create_from_attributes(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer: Organization,
        mut nested_attributes: IndexMap<String, Attribute>,
    ) -> Result<Self, AttestationError> {
        let (attestation_type, display_metadata, claims, schema) = metadata.into_leaf_components();

        // Extract the JSON Schema properties from the metadata, which has the same structure as the attributes
        // (otherwise, they wouldn't validate later on when converted to a `CredentialPayload`). The JSON Schema is used
        // to provide extra metadata for converting attributes values.
        let schema_properties = schema.into_properties().properties;

        // For every claim in the metadata, traverse the nested attributes to find it,
        // then convert it to a `AttestationAttribute` value.
        let attributes = claims
            .into_iter()
            .filter_map(|claim| {
                take_attribute_value_at_key_path(&mut nested_attributes, claim.path, &schema_properties)
                    .map(|attr| {
                        attr.map(|(path, value)| AttestationAttribute {
                            key: path,
                            metadata: claim.display,
                            value,
                        })
                    })
                    .transpose()
            })
            .try_collect()?;

        // The nested attributes should now be fully drained of any attribute values.
        // If this is not the case, we were provided an attribute
        // that is not covered by the metadata, which is an error.
        let remaining_key_paths = collect_key_paths(&nested_attributes);

        if !remaining_key_paths.is_empty() {
            return Err(AttestationError::AttributeNotProcessedByClaim(remaining_key_paths));
        }

        // Finally, construct the `Attestation` type.
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
    json_schema_properties: &HashMap<String, JsonSchemaProperty>,
) -> Result<Option<(Vec<String>, AttestationAttributeValue)>, AttestationError> {
    // First, confirm that the path is made up of key entries by converting to a `Vec<String>`.
    // This will return `None` if any of the elements of the path is not an index.
    path.into_iter()
        .map(ClaimPath::try_into_key_path)
        .collect::<Option<Vec<_>>>()
        .and_then(|key_path| {
            // Iterate over the path to first find the correct `IndexMap` and then look for the value in it.
            key_path
                .iter()
                // This is guaranteed not to underflow because the key path has at least one entry.
                .take(key_path.len() - 1)
                // For all entries in the path but the last, start traversing the path
                // and expect to find another nested IndexMap every step along the way.
                // Since the JSON schema properties have the same structure, it is matched
                // in a tuple along with the attributes IndexMap.
                // Note that for a path length of 1, this will result in the input IndexMap.
                .try_fold(
                    (attributes, json_schema_properties),
                    |(attributes, json_schema_properties), key| match (
                        attributes.get_mut(key),
                        json_schema_properties
                            .get(key)
                            .and_then(|prop| prop.properties.as_ref()),
                    ) {
                        (Some(Attribute::Nested(nested_attributes)), Some(props)) => Some((nested_attributes, props)),
                        _ => None,
                    },
                )
                .and_then(|(attributes, json_props)| {
                    // For the last entry in the path, if the IndexMap found in the last step
                    // contains a value for that key, remove it and return it.
                    let key = key_path.last().unwrap();
                    match (attributes.swap_remove(key), json_props.get(key)) {
                        (Some(Attribute::Single(value)), Some(json_property)) => Some(
                            AttestationAttributeValue::try_from_attribute_value(value, json_property)
                                // Combine the resulting attribute value with the full path created earlier.
                                .map(|attribute_value| (key_path, attribute_value)),
                        ),
                        _ => None,
                    }
                })
        })
        .transpose()
}

/// Collect all full key paths present in `attributes` by unrolling any nested attribute paths.
fn collect_key_paths(attributes: &IndexMap<String, Attribute>) -> HashSet<Vec<String>> {
    collect_key_paths_recursive(attributes, &[])
}

fn collect_key_paths_recursive(attributes: &IndexMap<String, Attribute>, parent_path: &[&str]) -> HashSet<Vec<String>> {
    // Prepare a `Vec`` with the current path by prepending it with the parent path.
    let mut current_path = Vec::with_capacity(parent_path.len() + 1);
    current_path.extend_from_slice(parent_path);

    // Collect all of the leaf nodes encountered into a `HashSet` accumulator, while recursing all nested values.
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

impl AttestationAttributeValue {
    fn try_from_attribute_value(
        value: AttributeValue,
        schema_type: &JsonSchemaProperty,
    ) -> Result<Self, AttestationError> {
        match (&schema_type.r#type, value) {
            (JsonSchemaPropertyType::Boolean, AttributeValue::Bool(bool)) => {
                Ok(AttestationAttributeValue::Basic(AttributeValue::Bool(bool)))
            }
            (JsonSchemaPropertyType::Integer, AttributeValue::Integer(integer)) => {
                Ok(AttestationAttributeValue::Basic(AttributeValue::Integer(integer)))
            }
            (JsonSchemaPropertyType::String, AttributeValue::Text(text)) => {
                if let Some(JsonSchemaPropertyFormat::Date) = schema_type.format {
                    let date = NaiveDate::parse_from_str(&text, "%Y-%m-%d")?;
                    Ok(AttestationAttributeValue::Date(date))
                } else {
                    Ok(AttestationAttributeValue::Basic(AttributeValue::Text(text)))
                }
            }
            (_, value) => Err(AttestationError::AttributeConversion(value, schema_type.clone())),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;

    use openid4vc::attributes::Attribute;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt_vc_metadata::ClaimMetadata;
    use sd_jwt_vc_metadata::ClaimPath;
    use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
    use sd_jwt_vc_metadata::JsonSchemaProperty;
    use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;

    use crate::attestation::attribute::collect_key_paths;
    use crate::attestation::attribute::take_attribute_value_at_key_path;
    use crate::attestation::AttestationAttributeValue;

    static ATTRIBUTES: LazyLock<IndexMap<String, Attribute>> = LazyLock::new(|| {
        IndexMap::from([
            (
                String::from("single"),
                Attribute::Single(AttributeValue::Text(String::from("single"))),
            ),
            (
                String::from("date"),
                Attribute::Single(AttributeValue::Text(String::from("2024-12-26"))),
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
            &HashMap::from([(
                String::from("single"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: None,
                    properties: None,
                },
            )]),
        )
        .unwrap()
        .unwrap();

        assert_matches!(
            result,
            (path, AttestationAttributeValue::Basic(AttributeValue::Text(value)))
                if path == vec!["single"] && value == "single",
            "selecting single attribute by key should find attribute"
        );
    }

    #[test]
    fn test_take_date_attribute_value_at_path() {
        let result = take_attribute_value_at_key_path(
            &mut ATTRIBUTES.clone(),
            vec![ClaimPath::SelectByKey(String::from("date"))].try_into().unwrap(),
            &HashMap::from([(
                String::from("date"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: Some(JsonSchemaPropertyFormat::Date),
                    properties: None,
                },
            )]),
        )
        .unwrap()
        .unwrap();

        assert_matches!(
            result,
            (path, AttestationAttributeValue::Date(value))
                if path == vec!["date"] && value == NaiveDate::from_ymd_opt(2024, 12, 26).unwrap(),
            "selecting date attribute by key should find attribute"
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
            &HashMap::from([(
                String::from("single"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: None,
                    properties: None,
                },
            )]),
        );

        assert_matches!(
            result,
            Ok(None),
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
            &HashMap::from([(
                String::from("nested_1a"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: None,
                    properties: Some(HashMap::from([(
                        String::from("nested_1b"),
                        JsonSchemaProperty {
                            r#type: JsonSchemaPropertyType::String,
                            format: None,
                            properties: Some(HashMap::from([(
                                String::from("nested_1c"),
                                JsonSchemaProperty {
                                    r#type: JsonSchemaPropertyType::String,
                                    format: None,
                                    properties: None,
                                },
                            )])),
                        },
                    )])),
                },
            )]),
        )
        .unwrap()
        .unwrap();

        assert_matches!(
            result,
            (path, AttestationAttributeValue::Basic(AttributeValue::Text(value)))
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
            &HashMap::from([(
                String::from("nested_1a"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: None,
                    properties: Some(HashMap::from([(
                        String::from("nested_1b"),
                        JsonSchemaProperty {
                            r#type: JsonSchemaPropertyType::String,
                            format: None,
                            properties: None,
                        },
                    )])),
                },
            )]),
        );

        assert_matches!(
            result,
            Ok(None),
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
            &HashMap::from([(
                String::from("nested_1a"),
                JsonSchemaProperty {
                    r#type: JsonSchemaPropertyType::String,
                    format: None,
                    properties: Some(HashMap::from([(
                        String::from("nested_1b"),
                        JsonSchemaProperty {
                            r#type: JsonSchemaPropertyType::String,
                            format: None,
                            properties: Some(HashMap::from([(
                                String::from("nested_1c"),
                                JsonSchemaProperty {
                                    r#type: JsonSchemaPropertyType::String,
                                    format: None,
                                    properties: Some(HashMap::from([(
                                        String::from("nested_1d"),
                                        JsonSchemaProperty {
                                            r#type: JsonSchemaPropertyType::String,
                                            format: None,
                                            properties: None,
                                        },
                                    )])),
                                },
                            )])),
                        },
                    )])),
                },
            )]),
        );

        assert_matches!(
            result,
            Ok(None),
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

    #[rstest]
    #[case(AttributeValue::Text(String::from("normal text")), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: None,
            properties: None,
        })]
    #[case(AttributeValue::Bool(false), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Boolean,
            format: None,
            properties: None,
        })]
    #[case(AttributeValue::Integer(123), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Integer,
            format: None,
            properties: None,
        })]
    #[case(AttributeValue::Text(String::from("2002-12-28")), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: Some(JsonSchemaPropertyFormat::Date),
            properties: None,
        })]
    #[should_panic]
    #[case(AttributeValue::Text(String::from("2002-12-28")), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Number,
            format: None,
            properties: None,
        })]
    #[should_panic]
    #[case(AttributeValue::Text(String::from("2002-13-13")), JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: Some(JsonSchemaPropertyFormat::Date),
            properties: None,
        })]
    fn test_attribute_conversion(#[case] value: AttributeValue, #[case] prop: JsonSchemaProperty) {
        AttestationAttributeValue::try_from_attribute_value(value, &prop).unwrap();
    }
}
