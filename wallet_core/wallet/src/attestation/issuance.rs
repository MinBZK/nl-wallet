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
    pub(crate) fn create_for_issuance(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attributes::from_mdoc_attributes(&metadata, mdoc_attributes)?;

        Self::create_from_attributes(identity, metadata, issuer_organization, &nested_attributes)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use indexmap::IndexMap;

    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::AttributesError;
    use attestation_data::auth::Organization;
    use mdoc::Entry;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;
    use utils::generator::mock::MockTimeGenerator;

    use super::super::AttestationAttributeValue;
    use super::super::AttestationError;
    use super::super::AttestationIdentity;
    use super::super::AttestationPresentation;
    use super::super::BSN_ATTR_NAME;
    use super::super::PID_DOCTYPE;
    use super::super::test::create_bsn_only_mdoc_attributes;
    use super::super::test::create_example_mdoc_attributes;

    #[test]
    fn test_happy() {
        let (mdoc_attributes, metadata) = create_example_mdoc_attributes(&MockTimeGenerator::default());

        let attestation = AttestationPresentation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            NormalizedTypeMetadata::from_single_example(metadata.into_inner()),
            Organization::new_mock(),
            mdoc_attributes,
        )
        .expect("creating new Attestation should be successful");

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [
                (
                    vec![String::from("family_name")],
                    AttestationAttributeValue::Basic(AttributeValue::Text(String::from("De Bruijn")))
                ),
                (
                    vec![String::from("given_name")],
                    AttestationAttributeValue::Basic(AttributeValue::Text(String::from("Willeke Liselotte")))
                ),
                (
                    vec![String::from("birthdate")],
                    AttestationAttributeValue::Date(NaiveDate::from_ymd_opt(1997, 5, 10).unwrap())
                ),
                (
                    vec![String::from("age_over_18")],
                    AttestationAttributeValue::Basic(AttributeValue::Bool(true))
                ),
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_attribute_not_found() {
        let (mdoc_attributes, _) = create_bsn_only_mdoc_attributes(&MockTimeGenerator::default());

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            PID_DOCTYPE,
            &[
                ("not_found", JsonSchemaPropertyType::String, None),
                (BSN_ATTR_NAME, JsonSchemaPropertyType::String, None),
            ],
        ));

        let attestation = AttestationPresentation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            mdoc_attributes,
        )
        .expect("creating new Attestation should be successful");

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [(
                vec![BSN_ATTR_NAME.to_string()],
                AttestationAttributeValue::Basic(AttributeValue::Text(String::from("999999999")))
            ),],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_attribute_not_processed() {
        let (mdoc_attributes, _) = create_example_mdoc_attributes(&MockTimeGenerator::default());

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            PID_DOCTYPE,
            &[
                ("family_name", JsonSchemaPropertyType::String, None),
                ("given_name", JsonSchemaPropertyType::String, None),
                ("age_over_18", JsonSchemaPropertyType::Boolean, None),
            ],
        ));

        let error = AttestationPresentation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            mdoc_attributes,
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(
            error,
            AttestationError::Attributes(AttributesError::SomeAttributesNotProcessed(claims))
                if claims == IndexMap::from([
                    (String::from(PID_DOCTYPE),
                    vec![Entry {
                        name: String::from("birthdate"),
                        value: ciborium::value::Value::Text("1997-05-10".to_string())
                    }]
                )]
            )
        );
    }
}
