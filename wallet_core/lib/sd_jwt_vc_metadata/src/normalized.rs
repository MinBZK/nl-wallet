use std::collections::HashMap;
use std::mem;
use std::num::NonZeroUsize;

use itertools::Either;
use itertools::Itertools;
use jsonschema::ValidationError;

use utils::vec_at_least::VecNonEmpty;

use crate::chain::SortedTypeMetadata;
use crate::metadata::ClaimDisplayMetadata;
use crate::metadata::ClaimMetadata;
use crate::metadata::ClaimPath;
use crate::metadata::ClaimSelectiveDisclosureMetadata;
use crate::metadata::DisplayMetadata;
use crate::metadata::JsonSchema;
use crate::metadata::SchemaOption;
use crate::metadata::UncheckedTypeMetadata;

#[derive(Debug, thiserror::Error)]
pub enum NormalizedTypeMetadataError {
    #[error(
        "claim selective disclosure option is inconsistent for vct \"{}\" at path {}",
        .0,
        ClaimMetadata::path_to_string(.1.as_ref())
    )]
    InconsistentSelectiveDisclosure(String, VecNonEmpty<ClaimPath>),

    #[error(
        "metadata extension for vct \"{}\" missing claim(s) at path(s) {}",
        .0,
        .1.iter().map(|path| ClaimMetadata::path_to_string(path.as_ref())).join(", ")
    )]
    ExtensionMissingClaims(String, Vec<VecNonEmpty<ClaimPath>>),

    #[error("No display metadata present in any of the chain documents")]
    NoDisplayMetadata,

    #[error("JSON schema is not embedded for vct \"{0}\"")]
    NoEmbeddedSchema(String),

    #[error("No claim display metadata present for claim at path: {}", ClaimMetadata::path_to_string(.0.as_ref()))]
    NoClaimDisplayMetadata(VecNonEmpty<ClaimPath>),

    #[error("found missing `svg_id`s: {}", .0.join(", "))]
    MissingSvgIds(Vec<String>),
}

#[derive(Debug, thiserror::Error)]
#[error("JSON schema validation failed for vct \"{0}\": {1}")]
pub struct TypeMetadataValidationError(String, #[source] Box<ValidationError<'static>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTypeMetadata {
    vcts: VecNonEmpty<String>,
    display: VecNonEmpty<DisplayMetadata>,
    claims: Vec<ClaimMetadata>,
    schemas: VecNonEmpty<JsonSchema>,
}

