use indexmap::IndexMap;

use mdoc::unsigned::Entry;
use mdoc::utils::auth::Organization;
use mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;

use super::Attestation;
use super::AttestationError;
use super::AttestationIdentity;

impl Attestation {
    pub(crate) fn create_for_issuance(
        identity: AttestationIdentity,
        metadata: NormalizedTypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attribute::from_mdoc_attributes(&metadata, mdoc_attributes)?;

        Self::create_from_attributes(identity, metadata, issuer_organization, nested_attributes)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use chrono::NaiveDate;
    use indexmap::IndexMap;
    use mdoc::unsigned::Entry;

    use mdoc::utils::auth::Organization;
    use openid4vc::attributes::AttributeError;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt_vc_metadata::JsonSchemaPropertyType;
    use sd_jwt_vc_metadata::NormalizedTypeMetadata;
    use sd_jwt_vc_metadata::UncheckedTypeMetadata;

    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::issuance::mock::create_bsn_only_unsigned_mdoc;
    use crate::issuance::mock::create_example_unsigned_mdoc;
    use crate::Attestation;
    use crate::AttestationIdentity;

    #[test]
    fn test_happy() {
        let (unsigned_mdoc, metadata) = create_example_unsigned_mdoc();

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            NormalizedTypeMetadata::from_single_example(metadata.into_inner()),
            Organization::new_mock(),
            unsigned_mdoc.attributes.into_inner(),
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
                    vec![String::from("birth_date")],
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
        let (unsigned_mdoc, _) = create_bsn_only_unsigned_mdoc();

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            &unsigned_mdoc.doc_type,
            &[
                ("not_found", JsonSchemaPropertyType::String, None),
                ("bsn", JsonSchemaPropertyType::String, None),
            ],
        ));

        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            unsigned_mdoc.attributes.into_inner(),
        )
        .expect("creating new Attestation should be successful");

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| (attr.key.clone(), attr.value.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            [(
                vec![String::from("bsn")],
                AttestationAttributeValue::Basic(AttributeValue::Text(String::from("999999999")))
            ),],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_attribute_not_processed() {
        let (unsigned_mdoc, _) = create_example_unsigned_mdoc();

        let metadata = NormalizedTypeMetadata::from_single_example(UncheckedTypeMetadata::example_with_claim_names(
            &unsigned_mdoc.doc_type,
            &[
                ("family_name", JsonSchemaPropertyType::String, None),
                ("given_name", JsonSchemaPropertyType::String, None),
                ("age_over_18", JsonSchemaPropertyType::Boolean, None),
            ],
        ));

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            unsigned_mdoc.attributes.into_inner(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(
            error,
            AttestationError::Attribute(AttributeError::SomeAttributesNotProcessed(claims))
                if claims == IndexMap::from([
                    (String::from("com.example.pid"),
                    vec![Entry {
                        name: String::from("birth_date"),
                        value: ciborium::value::Value::Text("1997-05-10".to_string())
                    }]
                )]
            )
        );
    }
}
