use indexmap::IndexMap;

use mdoc::unsigned::Entry;
use mdoc::utils::auth::Organization;
use mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt::metadata::TypeMetadata;

use super::Attestation;
use super::AttestationError;
use super::AttestationIdentity;

impl Attestation {
    pub(crate) fn create_for_disclosure(
        metadata: TypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attribute::from_mdoc_attributes(&metadata, mdoc_attributes)?;

        Self::create_from_attributes(
            AttestationIdentity::Ephemeral,
            metadata,
            issuer_organization,
            nested_attributes,
        )
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use mdoc::unsigned::Entry;
    use mdoc::utils::auth::Organization;
    use mdoc::DataElementValue;
    use openid4vc::attributes::AttributeError;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::JsonSchemaPropertyType;
    use sd_jwt::metadata::TypeMetadata;
    use sd_jwt::metadata::UncheckedTypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::Attestation;
    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;

    fn example_metadata() -> TypeMetadata {
        TypeMetadata::try_new(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"]), claim_metadata(&["entry2"])],
            ..TypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::Boolean, None),
                ],
            )
            .into_inner()
        })
        .unwrap()
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

        let attestation =
            Attestation::create_for_disclosure(example_metadata(), Organization::new_mock(), mdoc_attributes).unwrap();

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

        let attestation =
            Attestation::create_for_disclosure(example_metadata(), Organization::new_mock(), mdoc_attributes);

        assert!(attestation.is_ok());
    }

    #[test]
    fn test_attributes_not_processed() {
        let metadata = TypeMetadata::try_new(UncheckedTypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"])],
            ..TypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::String, None),
                ],
            )
            .into_inner()
        })
        .unwrap();

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

        let attestation = Attestation::create_for_disclosure(metadata, Organization::new_mock(), mdoc_attributes);

        assert_matches!(
            attestation,
            Err(AttestationError::Attribute(AttributeError::SomeAttributesNotProcessed(claims)))
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