impl NormalizedTypeMetadata {
    /// Attempt to combine all of the SD-JWT VC type metadata in a chain that is sorted from leaf to root into a single
    /// [`NormalizedTypeMetadata`], which can then be used to both validate a received attestation and convert it to a
    /// display representation.
    pub(crate) fn try_from_sorted_metadata(
        sorted_metadata: SortedTypeMetadata,
    ) -> Result<Self, NormalizedTypeMetadataError> {
        let chain = sorted_metadata.into_inner();
        let chain_length = chain.len().get();

        // Extract the root metadata document, which is at the end of the chain. This is guaranteed to succeed because
        // of the use of `VecNonEmpty`.
        let (chain, root) = chain.into_inner_last();
        let UncheckedTypeMetadata {
            vct: root_vct,
            display: root_display,
            claims: root_claims,
            schema: root_schema,
            ..
        } = root.into_inner();

        // Extract some properties of each of the metadata documents into individual vectors, two of which cover the
        // entire chain while the other two skip the root document.
        let mut vcts = Vec::with_capacity(chain_length);
        let mut schemas = Vec::with_capacity(chain_length);

        let (display_extensions, claims_extensions): (Vec<_>, Vec<_>) = chain
            .into_iter()
            .map(|metadata| {
                let metadata = metadata.into_inner();

                vcts.push(metadata.vct);
                schemas.push(metadata.schema);

                (metadata.display, metadata.claims)
            })
            .unzip();

        vcts.push(root_vct);
        schemas.push(root_schema);

        // Merge the display properties of all of the documents, going from the root to the leaf.
        let display = display_extensions
            .into_iter()
            .rev()
            .fold(root_display, extend_display_properties);

        let display = VecNonEmpty::try_from(display).map_err(|_| NormalizedTypeMetadataError::NoDisplayMetadata)?;

        // Merge the claims of all of the documents, going from the root to the leaf.
        let claims = claims_extensions.into_iter().enumerate().rev().try_fold(
            root_claims,
            |normalized_claims, (index, extending_claims)| {
                ClaimMetadata::extend_claims(normalized_claims, &vcts[index], extending_claims)
            },
        )?;

        for claim in &claims {
            if claim.display.is_empty() {
                return Err(NormalizedTypeMetadataError::NoClaimDisplayMetadata(claim.path.clone()));
            }
        }

        // Check if all of the JSON schemas are embedded and extract them.
        let schemas = schemas
            .into_iter()
            .zip(vcts.iter())
            .map(|(schema, vct)| match schema {
                SchemaOption::Embedded { schema } => Ok(*schema),
                _ => Err(NormalizedTypeMetadataError::NoEmbeddedSchema(vct.clone())),
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .unwrap();

        // Check if the `svg_id`s are still consistent after normalizing. Note that we do not have to check for
        // duplicate `svg_id`s here, as this is already checked for each metadata document individually and, because all
        // of the claims of an extended document need to be repeated, this then also holds for the normalized claims.
        let missing_svg_ids = crate::metadata::find_missing_svg_ids(display.as_ref(), &claims);
        if !missing_svg_ids.is_empty() {
            return Err(NormalizedTypeMetadataError::MissingSvgIds(missing_svg_ids));
        }

        let vcts = vcts.try_into().unwrap();

        let normalized = Self {
            vcts,
            display,
            claims,
            schemas,
        };

        Ok(normalized)
    }

    pub fn vct_count(&self) -> NonZeroUsize {
        self.vcts.len()
    }

    pub fn vct(&self) -> &str {
        self.vcts.first()
    }

    pub fn display(&self) -> &[DisplayMetadata] {
        self.display.as_ref()
    }

    pub fn claims(&self) -> &[ClaimMetadata] {
        &self.claims
    }

    pub fn into_presentation_components(
        self,
    ) -> (String, VecNonEmpty<DisplayMetadata>, Vec<ClaimMetadata>, JsonSchema) {
        (
            self.vcts.into_first(),
            self.display,
            self.claims,
            self.schemas.into_first(),
        )
    }

    pub fn validate(&self, attestation_json: &serde_json::Value) -> Result<(), TypeMetadataValidationError> {
        for (vct, schema) in self.vcts.iter().zip(self.schemas.as_slice()) {
            schema
                .validate(attestation_json)
                .map_err(|error| TypeMetadataValidationError(vct.clone(), error))?;
        }

        Ok(())
    }

    /// Returns all claim paths that only consist out of `SelectByKey` as a `VecNonEmpty` of `&str`
    pub fn claim_key_paths(&self) -> Vec<VecNonEmpty<&str>> {
        self.claims
            .iter()
            .map(|claim| claim.path.iter().filter_map(|path| path.try_key_path()).collect_vec())
            .filter_map(|key_path| VecNonEmpty::try_from(key_path).ok())
            .collect()
    }
}

impl ClaimMetadata {
    /// Extend a vector of [`ClaimMetadata`] with another one. The extending vector must contain a superset of the claim
    /// paths contained in the extended vector. Each claim in the extended vector will have both its `display` and `sd`
    /// fields combined, according to the logic below. The `svg_id` field is simply overwritten by the extending claim.
    fn extend_claims(
        extended_claims: Vec<Self>,
        extending_vct: &str,
        extending_claims: Vec<Self>,
    ) -> Result<Vec<Self>, NormalizedTypeMetadataError> {
        // Convert the extended claims vector to a map, keyed by its original index. The entries will be consumed later.
        let mut extended_claims_by_index = extended_claims.into_iter().enumerate().collect::<HashMap<_, _>>();

        // Create a map of claim paths to indices into the original extended claims vector. We use this layer of
        // indirection through the indices so we can use a reference to the claim path as the key.
        let mut extended_index_by_path = extended_claims_by_index
            .iter()
            .map(|(index, claim)| (claim.path.as_ref(), *index))
            .collect::<HashMap<_, _>>();

        // For each extending claim, find the index into the extended claims vector that match the path, if present.
        let found_extended_indices = extending_claims
            .iter()
            .map(|extending_claim| extended_index_by_path.remove(extending_claim.path.as_ref()))
            .collect_vec();

        // Finally combine the extended claims vector with the found indices to create a new vector with the merged
        // result, extending the the individual claims if present.
        let extending_claims = extending_claims
            .into_iter()
            .zip(found_extended_indices)
            .map(|(extending_claim, extended_index)| {
                let extended_claim = extended_index.and_then(|index| extended_claims_by_index.remove(&index));
                if let Some(extended_claim) = extended_claim {
                    // The display entries for the claim are merged according to the logic described below.
                    let display = extend_display_properties(extended_claim.display, extending_claim.display);

                    // The selective disclosure option is updated, but only if the extended value is `allowed`.
                    let sd = ClaimSelectiveDisclosureMetadata::extend(extended_claim.sd, extending_claim.sd).ok_or(
                        NormalizedTypeMetadataError::InconsistentSelectiveDisclosure(
                            extending_vct.to_string(),
                            extended_claim.path,
                        ),
                    )?;

                    // The `svg_id`` is simply overwritten by the extending claim.
                    let svg_id = extending_claim.svg_id;

                    let claim = Self {
                        path: extending_claim.path,
                        display,
                        sd,
                        svg_id,
                    };

                    Ok(claim)
                } else {
                    Ok(extending_claim)
                }
            })
            .try_collect()?;

        // All extended claims should have been consumed now, otherwise return an error.
        if !extended_claims_by_index.is_empty() {
            let missing_paths = extended_claims_by_index
                .into_iter()
                .sorted_by_key(|(index, _)| *index)
                .map(|(_, claim)| claim.path)
                .collect();

            return Err(NormalizedTypeMetadataError::ExtensionMissingClaims(
                extending_vct.to_string(),
                missing_paths,
            ));
        }

        Ok(extending_claims)
    }
}

impl ClaimSelectiveDisclosureMetadata {
    /// Attempt to overwrite one [`ClaimSelectiveDisclosureMetadata`] with another and return the result if possible,
    /// which will return `None` if the value is changed and the extended value is not `allowed`.
    fn extend(extended_sd: Self, extending_sd: Self) -> Option<Self> {
        match (extended_sd, extending_sd) {
            (extended_sd, extending_sd) if extended_sd == extending_sd => Some(extending_sd),
            (Self::Allowed, extending_sd) => Some(extending_sd),
            _ => None,
        }
    }
}

trait HasLanguage {
    fn language(&self) -> &str;
}

impl HasLanguage for DisplayMetadata {
    fn language(&self) -> &str {
        &self.lang
    }
}

impl HasLanguage for ClaimDisplayMetadata {
    fn language(&self) -> &str {
        &self.lang
    }
}

/// Generic function for extending one vector of either [`DisplayMetadata`] or [`ClaimDisplayMetadata`] entries with
/// another. Entries in the extended vector with the same language as those in the extending vector will be overwritten,
/// maintaining the original order of the extended vector. Entries with new languages will be appended after those,
/// according to the order of the extending vector.
fn extend_display_properties<T>(mut extended_display: Vec<T>, extending_display: Vec<T>) -> Vec<T>
where
    T: HasLanguage,
{
    // First, build a map of languages to indices into the extended vector. We use this layer of indirection through the
    // indices so we can use a reference to the language as the key.
    let index_by_language = extended_display
        .iter()
        .enumerate()
        .map(|(index, display)| (display.language(), index))
        .collect::<HashMap<_, _>>();

    // Sort the entries of the extending vector into two groups, one for those replacing an entry in the extended vector
    // (along with the relevant index) and another for those that have new languages.
    let (to_be_replaced, to_be_added): (Vec<_>, Vec<_>) = extending_display.into_iter().partition_map(|display| {
        if let Some(index) = index_by_language.get(display.language()) {
            Either::Left((*index, display))
        } else {
            Either::Right(display)
        }
    });

    // Overwrite the entries in the first group...
    for (index, display) in to_be_replaced {
        let _ = mem::replace(&mut extended_display[index], display);
    }

    // ...and append the second group.
    extended_display.extend(to_be_added);

    extended_display
}

#[cfg(any(test, feature = "example_constructors"))]
mod example_constructors {
    use crate::VerifiedTypeMetadataDocuments;
    use crate::metadata::SchemaOption;
    use crate::metadata::UncheckedTypeMetadata;

    use super::NormalizedTypeMetadata;

    impl NormalizedTypeMetadata {
        pub fn from_single_example(metadata: UncheckedTypeMetadata) -> Self {
            let schema = match metadata.schema {
                SchemaOption::Embedded { schema } => *schema,
                _ => panic!(),
            };

            Self {
                vcts: vec![metadata.vct].try_into().unwrap(),
                display: metadata.display.try_into().unwrap(),
                claims: metadata.claims,
                schemas: vec![schema].try_into().unwrap(),
            }
        }

        pub fn empty_example() -> Self {
            Self::from_single_example(UncheckedTypeMetadata::empty_example())
        }

        pub fn example() -> Self {
            Self::from_single_example(UncheckedTypeMetadata::example())
        }

        pub fn nl_pid_example() -> Self {
            VerifiedTypeMetadataDocuments::nl_pid_example().to_normalized().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use itertools::Itertools;
    use rstest::rstest;
    use serde_json::json;
    use ssri::Integrity;

    use utils::vec_at_least::VecNonEmpty;

    use crate::chain::SortedTypeMetadata;
    use crate::metadata::ClaimDisplayMetadata;
    use crate::metadata::ClaimMetadata;
    use crate::metadata::ClaimPath;
    use crate::metadata::ClaimSelectiveDisclosureMetadata;
    use crate::metadata::DisplayMetadata;
    use crate::metadata::JsonSchema;
    use crate::metadata::SchemaOption;
    use crate::metadata::SvgId;
    use crate::metadata::TypeMetadata;
    use crate::metadata::UncheckedTypeMetadata;

    use super::NormalizedTypeMetadata;
    use super::NormalizedTypeMetadataError;
    use super::TypeMetadataValidationError;

    fn all_claim_paths(claims: &[ClaimMetadata]) -> Vec<&[ClaimPath]> {
        claims.iter().map(|claim| claim.path.as_ref()).collect()
    }

    #[test]
    fn test_normalized_type_metadata_try_from_metadata_chain() {
        let normalized =
            NormalizedTypeMetadata::try_from_sorted_metadata(SortedTypeMetadata::example_with_extensions())
                .expect("normalizing SD-JWT VC type metadata chain should succeed");

        let metadata = UncheckedTypeMetadata::example();
        let metadata_v2 = UncheckedTypeMetadata::example_v2();
        let metadata_v3 = UncheckedTypeMetadata::example_v3();

        // The vcts should be ordered from leaf to root.
        assert_eq!(
            normalized.vcts.as_ref(),
            vec![
                metadata_v3.vct.as_str(),
                metadata_v2.vct.as_str(),
                metadata.vct.as_str()
            ]
        );

        // The metadata display values should be merged, with existing values being updated and new values appended.
        assert_eq!(normalized.display.len().get(), 2);
        assert_ne!(normalized.display[0], metadata.display[0]);
        assert_eq!(normalized.display[0], metadata_v2.display[1]);
        assert_eq!(normalized.display[1], metadata_v2.display[0]);

        // The claim paths should be in the same order as they are in the leaf extension.
        assert_ne!(all_claim_paths(&normalized.claims), all_claim_paths(&metadata.claims));
        assert_ne!(
            all_claim_paths(&normalized.claims),
            all_claim_paths(&metadata_v2.claims)
        );
        assert_eq!(
            all_claim_paths(&normalized.claims),
            all_claim_paths(&metadata_v3.claims)
        );

        // All of the claims should have their selective disclosure state changed to a more strict option.
        assert!(
            !normalized
                .claims
                .iter()
                .any(|claim| claim.sd == ClaimSelectiveDisclosureMetadata::Allowed)
        );

        // All of the claims should have their "svg_id" property overwritten by the extension.
        assert!(
            normalized
                .claims
                .iter()
                .find(|claim| claim.path.as_ref() == vec![ClaimPath::SelectByKey("birth_date".to_string())])
                .unwrap()
                .svg_id
                .is_none()
        );
        assert_eq!(
            normalized
                .claims
                .iter()
                .find(|claim| claim.path.as_ref() == vec![ClaimPath::SelectByKey("nickname".to_string())])
                .unwrap()
                .svg_id
                .as_deref(),
            Some("nickname")
        );

        // The JSON schemas should be ordered from leaf to root.
        assert_eq!(normalized.schemas.len().get(), 3);
        assert_eq!(
            normalized.schemas.iter().collect_vec(),
            vec![&metadata_v3.schema, &metadata_v2.schema, &metadata.schema]
                .into_iter()
                .map(|schema_option| match schema_option {
                    SchemaOption::Embedded { schema } => schema.as_ref(),
                    _ => unreachable!(),
                })
                .collect_vec()
        );
    }

    #[test]
    fn test_normalized_type_metadata_validate() {
        let normalized =
            NormalizedTypeMetadata::try_from_sorted_metadata(SortedTypeMetadata::example_with_extensions())
                .expect("normalizing SD-JWT VC type metadata chain should succeed");

        normalized
            .validate(&json!({
              "vct": "https://sd_jwt_vc_metadata.example.com/example_credential_v3",
              "iss": "https://example.com/issuer",
              "nbf": 1683000000,
              "iat": 1683000000,
              "exp": 1883000000,
              "attestation_qualification": "EAA"
            }))
            .expect("all JSON schemas in chain should validate");
    }

    #[test]
    fn test_normalized_type_metadata_validate_error() {
        let normalized =
            NormalizedTypeMetadata::try_from_sorted_metadata(SortedTypeMetadata::example_with_extensions())
                .expect("normalizing SD-JWT VC type metadata chain should succeed");

        let error = normalized
            .validate(&json!({}))
            .expect_err("first JSON schemas in chain should fail validation");

        assert_matches!(
            error,
            TypeMetadataValidationError(vct, _)
                if vct == "https://sd_jwt_vc_metadata.example.com/example_credential_v3"
        )
    }

    fn claim_with_sd(sd: ClaimSelectiveDisclosureMetadata) -> ClaimMetadata {
        ClaimMetadata {
            path: vec![ClaimPath::SelectByKey("path".to_string())].try_into().unwrap(),
            display: vec![],
            sd,
            svg_id: None,
        }
    }

    fn test_claim_metadata_extend_claims_selective_disclosure_ok(
        extended_sd: ClaimSelectiveDisclosureMetadata,
        extending_sd: ClaimSelectiveDisclosureMetadata,
        expected_sd: ClaimSelectiveDisclosureMetadata,
    ) {
        let claims = ClaimMetadata::extend_claims(
            vec![claim_with_sd(extended_sd)],
            "test",
            vec![claim_with_sd(extending_sd)],
        )
        .expect("extending claims should succeed");

        assert_eq!(claims[0].sd, expected_sd);
    }

    #[rstest]
    fn test_claim_metadata_extend_claims_selective_disclosure_unchanged(
        #[values(
            ClaimSelectiveDisclosureMetadata::Always,
            ClaimSelectiveDisclosureMetadata::Allowed,
            ClaimSelectiveDisclosureMetadata::Never
        )]
        sd: ClaimSelectiveDisclosureMetadata,
    ) {
        test_claim_metadata_extend_claims_selective_disclosure_ok(sd, sd, sd);
    }

    #[rstest]
    fn test_claim_metadata_extend_claims_selective_disclosure_more_strict(
        #[values(ClaimSelectiveDisclosureMetadata::Always, ClaimSelectiveDisclosureMetadata::Never)]
        sd: ClaimSelectiveDisclosureMetadata,
    ) {
        test_claim_metadata_extend_claims_selective_disclosure_ok(ClaimSelectiveDisclosureMetadata::Allowed, sd, sd);
    }

    #[rstest]
    #[case(ClaimSelectiveDisclosureMetadata::Always, ClaimSelectiveDisclosureMetadata::Allowed)]
    #[case(ClaimSelectiveDisclosureMetadata::Never, ClaimSelectiveDisclosureMetadata::Allowed)]
    #[case(ClaimSelectiveDisclosureMetadata::Always, ClaimSelectiveDisclosureMetadata::Never)]
    #[case(ClaimSelectiveDisclosureMetadata::Never, ClaimSelectiveDisclosureMetadata::Always)]
    fn test_claim_metadata_extend_claims_error_inconsistent_selective_disclosure(
        #[case] extended_sd: ClaimSelectiveDisclosureMetadata,
        #[case] extending_sd: ClaimSelectiveDisclosureMetadata,
    ) {
        let error = ClaimMetadata::extend_claims(
            vec![claim_with_sd(extended_sd)],
            "inconsistent_sd",
            vec![claim_with_sd(extending_sd)],
        )
        .expect_err("extending claims should not succeed");

        let expected_path = VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("path".to_string())]).unwrap();
        assert_matches!(
            error,
            NormalizedTypeMetadataError::InconsistentSelectiveDisclosure(vct, path)
                if vct == "inconsistent_sd" && path == expected_path
        );
    }

