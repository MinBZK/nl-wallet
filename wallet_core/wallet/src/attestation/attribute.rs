use std::collections::HashSet;

use chrono::NaiveDate;

use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::auth::Organization;
use sd_jwt_vc_metadata::ClaimPath;
use sd_jwt_vc_metadata::JsonSchemaProperty;
use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
use sd_jwt_vc_metadata::JsonSchemaPropertyType;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
use utils::vec_at_least::VecNonEmpty;

use super::AttestationAttribute;
use super::AttestationAttributeValue;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttestationPresentation;

impl AttestationPresentation {
    // Construct a new `AttestationPresentation` from a combination of metadata and nested attributes.
    // This method has different behaviour depending on the `selection_mode` parameter.
    pub fn create_from_attributes(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer: Organization,
        nested_attributes: &Attributes,
    ) -> Result<Self, AttestationError> {
        let (attestation_type, display_metadata, claims, schema) = metadata.into_presentation_components();

        // Root JSON property that makes folding in loop easier
        let root_json_property = JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Object,
            format: None,
            properties: Some(schema.into_properties().properties),
        };

        // For every claim in the metadata, find the correct attribute
        // and convert it to a `AttestationAttribute` value (with optionally Json Schema metadata).
        let flattened_attributes = nested_attributes.flattened();
        let mut attributes = Vec::with_capacity(flattened_attributes.len());
        for claim in claims {
            let Some(claim_path) = claim
                .path
                .into_iter()
                .map(ClaimPath::try_into_key_path)
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };

            // Extract the JSON Schema properties from the metadata,
            // and try to use the metadata to enrich the attribute value.
            let json_property = claim_path.iter().try_fold(&root_json_property, |json_property, name| {
                json_property
                    .properties
                    .as_ref()
                    .and_then(|properties| properties.get(name))
            });

            // Get value of claim out of the nested attributes via flattened view
            let value = {
                // claim.path is also VecNonEmpty
                let path_with_refs =
                    VecNonEmpty::try_from(claim_path.iter().map(String::as_str).collect::<Vec<&str>>()).unwrap();
                // Cannot use swap_remove here to make the error checking easier
                let Some(&value) = flattened_attributes.get(&path_with_refs) else {
                    continue;
                };
                AttestationAttributeValue::try_from_attribute_value(value.clone(), json_property)?
            };

            attributes.push(AttestationAttribute {
                key: claim_path,
                metadata: claim.display,
                value,
                svg_id: claim.svg_id.map(String::from),
            })
        }

        // Should not happen as the attributes should be validated by `Attributes::validate`
        // on receiving which does same check
        if attributes.len() != flattened_attributes.len() {
            let mut paths = flattened_attributes
                .into_iter()
                .map(|(path, _)| path.iter().map(ToString::to_string).collect::<Vec<_>>())
                .collect::<HashSet<_>>();
            for attribute in attributes {
                paths.remove(&attribute.key);
            }
            return Err(AttestationError::AttributesNotProcessedByClaim(paths));
        }

        // Finally, construct the `AttestationPresentation` type.
        Ok(AttestationPresentation {
            identity,
            display_metadata,
            attestation_type,
            issuer,
            attributes,
        })
    }
}

