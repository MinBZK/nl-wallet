use std::collections::HashMap;
use std::collections::HashSet;

use ciborium::Value;
use ciborium::value::Integer;
use mdoc::DataElementIdentifier;
use mdoc::IssuerSigned;
use mdoc::NameSpace;
use mdoc::holder::Mdoc;
use mdoc::holder::disclosure::PartialMdoc;

/// All decoded claims of a (partial) mdoc, grouped per namespace.
///
/// Nested CBOR maps only have text or integer keys, without duplicates.
#[derive(Debug, Clone, PartialEq)]
pub struct MdocClaims(HashMap<NameSpace, HashMap<DataElementIdentifier, Value>>);

#[derive(Debug, thiserror::Error)]
pub enum MdocClaimsError {
    #[error("invalid CBOR map key in attribute \"{element_identifier}\" of namespace \"{name_space}\": {error}")]
    InvalidMapKey {
        name_space: NameSpace,
        element_identifier: DataElementIdentifier,
        #[source]
        error: MapKeyError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MapKeyError {
    #[error("unsupported key type: {0:?}")]
    UnsupportedType(Box<Value>),

    #[error("duplicate key: {0:?}")]
    Duplicate(Box<Value>),
}

impl MdocClaims {
    pub fn from_mdoc(mdoc: Mdoc) -> Result<Self, MdocClaimsError> {
        Self::from_issuer_signed(mdoc.into_issuer_signed())
    }

    pub fn from_partial_mdoc(partial_mdoc: PartialMdoc) -> Result<Self, MdocClaimsError> {
        Self::from_issuer_signed(partial_mdoc.into_issuer_signed())
    }

    fn from_issuer_signed(issuer_signed: IssuerSigned) -> Result<Self, MdocClaimsError> {
        let claims = issuer_signed
            .into_entries_by_namespace()
            .into_iter()
            .map(|(name_space, entries)| {
                let attributes = entries
                    .into_iter()
                    .map(|entry| match check_map_keys(&entry.value) {
                        Ok(()) => Ok((entry.name, entry.value)),
                        Err(error) => Err(MdocClaimsError::InvalidMapKey {
                            name_space: name_space.clone(),
                            element_identifier: entry.name,
                            error,
                        }),
                    })
                    .collect::<Result<_, _>>()?;

                Ok((name_space, attributes))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(claims))
    }
}

/// Recursively checks that all CBOR maps in `value` have unique text or integer keys.
///
/// Recursion depth is bounded by `ciborium`'s nesting limit (256) for deserialized values.
fn check_map_keys(value: &Value) -> Result<(), MapKeyError> {
    #[derive(PartialEq, Eq, Hash)]
    enum MapKey<'a> {
        Text(&'a str),
        Integer(Integer),
    }

    match value {
        Value::Map(entries) => {
            let mut keys = HashSet::with_capacity(entries.len());

            entries.iter().try_for_each(|(key, value)| {
                let map_key = match key {
                    Value::Text(text) => MapKey::Text(text),
                    Value::Integer(integer) => MapKey::Integer(*integer),
                    _ => return Err(MapKeyError::UnsupportedType(Box::new(key.clone()))),
                };

                if !keys.insert(map_key) {
                    return Err(MapKeyError::Duplicate(Box::new(key.clone())));
                }

                check_map_keys(value)
            })
        }
        Value::Array(values) => values.iter().try_for_each(check_map_keys),
        Value::Tag(_, value) => check_map_keys(value),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;
    use std::collections::HashMap;

    use attestation_types::claim_path::ClaimPath;
    use ciborium::Value;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use indexmap::IndexMap;
    use mdoc::Entry;
    use mdoc::holder::Mdoc;
    use mdoc::holder::disclosure::PartialMdoc;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::Generate;
    use rstest::rstest;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;

    use super::MapKeyError;
    use super::MdocClaims;
    use super::MdocClaimsError;
    use super::check_map_keys;

    const DOC_TYPE: &str = "com.example.doc_type";
    const NAME_SPACE: &str = "com.example.name_space";
    const OTHER_NAME_SPACE: &str = "com.example.other_name_space";

    fn text(text: &str) -> Value {
        Value::Text(text.to_string())
    }

    fn map(key: Value, value: Value) -> Value {
        Value::Map(vec![(key, value)])
    }

    fn entry(name: &str, value: Value) -> Entry {
        Entry {
            name: name.to_string(),
            value,
        }
    }

    async fn mdoc_with_name_spaces(name_spaces: IndexMap<String, Vec<Entry>>) -> Mdoc {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let device_key = MockRemoteEcdsaKey::new("identifier".to_string(), SigningKey::generate());

        Mdoc::new_unverified_from_data(
            DOC_TYPE.to_string(),
            name_spaces,
            None,
            &ca,
            &device_key,
            &MockTimeGenerator::default(),
        )
        .await
    }

    fn example_name_spaces() -> IndexMap<String, Vec<Entry>> {
        IndexMap::from([
            (
                NAME_SPACE.to_string(),
                vec![
                    entry("given_name", text("Willeke")),
                    entry(
                        "address",
                        Value::Map(vec![
                            (text("street"), text("Turfmarkt")),
                            (Value::Integer(1.into()), text("positive")),
                            (Value::Integer((-1).into()), text("negative")),
                        ]),
                    ),
                ],
            ),
            (
                OTHER_NAME_SPACE.to_string(),
                vec![entry("number", Value::Integer(42.into()))],
            ),
        ])
    }

    fn claim_path(name_space: &str, element_identifier: &str) -> VecNonEmpty<ClaimPath> {
        vec_nonempty![
            ClaimPath::SelectByKey(name_space.to_string()),
            ClaimPath::SelectByKey(element_identifier.to_string()),
        ]
    }

    #[tokio::test]
    async fn test_mdoc_claims_from_mdoc() {
        let mdoc = mdoc_with_name_spaces(example_name_spaces()).await;

        let MdocClaims(claims) = MdocClaims::from_mdoc(mdoc).expect("mdoc should have valid claims");

        let expected: HashMap<_, HashMap<_, _>> = example_name_spaces()
            .into_iter()
            .map(|(name_space, entries)| {
                let attributes = entries.into_iter().map(|entry| (entry.name, entry.value)).collect();
                (name_space, attributes)
            })
            .collect();
        assert_eq!(claims, expected);
    }

    #[tokio::test]
    async fn test_mdoc_claims_from_partial_mdoc() {
        let mdoc = mdoc_with_name_spaces(example_name_spaces()).await;
        let partial_mdoc = PartialMdoc::try_new(mdoc, &[claim_path(NAME_SPACE, "address")]).unwrap();

        let MdocClaims(claims) = MdocClaims::from_partial_mdoc(partial_mdoc).expect("mdoc should have valid claims");

        assert!(claims.keys().eq([NAME_SPACE]));
        assert!(claims[NAME_SPACE].keys().eq(["address"]));
    }

    #[tokio::test]
    async fn test_mdoc_claims_from_partial_mdoc_without_claims() {
        let mdoc = mdoc_with_name_spaces(example_name_spaces()).await;
        let partial_mdoc = PartialMdoc::try_new(mdoc, &[]).unwrap();

        let MdocClaims(claims) = MdocClaims::from_partial_mdoc(partial_mdoc).expect("mdoc should have valid claims");

        assert!(claims.is_empty());
    }

    #[tokio::test]
    async fn test_mdoc_claims_from_mdoc_invalid_map_key() {
        let mut name_spaces = example_name_spaces();
        name_spaces[OTHER_NAME_SPACE].push(entry("invalid", map(Value::Bool(true), Value::Null)));
        let mdoc = mdoc_with_name_spaces(name_spaces).await;

        let error = MdocClaims::from_mdoc(mdoc).expect_err("mdoc should have an invalid map key");

        assert_matches!(
            error,
            MdocClaimsError::InvalidMapKey {
                name_space,
                element_identifier,
                error: MapKeyError::UnsupportedType(key),
            } if name_space == OTHER_NAME_SPACE && element_identifier == "invalid" && *key == Value::Bool(true)
        );
    }

    #[rstest]
    #[case::text(Value::Text("text".to_string()))]
    #[case::integer(Value::Integer(1.into()))]
    #[case::bool(Value::Bool(true))]
    #[case::null(Value::Null)]
    #[case::bytes(Value::Bytes(vec![1, 2, 3]))]
    #[case::float(Value::Float(1.5))]
    #[case::empty_map(Value::Map(vec![]))]
    #[case::empty_array(Value::Array(vec![]))]
    #[case::text_key(map(text("key"), text("value")))]
    #[case::positive_integer_key(map(Value::Integer(1.into()), text("value")))]
    #[case::negative_integer_key(map(Value::Integer((-1).into()), text("value")))]
    #[case::equal_text_and_integer_key(Value::Map(vec![
        (text("1"), Value::Null),
        (Value::Integer(1.into()), Value::Null),
    ]))]
    #[case::equal_keys_in_different_maps(Value::Array(vec![
        map(text("key"), Value::Null),
        map(text("key"), Value::Null),
    ]))]
    #[case::nested_map(map(text("outer"), map(Value::Integer((-1).into()), map(text("inner"), Value::Null))))]
    #[case::map_in_array(Value::Array(vec![map(text("key"), Value::Null)]))]
    #[case::map_in_tag(Value::Tag(1004, Box::new(map(text("key"), Value::Null))))]
    #[case::full_date(Value::Tag(1004, Box::new(text("2024-03-01"))))]
    fn test_check_map_keys_valid(#[case] value: Value) {
        check_map_keys(&value).expect("map keys should be valid");
    }

    #[rstest]
    #[case::bool_key(Value::Bool(true))]
    #[case::null_key(Value::Null)]
    #[case::bytes_key(Value::Bytes(vec![1]))]
    #[case::float_key(Value::Float(1.0))]
    #[case::array_key(Value::Array(vec![text("key")]))]
    #[case::map_key(map(text("key"), Value::Null))]
    #[case::tagged_text_key(Value::Tag(0, Box::new(text("key"))))]
    fn test_check_map_keys_unsupported_type(
        #[case] key: Value,
        #[values(
            |value| value,
            |value| map(text("outer"), value),
            |value| Value::Array(vec![Value::Null, value]),
            |value| Value::Tag(1004, Box::new(value)),
        )]
        wrap: fn(Value) -> Value,
    ) {
        let value = wrap(map(key.clone(), Value::Null));

        let error = check_map_keys(&value).expect_err("map keys should be invalid");

        assert_matches!(error, MapKeyError::UnsupportedType(error_key) if *error_key == key);
    }

    #[rstest]
    #[case::text_key(text("key"))]
    #[case::integer_key(Value::Integer((-1).into()))]
    fn test_check_map_keys_duplicate(
        #[case] key: Value,
        #[values(
            |value| value,
            |value| map(text("outer"), value),
            |value| Value::Array(vec![Value::Null, value]),
            |value| Value::Tag(1004, Box::new(value)),
        )]
        wrap: fn(Value) -> Value,
    ) {
        let value = wrap(Value::Map(vec![
            (key.clone(), Value::Null),
            (text("other"), Value::Null),
            (key.clone(), Value::Bool(true)),
        ]));

        let error = check_map_keys(&value).expect_err("map keys should be invalid");

        assert_matches!(error, MapKeyError::Duplicate(error_key) if *error_key == key);
    }
}
