use std::collections::VecDeque;

use indexmap::IndexMap;
use itertools::Itertools;
use nl_wallet_mdoc::utils::auth::Organization;
use openid4vc::attributes::Attribute;
use openid4vc::credential_payload::CredentialPayload;
use sd_jwt::metadata::ClaimPath;
use sd_jwt::metadata::TypeMetadata;

use crate::attestation::AttestationError;
use crate::attestation::AttestationIdentity;
use crate::Attestation;
use crate::AttestationAttribute;
use crate::LocalizedString;

impl Attestation {
    pub(crate) fn from_credential_payload(
        identity: AttestationIdentity,
        payload: CredentialPayload,
        metadata: TypeMetadata,
        issuer_organization: Organization,
    ) -> Result<Self, AttestationError> {
        let attributes = metadata
            .claims
            .into_iter()
            .map(|claim| {
                let key = claim.path.iter().join(".");
                let mut paths = claim.path.iter().collect::<VecDeque<_>>();
                let attribute = Self::select_attribute(&mut paths, &payload.attributes);
                let attribute_value = match attribute {
                    Some(Attribute::Single(value)) => Ok(value),
                    _ => Err(AttestationError::AttributeNotFoundForClaim(claim.clone())),
                }?;

                let attribute = AttestationAttribute {
                    key,
                    value: attribute_value.into(),
                    labels: claim.display.into_iter().map(LocalizedString::from).collect(),
                };
                Ok(attribute)
            })
            .collect::<Result<_, _>>()?;

        let attestation = Attestation {
            identity,
            display_metadata: metadata.display,
            attestation_type: payload.attestation_type,
            issuer: issuer_organization,
            attributes,
        };

        Ok(attestation)
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

#[cfg(test)]
mod test {
    use std::sync::LazyLock;

    use assert_matches::assert_matches;
    use chrono::Utc;
    use http::Uri;
    use indexmap::IndexMap;

    use nl_wallet_mdoc::utils::auth::Organization;
    use openid4vc::attributes::Attribute;
    use openid4vc::attributes::AttributeValue;
    use openid4vc::credential_payload::CredentialPayload;
    use sd_jwt::metadata::ClaimMetadata;
    use sd_jwt::metadata::ClaimPath;
    use sd_jwt::metadata::ClaimSelectiveDisclosureMetadata;
    use sd_jwt::metadata::TypeMetadata;

    use crate::attestation::AttestationError;
    use crate::Attestation;
    use crate::AttestationIdentity;
    use crate::AttestationValue;

    static ATTRIBUTES: LazyLock<IndexMap<String, Attribute>> = LazyLock::new(|| {
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

    #[test]
    fn test_from_credential_payload_happy() {
        let attributes = &*ATTRIBUTES;

        let mut metadata = TypeMetadata::new_example();
        metadata.claims = vec![
            ClaimMetadata {
                path: vec![ClaimPath::SelectByKey(String::from("single"))].try_into().unwrap(),
                display: vec![],
                sd: ClaimSelectiveDisclosureMetadata::Always,
                svg_id: None,
            },
            ClaimMetadata {
                path: vec![
                    ClaimPath::SelectByKey(String::from("nested_1a")),
                    ClaimPath::SelectByKey(String::from("nested_1b")),
                    ClaimPath::SelectByKey(String::from("nested_1c")),
                ]
                .try_into()
                .unwrap(),
                display: vec![],
                sd: ClaimSelectiveDisclosureMetadata::Always,
                svg_id: None,
            },
        ];

        let payload = CredentialPayload {
            attestation_type: String::from("pid456"),
            issuer: Uri::from_static("data://org.example.com/org1"),
            issued_at: Some(Utc::now()),
            expires: Some(Utc::now()),
            not_before: None,
            attributes: attributes.clone(),
        };

        let attestation = Attestation::from_credential_payload(
            AttestationIdentity::Ephemeral,
            payload,
            metadata,
            Organization::new_mock(),
        )
        .unwrap();

        let attrs = attestation
            .attributes
            .iter()
            .map(|attr| {
                (
                    attr.key.as_str(),
                    match &attr.value {
                        AttestationValue::String { value } => value.to_string(),
                        AttestationValue::Boolean { value } => value.to_string(),
                        AttestationValue::Number { value } => value.to_string(),
                    },
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            [
                ("single", String::from("single")),
                ("nested_1a.nested_1b.nested_1c", String::from("nested_value"))
            ],
            attrs.as_slice()
        );
    }

    #[test]
    fn test_from_credential_payload_attribute_not_found() {
        let attributes = &*ATTRIBUTES;

        let mut metadata = TypeMetadata::new_example();
        metadata.claims = vec![ClaimMetadata {
            path: vec![ClaimPath::SelectByKey(String::from("not_found"))]
                .try_into()
                .unwrap(),
            display: vec![],
            sd: ClaimSelectiveDisclosureMetadata::Always,
            svg_id: None,
        }];

        let payload = CredentialPayload {
            attestation_type: String::from("pid123"),
            issuer: Uri::from_static("data://org.example.com/org2"),
            issued_at: Some(Utc::now()),
            expires: Some(Utc::now()),
            not_before: None,
            attributes: attributes.clone(),
        };

        let attestation = Attestation::from_credential_payload(
            AttestationIdentity::Ephemeral,
            payload,
            metadata,
            Organization::new_mock(),
        );
        assert_matches!(attestation, Err(AttestationError::AttributeNotFoundForClaim(_)));
    }

    #[test]
    fn test_select_single_attribute_happy() {
        let attributes = &*ATTRIBUTES;

        let result = Attestation::select_attribute(
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

        let result = Attestation::select_attribute(
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

        let result = Attestation::select_attribute(
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

        let result = Attestation::select_attribute(
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

        let result = Attestation::select_attribute(
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

        let result = Attestation::select_attribute(&mut vec![].into(), attributes);
        assert_matches!(result, None, "selecting nothing should find nothing");
    }
}
