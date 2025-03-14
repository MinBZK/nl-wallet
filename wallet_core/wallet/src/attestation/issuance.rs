use indexmap::IndexMap;

use mdoc::unsigned::Entry;
use mdoc::utils::auth::Organization;
use mdoc::NameSpace;
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
    use chrono::NaiveDate;

    use mdoc::utils::auth::Organization;
    use openid4vc::attributes::AttributeValue;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::attribute::test::claim_metadata;
    use crate::attestation::AttestationAttributeValue;
    use crate::attestation::AttestationError;
    use crate::issuance::mock::create_example_unsigned_mdoc;
    use crate::Attestation;
    use crate::AttestationIdentity;

    #[test]
    fn test_happy() {
        let (unsigned_mdoc, metadata) = create_example_unsigned_mdoc();

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
        let (unsigned_mdoc, metadata) = create_example_unsigned_mdoc();

        let metadata = TypeMetadata {
            vct: unsigned_mdoc.doc_type,
            claims: vec![claim_metadata(&["not_found"])],
            ..metadata
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            unsigned_mdoc.attributes.into_inner(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(error, AttestationError::AttributeNotFoundForClaim(_));
    }

    #[test]
    fn test_attribute_not_processed() {
        let (unsigned_mdoc, metadata) = create_example_unsigned_mdoc();

        let metadata = TypeMetadata {
            vct: unsigned_mdoc.doc_type,
            claims: vec![
                claim_metadata(&["family_name"]),
                claim_metadata(&["given_name"]),
                claim_metadata(&["birth_date"]),
            ],
            ..metadata
        };

        let error = Attestation::create_for_issuance(
            AttestationIdentity::Ephemeral,
            metadata,
            Organization::new_mock(),
            unsigned_mdoc.attributes.into_inner(),
        )
        .expect_err("creating new Attestation should not be successful");

        assert_matches!(
            error,
            AttestationError::AttributeNotProcessedByClaim(keys)
                if keys == HashSet::from([vec![String::from("age_over_18")]])
        );
    }
}
