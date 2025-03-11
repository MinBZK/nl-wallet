use indexmap::IndexMap;

use nl_wallet_mdoc::unsigned::Entry;
use nl_wallet_mdoc::utils::auth::Organization;
use nl_wallet_mdoc::NameSpace;
use openid4vc::attributes::Attribute;
use sd_jwt::metadata::TypeMetadata;

use super::Attestation;
use super::AttestationError;
use super::AttestationIdentity;
use super::AttributeSelectionMode;

impl Attestation {
    pub(crate) fn create_for_issuance(
        identity: AttestationIdentity,
        metadata: TypeMetadata,
        issuer_organization: Organization,
        mdoc_attributes: IndexMap<NameSpace, Vec<Entry>>,
    ) -> Result<Self, AttestationError> {
        let nested_attributes = Attribute::from_mdoc_attributes(&metadata.vct, mdoc_attributes)?;

        Self::create_from_attributes(
            identity,
            metadata,
            issuer_organization,
            nested_attributes,
            AttributeSelectionMode::Issuance,
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use indexmap::IndexMap;

    use nl_wallet_mdoc::unsigned::Entry;
    use nl_wallet_mdoc::utils::auth::Organization;
    use nl_wallet_mdoc::DataElementValue;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::JsonSchemaPropertyType;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::Attestation;
    use crate::AttestationIdentity;

    fn example_metadata() -> TypeMetadata {
        TypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"]), claim_metadata(&["entry2"])],
            ..TypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::Boolean, None),
                ],
            )
        }
    }

    fn example_mdoc_attributes() -> IndexMap<String, Vec<Entry>> {
        IndexMap::from([(
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
        )])
    }

    #[test]
    fn test_happy() {
        let attestation = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            example_metadata(),
            Organization::new_mock(),
            example_mdoc_attributes(),
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
    fn test_attribute_not_found() {
        let metadata = TypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["not_found"])],
            ..TypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::Boolean, None),
                    ("not_found", JsonSchemaPropertyType::String, None),
                ],
            )
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            example_mdoc_attributes(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(error, AttestationError::AttributeNotFoundForClaim(_));
    }

    #[test]
    fn test_attribute_not_processed() {
        let metadata = TypeMetadata {
            vct: String::from("example_attestation_type"),
            claims: vec![claim_metadata(&["entry1"])],
            ..TypeMetadata::example_with_claim_names(
                "example_attestation_type",
                &[
                    ("entry1", JsonSchemaPropertyType::String, None),
                    ("entry2", JsonSchemaPropertyType::Boolean, None),
                ],
            )
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            example_mdoc_attributes(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(
            error,
            AttestationError::AttributeNotProcessedByClaim(keys)
                if keys == HashSet::from([vec![String::from("entry2")]])
        );
    }
}