impl AttestationAttributeValue {
    fn try_from_attribute_value(
        value: AttributeValue,
        schema_property: Option<&JsonSchemaProperty>,
    ) -> Result<Self, AttestationError> {
        let schema_type = schema_property.map(|property| property.r#type);
        match (schema_type, value) {
            (None, AttributeValue::Bool(bool))
            | (Some(JsonSchemaPropertyType::Boolean), AttributeValue::Bool(bool)) => {
                Ok(AttestationAttributeValue::Basic(AttributeValue::Bool(bool)))
            }

            (None, AttributeValue::Integer(integer))
            | (Some(JsonSchemaPropertyType::Integer), AttributeValue::Integer(integer)) => {
                Ok(AttestationAttributeValue::Basic(AttributeValue::Integer(integer)))
            }

            (None, AttributeValue::Text(text)) | (Some(JsonSchemaPropertyType::String), AttributeValue::Text(text)) => {
                let format = schema_property.and_then(|property| property.format);
                if let Some(JsonSchemaPropertyFormat::Date) = format {
                    let date = NaiveDate::parse_from_str(&text, "%Y-%m-%d")?;
                    Ok(AttestationAttributeValue::Date(date))
                } else {
                    Ok(AttestationAttributeValue::Basic(AttributeValue::Text(text)))
                }
            }
            (_, value) => Err(AttestationError::AttributeConversion(value, schema_type)),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;

    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use attestation_data::auth::Organization;
    use sd_jwt_vc_metadata::ClaimDisplayMetadata;
    use sd_jwt_vc_metadata::ClaimMetadata;
    use sd_jwt_vc_metadata::ClaimPath;
    use sd_jwt_vc_metadata::ClaimSelectiveDisclosureMetadata;
    use sd_jwt_vc_metadata::JsonSchemaProperty;
    use sd_jwt_vc_metadata::JsonSchemaPropertyFormat;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;

    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::AttestationAttribute;
    use crate::AttestationIdentity;
    use crate::AttestationPresentation;

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

    fn example_attributes() -> Attributes {
        IndexMap::from([
            (
                "name".to_string(),
                Attribute::Single(AttributeValue::Text("Wallet".to_string())),
            ),
            (
                "birth_date".to_string(),
                Attribute::Single(AttributeValue::Text("1996-06-16".to_string())),
            ),
            (
                "address".to_string(),
                Attribute::Nested(IndexMap::from([
                    (
                        "street".to_string(),
                        Attribute::Single(AttributeValue::Text("Gracht".to_string())),
                    ),
                    ("number".to_string(), Attribute::Single(AttributeValue::Integer(123))),
                ])),
            ),
        ])
        .into()
    }

    #[test]
    fn test_create_from_attributes() {
        let attributes = example_attributes();
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"lang": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"lang": "en", "label": "birth date"}],
                },
                {
                    "path": ["address", "street"],
                    "display": [{"lang": "en", "label": "address street"}],
                },
                {
                    "path": ["address", "number"],
                    "display": [{"lang": "en", "label": "address number"}],
                },
                {
                    "path": ["country", "iso"],
                    "display": [{"lang": "en", "label": "country iso"}],
                },
            ],
            "schema": { "properties": {
                "name": { "type": "string" },
                "birth_date": { "type": "string", "format": "date" },
                "address": { "type": "object", "properties": {
                    "street": { "type": "string" },
                    "number": { "type": "integer" },
                } },
            } }
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let attestation_presentation = AttestationPresentation::create_from_attributes(
            AttestationIdentity::Ephemeral,
            type_metadata,
            Organization::new_mock(),
            &attributes,
        )
        .unwrap();
        assert_eq!(attestation_presentation.issuer, Organization::new_mock());
        assert_eq!(
            attestation_presentation.attributes,
            vec![
                AttestationAttribute {
                    key: vec!["name".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        lang: "en".to_string(),
                        label: "name".to_string(),
                        description: None
                    }],
                    value: AttestationAttributeValue::Basic(AttributeValue::Text("Wallet".to_string())),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec!["birth_date".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        lang: "en".to_string(),
                        label: "birth date".to_string(),
                        description: None
                    }],
                    value: AttestationAttributeValue::Date(NaiveDate::from_ymd_opt(1996, 6, 16).unwrap()),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec!["address".to_string(), "street".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        lang: "en".to_string(),
                        label: "address street".to_string(),
                        description: None
                    }],
                    value: AttestationAttributeValue::Basic(AttributeValue::Text("Gracht".to_string())),
                    svg_id: None
                },
                AttestationAttribute {
                    key: vec!["address".to_string(), "number".to_string()],
                    metadata: vec![ClaimDisplayMetadata {
                        lang: "en".to_string(),
                        label: "address number".to_string(),
                        description: None
                    }],
                    value: AttestationAttributeValue::Basic(AttributeValue::Integer(123)),
                    svg_id: None
                },
            ]
        );
    }

    #[test]
    fn test_create_from_attributes_missing() {
        let attributes = example_attributes();
        let metadata_json = json!({
            "vct": "com.example.pid",
            "display": [{"lang": "en", "name": "example"}],
            "claims": [
                {
                    "path": ["name"],
                    "display": [{"lang": "en", "label": "name"}],
                },
                {
                    "path": ["birth_date"],
                    "display": [{"lang": "en", "label": "birth date"}],
                },
            ],
            "schema": {}
        });
        let type_metadata = NormalizedTypeMetadata::from_single_example(serde_json::from_value(metadata_json).unwrap());

        let error = AttestationPresentation::create_from_attributes(
            AttestationIdentity::Ephemeral,
            type_metadata,
            Organization::new_mock(),
            &attributes,
        )
        .unwrap_err();
        assert_matches!(error, AttestationError::AttributesNotProcessedByClaim(attributes) if attributes ==
        HashSet::from_iter(vec![vec!["address".to_string(), "street".to_string()], vec!["address".to_string(), "number".to_string()]]));
    }

    #[rstest]
    #[case(
        AttributeValue::Text(String::from("normal text")),
        None,
        Some(AttestationAttributeValue::Basic(AttributeValue::Text(String::from("normal text"))))
    )]
    #[case(
        AttributeValue::Bool(true),
        None,
        Some(AttestationAttributeValue::Basic(AttributeValue::Bool(true)))
    )]
    #[case(
        AttributeValue::Integer(0),
        None,
        Some(AttestationAttributeValue::Basic(AttributeValue::Integer(0)))
    )]
    #[case(AttributeValue::Text(String::from("normal text")), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: None,
            properties: None,
        }), Some(AttestationAttributeValue::Basic(AttributeValue::Text(String::from("normal text")))))
    ]
    #[case(AttributeValue::Bool(false), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Boolean,
            format: None,
            properties: None,
        }), Some(AttestationAttributeValue::Basic(AttributeValue::Bool(false))))]
    #[case(AttributeValue::Integer(123), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Integer,
            format: None,
            properties: None,
        }), Some(AttestationAttributeValue::Basic(AttributeValue::Integer(123))))]
    #[case(AttributeValue::Text(String::from("2002-12-28")), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: Some(JsonSchemaPropertyFormat::Date),
            properties: None,
        }), Some(AttestationAttributeValue::Date(NaiveDate::from_ymd_opt(2002, 12, 28).unwrap())))]
    #[case(AttributeValue::Text(String::from("text")), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::Number,
            format: None,
            properties: None,
        }), None)]
    #[case(AttributeValue::Text(String::from("2002-13-13")), Some(&JsonSchemaProperty {
            r#type: JsonSchemaPropertyType::String,
            format: Some(JsonSchemaPropertyFormat::Date),
            properties: None,
        }), None)]
    fn test_attribute_conversion(
        #[case] value: AttributeValue,
        #[case] prop: Option<&JsonSchemaProperty>,
        #[case] expected: Option<AttestationAttributeValue>,
    ) {
        let result = AttestationAttributeValue::try_from_attribute_value(value, prop);
        assert_eq!(result.ok(), expected);
    }
}
