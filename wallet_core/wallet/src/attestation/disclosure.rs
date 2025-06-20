use indexmap::IndexMap;

use attestation_data::attributes::Attributes;
use attestation_data::auth::Organization;
use mdoc::Entry;
use mdoc::NameSpace;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use super::AttestationError;
use super::AttestationIdentity;
use super::AttestationPresentation;

impl AttestationPresentation {
    pub(crate) fn create_for_disclosure(
        metadata: NormalizedTypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attributes::from_mdoc_attributes(&metadata, mdoc_attributes)?;

        Self::create_from_attributes(
            AttestationIdentity::Ephemeral,
            metadata,
            issuer_organization,
            &nested_attributes,
        )
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::AttributesError;
    use attestation_data::auth::Organization;
    use mdoc::DataElementValue;
    use mdoc::Entry;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::attestation::AttestationPresentation;

    fn example_metadata() -> NormalizedTypeMetadata {
        NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"]), claim_metadata(&["entry2"])],
            ..UncheckedTypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::Boolean, None),
                ],
            )
        })
    }

    #[test]
    fn test_happy() {
        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
            vec![
                Entry {
                    name: String::from("entry1"),
                    value: DataElementValue::Text(String::from("value1")),
                },
                Entry {
                    name: String::from("entry2"),
                    value: DataElementValue::Bool(true),
                },
            ],
        )]);

        let attestation = AttestationPresentation::create_for_disclosure(
            example_metadata(),
            Organization::new_mock(),
            mdoc_attributes,
        )
        .unwrap();

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [
                (
                    vec![String::from("entry1")],
                    AttestationAttributeValue::Basic(AttributeValue::Text(String::from("value1")))
                ),
                (
                    vec![String::from("entry2")],
                    AttestationAttributeValue::Basic(AttributeValue::Bool(true))
                ),
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_not_all_claims_need_to_match_attributes() {
        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
            vec![Entry {
                name: String::from("entry1"),
                value: DataElementValue::Text(String::from("value1")),
            }],
        )]);

        let attestation = AttestationPresentation::create_for_disclosure(
            example_metadata(),
            Organization::new_mock(),
            mdoc_attributes,
        );

        assert!(attestation.is_ok());
    }

    #[test]
    fn test_attributes_not_processed() {
        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"])],
            ..UncheckedTypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::String, None),
                ],
            )
        });

        let mdoc_attributes = IndexMap::from([(
            String::from("example_attestation_type"),
            vec![
                Entry {
                    name: String::from("entry1"),
                    value: DataElementValue::Text(String::from("value1")),
                },
                Entry {
                    name: String::from("entry2"),
                    value: DataElementValue::Text(String::from("value2")),
                },
            ],
        )]);

        let attestation =
            AttestationPresentation::create_for_disclosure(metadata, Organization::new_mock(), mdoc_attributes);

        assert_matches!(
            attestation,
            Err(AttestationError::Attributes(AttributesError::SomeAttributesNotProcessed(claims)))
                if claims == IndexMap::from([
                    (String::from("example_attestation_type"),
                    vec![Entry {
                        name: String::from("entry2"),
                        value: ciborium::value::Value::Text(String::from("value2"))
                    }]
                )]
            )
        );
    }
}