    fn claim_with_single_key_path(path: String) -> ClaimMetadata {
        ClaimMetadata {
            path: vec![ClaimPath::SelectByKey(path)].try_into().unwrap(),
            display: vec![],
            sd: ClaimSelectiveDisclosureMetadata::default(),
            svg_id: None,
        }
    }

    #[test]
    fn test_claim_metadata_extend_claims_error_extension_missing_claims() {
        let error = ClaimMetadata::extend_claims(
            vec![
                claim_with_single_key_path("path1".to_string()),
                claim_with_single_key_path("path2".to_string()),
                claim_with_single_key_path("path3".to_string()),
            ],
            "missing_claims",
            vec![
                claim_with_single_key_path("path4".to_string()),
                claim_with_single_key_path("path2".to_string()),
            ],
        )
        .expect_err("extending claims should not succeed");

        let expected_paths = vec![
            VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("path1".to_string())]).unwrap(),
            VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("path3".to_string())]).unwrap(),
        ];
        assert_matches!(
            error,
            NormalizedTypeMetadataError::ExtensionMissingClaims(vct, paths)
                if vct == "missing_claims" && paths == expected_paths
        );
    }

    fn create_basic_unchecked_metadata() -> UncheckedTypeMetadata {
        UncheckedTypeMetadata {
            vct: "metadata".to_string(),
            name: None,
            description: None,
            extends: None,
            display: vec![DisplayMetadata {
                lang: "en".to_string(),
                name: "attestation".to_string(),
                description: None,
                summary: None,
                rendering: None,
            }],
            claims: vec![],
            schema: SchemaOption::Embedded {
                schema: Box::new(JsonSchema::example_with_claim_names(&[])),
            },
        }
    }

    fn normalized_type_metadata_error_from_unchecked_chain(
        chain: Vec<UncheckedTypeMetadata>,
    ) -> NormalizedTypeMetadataError {
        let chain = SortedTypeMetadata::new_mock(
            chain
                .into_iter()
                .map(|metadata| TypeMetadata::try_new(metadata).unwrap())
                .collect(),
        );

        NormalizedTypeMetadata::try_from_sorted_metadata(chain)
            .expect_err("normalizing SD-JWT VC type metadata chain should not succeed")
    }

    #[test]
    fn test_normalized_type_metadata_error_no_display_metadata() {
        let metadata = UncheckedTypeMetadata {
            display: vec![],
            ..create_basic_unchecked_metadata()
        };

        let error = normalized_type_metadata_error_from_unchecked_chain(vec![metadata]);

        assert_matches!(error, NormalizedTypeMetadataError::NoDisplayMetadata);
    }

    #[test]
    fn test_normalized_type_metadata_error_no_claim_display_metadata() {
        let metadata = UncheckedTypeMetadata {
            claims: vec![ClaimMetadata {
                path: vec![ClaimPath::SelectByKey("path".to_string())].try_into().unwrap(),
                display: vec![],
                sd: ClaimSelectiveDisclosureMetadata::Allowed,
                svg_id: None,
            }],
            ..create_basic_unchecked_metadata()
        };

        let error = normalized_type_metadata_error_from_unchecked_chain(vec![metadata]);

        let expected_path = VecNonEmpty::try_from(vec![ClaimPath::SelectByKey("path".to_string())]).unwrap();
        assert_matches!(error, NormalizedTypeMetadataError::NoClaimDisplayMetadata(path) if path == expected_path);
    }

    #[test]
    fn test_normalized_type_metadata_error_no_embedded_schema() {
        let metadata1 = UncheckedTypeMetadata {
            vct: "metadata_1".to_string(),
            ..create_basic_unchecked_metadata()
        };
        let metadata2 = UncheckedTypeMetadata {
            vct: "metadata_2".to_string(),
            schema: SchemaOption::Remote {
                schema_uri: "https://example.com/schema.json".parse().unwrap(),
                schema_uri_integrity: Integrity::from("").into(),
            },
            ..create_basic_unchecked_metadata()
        };
        let metadata3 = UncheckedTypeMetadata {
            vct: "metadata_3".to_string(),
            ..create_basic_unchecked_metadata()
        };

        let error = normalized_type_metadata_error_from_unchecked_chain(vec![metadata3, metadata2, metadata1]);

        assert_matches!(error, NormalizedTypeMetadataError::NoEmbeddedSchema(vct) if vct == "metadata_2");
    }

    #[test]
    fn test_normalized_type_metadata_error_missing_svg_ids() {
        // Create a metadata chain where the extension overwrites the `svg_id`,
        // but does not update the summary template accordingly.
        let claim = ClaimMetadata {
            path: vec![ClaimPath::SelectByKey("path".to_string())].try_into().unwrap(),
            display: vec![ClaimDisplayMetadata {
                lang: "en".to_string(),
                label: "claim".to_string(),
                description: None,
            }],
            sd: ClaimSelectiveDisclosureMetadata::Allowed,
            svg_id: None,
        };
        let metadata1 = UncheckedTypeMetadata {
            vct: "metadata_1".to_string(),
            display: vec![DisplayMetadata {
                lang: "en".to_string(),
                name: "attestation".to_string(),
                description: None,
                summary: Some("{{identifier}}".to_string()),
                rendering: None,
            }],
            claims: vec![ClaimMetadata {
                svg_id: Some(SvgId::try_new("identifier").unwrap()),
                ..claim.clone()
            }],
            ..create_basic_unchecked_metadata()
        };
        let metadata2 = UncheckedTypeMetadata {
            vct: "metadata_2".to_string(),
            display: vec![],
            claims: vec![ClaimMetadata {
                svg_id: Some(SvgId::try_new("identifier2").unwrap()),
                ..claim.clone()
            }],
            ..create_basic_unchecked_metadata()
        };

        let error = normalized_type_metadata_error_from_unchecked_chain(vec![metadata2, metadata1]);

        assert_matches!(error, NormalizedTypeMetadataError::MissingSvgIds(svg_ids) if svg_ids == vec!["identifier"]);
    }
}
