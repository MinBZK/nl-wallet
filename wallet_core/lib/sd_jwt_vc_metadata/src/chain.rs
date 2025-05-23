use std::collections::HashMap;
use std::collections::HashSet;

use derive_more::AsRef;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::base64::UrlSafe;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use serde_with::Bytes;
use serde_with::IfIsHumanReadable;
use ssri::Algorithm;
use ssri::Integrity;
use ssri::IntegrityChecker;

use utils::vec_at_least::VecNonEmpty;

use crate::metadata::TypeMetadata;
use crate::normalized::NormalizedTypeMetadata;
use crate::normalized::NormalizedTypeMetadataError;

pub const SD_JWT_VC_TYPE_METADATA_KEY: &str = "vctm";

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataChainError {
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("metadata for requested \"vct\" not found: {0}")]
    VctNotFound(String),

    #[error("circular chain detected, caused by \"vct\": {0}")]
    CircularChain(String),

    #[error("excess metadata documents detected: {}", .0.join(", "))]
    ExcessDocuments(Vec<String>),

    #[error("resource integrity did not validate: {0}")]
    ResourceIntegrity(#[from] ssri::Error),

    #[error("insecure resource integrity algorithm used: {0}")]
    IntegrityAlgorithmInsecure(Algorithm),

    #[error("normalization of type metadata failed: {0}")]
    Normalization(#[from] NormalizedTypeMetadataError),
}

fn check_resource_integrity(json: &[u8], integrity: Integrity) -> Result<(), TypeMetadataChainError> {
    let mut checker = IntegrityChecker::new(integrity);
    checker.input(json);

    let algorithm = checker.result()?;
    if algorithm > Algorithm::Sha256 {
        return Err(TypeMetadataChainError::IntegrityAlgorithmInsecure(algorithm));
    }

    Ok(())
}

/// Represents a chain of decoded SD-JWT Type Metadata documents that is sorted from leaf to root.
#[derive(Debug, Clone, AsRef)]
pub(crate) struct SortedTypeMetadata(VecNonEmpty<TypeMetadata>);

impl SortedTypeMetadata {
    pub fn into_inner(self) -> VecNonEmpty<TypeMetadata> {
        let SortedTypeMetadata(chain) = self;

        chain
    }
}

/// Represents a JSON-encoded chain of SD-JWT VC Type Metadata documents, which themselves serialize to a JSON
/// array representation of URL-safe base64 strings, as described for the `vctm` array in the specifications:
/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#section-6.3.5
///
/// Note that when transported using CBOR, the base64 en/decoding is skipped, as that format supports binary data.
///
/// The order of these is from the leaf extension document to the root extended document. That means that each
/// subsequent document is expected to reference the next one in its `extends*` fields and that these fields should be
/// absent for the last document, which constitutes the root of the chain.
///
/// Checking the internal consistency of this chain and normalizing the metadata can be done by transforming it in into
/// a [`SortedTypeMetadataDocuments`] type, which then can be turned into [`VerifiedTypeMetadataDocuments`] by verifying
/// its integrity.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Serialize, Deserialize)]
pub struct TypeMetadataDocuments(
    #[serde_as(as = "IfIsHumanReadable<Vec<Base64<UrlSafe, Unpadded>>, Vec<Bytes>>")] VecNonEmpty<Vec<u8>>,
);

impl TypeMetadataDocuments {
    /// Construct a chain from JSON-encoded documents. Does not perform validation.
    pub fn new(documents: VecNonEmpty<Vec<u8>>) -> Self {
        Self(documents)
    }

    /// Parse and verify the internal consistency of the chain of SD-JWT metadata documents, except for checking the
    /// resource integrity of the leaf document. As this is meant to be done before an actual attestation is received,
    /// the leaf document's resource integrity will not yet available. It will be received later as part of the actual
    /// attestation. This method produces a [`NormalizedTypeMetadata`] and a [`SortedTypeMetadataDocuments`] value while
    /// taking the leaf `vct` field as input. The latter value can be used later to verify the leaf document's resource
    /// integrity.
    ///
    /// Note that, as the specification does not clearly specify the order of the documents within their array
    /// representation, we do not make assumptions about it. This means that the received documents may be in any order.
    pub fn into_normalized(
        self,
        vct: &str,
    ) -> Result<(NormalizedTypeMetadata, SortedTypeMetadataDocuments), TypeMetadataChainError> {
        let Self(documents) = self;

        // Start by deserializing all of the metadata documents from JSON and map them by index into `source_documents`.
        // This also automatically performs some internal consistency checks on each individual metadata document.
        let mut metadata_by_index: HashMap<_, _> = documents
            .iter()
            .enumerate()
            .map(|(index, json)| serde_json::from_slice::<TypeMetadata>(json).map(|metadata| (index, metadata)))
            .try_collect()?;

        // Construct a map of `vct` fields to indices into `source_documents`, which we will consume later. The extra
        // indirection through indices both helps appease the borrow checker.
        let mut index_by_vct: HashMap<_, _> = metadata_by_index
            .iter()
            .map(|(index, metadata)| (metadata.as_ref().vct.as_str(), *index))
            .collect();

        // Prepare variables to collect data and iterator over the whole chain, starting at the leaf `vct`.
        let documents_count = documents.len().get();
        let mut metadata_chain_indices = Vec::with_capacity(documents_count);
        let mut seen_vcts = HashSet::with_capacity(documents_count);
        let mut next_extends = Some((vct, None));

        while let Some((vct, integrity)) = next_extends {
            // If the `vct` field cannot be found among the documents, it either means it was not present or we have
            // already processed it and this chain is a circle. The specification explicitly mandates that this should
            // be detected and prevented in section 10.3.
            let index = index_by_vct.remove(vct).ok_or_else(|| {
                if seen_vcts.contains(vct) {
                    TypeMetadataChainError::CircularChain(vct.to_string())
                } else {
                    TypeMetadataChainError::VctNotFound(vct.to_string())
                }
            })?;

            // Now that we know the index for this `vct`, get a reference to the deserialized metadata document from an
            // index that is guaranteed to exist...
            let metadata = metadata_by_index.get(&index).unwrap();

            // ...and if this not the leaf document in the chain, check the resource integrity of its source JSON.
            if let Some(integrity) = integrity {
                let json = documents.as_ref().get(index).unwrap().as_slice();
                check_resource_integrity(json, integrity)?;
            }

            // Remember the order of the documents within the chain and which `vct`s we have seen, then prepare the next
            // iteration of the loop, if we have not yet reached the end of the chain.
            metadata_chain_indices.push(index);
            seen_vcts.insert(metadata.as_ref().vct.as_str());

            next_extends = metadata.as_ref().extends.as_ref().map(|extends| {
                (
                    extends.extends.as_str(),
                    Some(extends.extends_integrity.as_ref().clone()),
                )
            });
        }

        // Be extra strict by checking that the set of `vct`s that have not been processed is now 0, as they should have
        // all been consumed by walking the chain.
        if !index_by_vct.is_empty() {
            // Appease the borrow checker by creating an intermediate `Vec`.
            let excess_indices = index_by_vct.into_values().collect_vec();
            let excess_vcst = excess_indices
                .into_iter()
                // These indices are guaranteed to exist.
                .map(|index| metadata_by_index.remove(&index).unwrap().into_inner().vct)
                .collect();

            return Err(TypeMetadataChainError::ExcessDocuments(excess_vcst));
        }

        // Collect an owned `Vec` of the metadata documents by consuming the indices, which are all guaranteed to exist.
        let metadata_chain = metadata_chain_indices
            .iter()
            .map(|index| metadata_by_index.remove(index).unwrap())
            .collect_vec()
            // Converting to a `VecNonEmpty` cannot fail, as the input is also `VecNonEmpty`.
            .try_into()
            .unwrap();

        // Normalize the chain of type metadata by combining the individual entries into
        // one type and move the documents to a `SortedTypeMetadataDocuments` type.
        let normalized = NormalizedTypeMetadata::try_from_sorted_metadata(SortedTypeMetadata(metadata_chain))?;

        let sorted_documents = documents
            .into_iter()
            .zip(metadata_chain_indices)
            .sorted_by_key(|(_, index)| *index)
            .map(|(json, _)| json)
            .collect_vec()
            .try_into()
            .unwrap();
        let sorted = SortedTypeMetadataDocuments(sorted_documents);

        Ok((normalized, sorted))
    }
}

/// Contains a sorted JSON-encoded chain of SD-JWT VC Type Metadata documents. The order of these is from the leaf
/// extension document to the root extended document. Note that the internal resource integrity of the leaf document has
/// not been validated yet. This can be done using the integrity digest from a received attestation.
#[derive(Debug, Clone, PartialEq, Eq, AsRef)]
pub struct SortedTypeMetadataDocuments(VecNonEmpty<Vec<u8>>);

impl SortedTypeMetadataDocuments {
    /// Verify the resource integrity of the leaf document and return a [`VerifiedTypeMetadataDocuments`] type.
    pub fn into_verified(self, integrity: Integrity) -> Result<VerifiedTypeMetadataDocuments, TypeMetadataChainError> {
        let Self(documents) = self;

        check_resource_integrity(documents.first(), integrity)?;

        Ok(VerifiedTypeMetadataDocuments(documents))
    }
}

impl PartialEq<SortedTypeMetadataDocuments> for TypeMetadataDocuments {
    fn eq(&self, other: &SortedTypeMetadataDocuments) -> bool {
        // A `TypeMetadataDocuments` is equal to a `SortedTypeMetadataDocuments` if the set of JSON documents they hold
        // is exactly the same. This holds, as a `SortedTypeMetadataDocuments` can only be constructed when there are no
        // excess JSON metadata documents within the set.
        HashSet::<&[u8]>::from_iter(self.as_ref().iter().map(Vec::as_slice))
            == HashSet::from_iter(other.as_ref().iter().map(Vec::as_slice))
    }
}

impl From<SortedTypeMetadataDocuments> for TypeMetadataDocuments {
    fn from(value: SortedTypeMetadataDocuments) -> Self {
        let SortedTypeMetadataDocuments(documents) = value;

        TypeMetadataDocuments(documents)
    }
}

impl PartialEq<TypeMetadataDocuments> for SortedTypeMetadataDocuments {
    fn eq(&self, other: &TypeMetadataDocuments) -> bool {
        other == self
    }
}

/// Contains a sorted JSON-encoded chain of SD-JWT VC Type Metadata documents that has been fully verified.
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Serialize)]
pub struct VerifiedTypeMetadataDocuments(VecNonEmpty<Vec<u8>>);

impl From<VerifiedTypeMetadataDocuments> for TypeMetadataDocuments {
    fn from(value: VerifiedTypeMetadataDocuments) -> Self {
        let VerifiedTypeMetadataDocuments(documents) = value;

        TypeMetadataDocuments(documents)
    }
}

#[cfg(any(test, feature = "example_constructors"))]
mod example_constructors {
    use ssri::Integrity;

    use utils::vec_at_least::VecNonEmpty;

    use crate::examples::ADDRESS_METADATA_BYTES;
    use crate::examples::DEGREE_METADATA_BYTES;
    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::EXAMPLE_V2_METADATA_BYTES;
    use crate::examples::EXAMPLE_V3_METADATA_BYTES;
    use crate::examples::PID_METADATA_BYTES;
    use crate::metadata::MetadataExtends;
    use crate::metadata::TypeMetadata;

    use super::TypeMetadataDocuments;
    use super::VerifiedTypeMetadataDocuments;

    impl TypeMetadataDocuments {
        /// Construct a [`TypeMetadataDocuments`] chain for transmission by JSON encoding an ordered sequence of
        /// [`TypeMetadata`] values. Note that the `extends*` fields of these types will be overwritten in order to
        /// construct this chain and have the resource integrity values match the encoded JSON.
        pub fn new_metadata_chain(
            metadata: VecNonEmpty<TypeMetadata>,
        ) -> Result<(String, Integrity, Self), serde_json::Error> {
            let mut next_extends: Option<(String, Integrity)> = None;

            let documents = metadata
                .into_iter()
                .rev()
                .map(|metadata| {
                    let mut unchecked_metadata = metadata.into_inner();
                    if let Some((extends, extends_integrity)) = next_extends.take() {
                        unchecked_metadata.extends = Some(MetadataExtends {
                            extends,
                            extends_integrity: extends_integrity.into(),
                        });
                    }

                    let json = serde_json::to_vec(&unchecked_metadata)?;

                    next_extends.replace((unchecked_metadata.vct, Integrity::from(&json)));

                    Ok(json)
                })
                .rev()
                .collect::<Result<Vec<_>, serde_json::Error>>()?
                .try_into()
                .unwrap();

            let (vct, integrity) = next_extends.take().unwrap();

            Ok((vct, integrity, Self(documents)))
        }

        pub fn from_single_example(example_metadata: TypeMetadata) -> (String, Integrity, Self) {
            Self::new_metadata_chain(vec![example_metadata].try_into().unwrap()).unwrap()
        }

        pub fn example() -> (Integrity, Self) {
            (
                Integrity::from(EXAMPLE_METADATA_BYTES),
                Self::new(vec![EXAMPLE_METADATA_BYTES.to_vec()].try_into().unwrap()),
            )
        }

        pub fn pid_example() -> (Integrity, Self) {
            (
                Integrity::from(PID_METADATA_BYTES),
                Self::new(vec![PID_METADATA_BYTES.to_vec()].try_into().unwrap()),
            )
        }

        pub fn address_example() -> (Integrity, Self) {
            (
                Integrity::from(ADDRESS_METADATA_BYTES),
                Self::new(vec![ADDRESS_METADATA_BYTES.to_vec()].try_into().unwrap()),
            )
        }

        pub fn degree_example() -> (Integrity, Self) {
            (
                Integrity::from(DEGREE_METADATA_BYTES),
                Self::new(vec![DEGREE_METADATA_BYTES.to_vec()].try_into().unwrap()),
            )
        }

        pub fn example_with_extensions() -> (Integrity, Self) {
            (
                Integrity::from(EXAMPLE_V3_METADATA_BYTES),
                Self::new(
                    vec![
                        EXAMPLE_V3_METADATA_BYTES.to_vec(),
                        EXAMPLE_V2_METADATA_BYTES.to_vec(),
                        EXAMPLE_METADATA_BYTES.to_vec(),
                    ]
                    .try_into()
                    .unwrap(),
                ),
            )
        }
    }

    impl VerifiedTypeMetadataDocuments {
        pub fn example() -> Self {
            Self(vec![EXAMPLE_METADATA_BYTES.to_vec()].try_into().unwrap())
        }

        pub fn pid_example() -> Self {
            Self(vec![PID_METADATA_BYTES.to_vec()].try_into().unwrap())
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use assert_matches::assert_matches;
    use itertools::Itertools;
    use rstest::rstest;
    use serde_json::json;
    use ssri::Algorithm;
    use ssri::Integrity;
    use ssri::IntegrityOpts;

    use crate::examples::EXAMPLE_METADATA_BYTES;
    use crate::examples::EXAMPLE_V2_METADATA_BYTES;
    use crate::examples::EXAMPLE_V3_METADATA_BYTES;
    use crate::examples::PID_METADATA_BYTES;
    use crate::metadata::MetadataExtends;
    use crate::metadata::TypeMetadata;
    use crate::metadata::UncheckedTypeMetadata;

    use super::SortedTypeMetadata;
    use super::TypeMetadataChainError;
    use super::TypeMetadataDocuments;

    impl SortedTypeMetadata {
        pub fn new_mock(chain: Vec<TypeMetadata>) -> Self {
            Self(chain.try_into().unwrap())
        }

        pub fn example_with_extensions() -> Self {
            let chain = vec![
                TypeMetadata::example_v3(),
                TypeMetadata::example_v2(),
                TypeMetadata::example(),
            ]
            .try_into()
            .unwrap();

            Self(chain)
        }
    }

    fn reversed_example_with_extension() -> (Integrity, TypeMetadataDocuments) {
        let (integrity, source_documents) = TypeMetadataDocuments::example_with_extensions();
        let TypeMetadataDocuments(documents_vec) = source_documents;
        let source_documents =
            TypeMetadataDocuments::new(documents_vec.into_iter().rev().collect_vec().try_into().unwrap());

        (integrity, source_documents)
    }

    #[rstest]
    #[case(
        "https://sd_jwt_vc_metadata.example.com/example_credential",
        TypeMetadataDocuments::example()
    )]
    #[case("urn:eudi:pid:nl:1", TypeMetadataDocuments::pid_example())]
    #[case("urn:eudi:pid-address:nl:1", TypeMetadataDocuments::address_example())]
    #[case("com.example.degree", TypeMetadataDocuments::degree_example())]
    #[case(
        "https://sd_jwt_vc_metadata.example.com/example_credential_v3",
        TypeMetadataDocuments::example_with_extensions()
    )]
    #[case(
        "https://sd_jwt_vc_metadata.example.com/example_credential_v3",
        reversed_example_with_extension()
    )]
    fn test_type_metadata_documents(
        #[case] vct: &str,
        #[case] (integrity, source_documents): (Integrity, TypeMetadataDocuments),
    ) {
        let (normalized, sorted) = source_documents
            .clone()
            .into_normalized(vct)
            .expect("parsing metadata document chain should succeed");

        assert_eq!(normalized.vct(), vct);
        assert_eq!(normalized.vct_count(), source_documents.as_ref().len());
        assert_eq!(
            sorted.as_ref().iter().collect::<HashSet<_>>(),
            source_documents.as_ref().iter().collect::<HashSet<_>>()
        );
        assert_eq!(
            serde_json::from_slice::<UncheckedTypeMetadata>(sorted.as_ref().first())
                .unwrap()
                .vct,
            vct
        );

        let verified = sorted
            .clone()
            .into_verified(integrity)
            .expect("veryfing leaf metadata document integrity should succeed");

        assert_eq!(verified.as_ref(), sorted.as_ref());
    }

    #[test]
    fn test_type_metadata_documents_error_json() {
        let document = serde_json::to_vec(&json!({
            "vct": "abc"
        }))
        .unwrap();
        let documents = TypeMetadataDocuments::new(vec![document].try_into().unwrap());

        let error = documents
            .into_normalized("abc")
            .expect_err("parsing metadata document chain should not succeed");

        assert_matches!(error, TypeMetadataChainError::Json(_));
    }

    #[test]
    fn test_type_metadata_documents_error_vct_not_found() {
        let (_, documents) = TypeMetadataDocuments::example_with_extensions();

        let error = documents
            .into_normalized("wrong_vct")
            .expect_err("parsing metadata document chain should not succeed");

        assert_matches!(error, TypeMetadataChainError::VctNotFound(vct) if vct == "wrong_vct");
    }

    #[test]
    fn test_type_metadata_documents_error_circular_chain() {
        let example_extension_document = EXAMPLE_V2_METADATA_BYTES.to_vec();

        let mut example_metadata = TypeMetadata::example().into_inner();
        example_metadata.extends = Some(MetadataExtends {
            extends: "https://sd_jwt_vc_metadata.example.com/example_credential".to_string(),
            extends_integrity: Integrity::from(&example_extension_document).into(),
        });
        let example_metadata = TypeMetadata::try_new(example_metadata).unwrap();

        let (vct, _, documents) = TypeMetadataDocuments::new_metadata_chain(
            vec![TypeMetadata::example_v2(), example_metadata].try_into().unwrap(),
        )
        .unwrap();

        let error = documents
            .into_normalized(&vct)
            .expect_err("parsing metadata document chain should not succeed");

        assert_matches!(
            error,
            TypeMetadataChainError::CircularChain(vct)
                if vct == "https://sd_jwt_vc_metadata.example.com/example_credential"
        );
    }

    #[test]
    fn test_type_metadata_documents_error_excess_documents() {
        let (_, documents) = TypeMetadataDocuments::example_with_extensions();
        let TypeMetadataDocuments(documents_vec) = documents;
        let mut json_documents = documents_vec.into_inner();
        json_documents.push(PID_METADATA_BYTES.to_vec());
        let documents = TypeMetadataDocuments::new(json_documents.try_into().unwrap());

        let error = documents
            .into_normalized("https://sd_jwt_vc_metadata.example.com/example_credential_v3")
            .expect_err("parsing metadata document chain should not succeed");

        assert_matches!(error, TypeMetadataChainError::ExcessDocuments(vcts) if vcts == vec!["urn:eudi:pid:nl:1"]);
    }

    fn test_type_metadata_documents_incorrect_extended_resource_integrity(
        integrity: Integrity,
    ) -> TypeMetadataChainError {
        let mut extension_metadata = TypeMetadata::example_v2().into_inner();
        extension_metadata.extends.as_mut().unwrap().extends_integrity = integrity.into();
        let extension_metadata = TypeMetadata::try_new(extension_metadata).unwrap();

        let documents = TypeMetadataDocuments::new(
            vec![
                serde_json::to_vec(&extension_metadata).unwrap(),
                EXAMPLE_METADATA_BYTES.to_vec(),
            ]
            .try_into()
            .unwrap(),
        );

        documents
            .into_normalized("https://sd_jwt_vc_metadata.example.com/example_credential_v2")
            .expect_err("parsing metadata document chain should not succeed")
    }

    #[test]
    fn test_type_metadata_documents_error_resource_integrity() {
        let error = test_type_metadata_documents_incorrect_extended_resource_integrity(Integrity::from("wrong_data"));

        assert_matches!(error, TypeMetadataChainError::ResourceIntegrity(_));
    }

    #[test]
    fn test_type_metadata_documents_error_integrity_algorithm_insecure() {
        let integrity = IntegrityOpts::new()
            .algorithm(Algorithm::Sha1)
            .chain(EXAMPLE_METADATA_BYTES)
            .result();
        let error = test_type_metadata_documents_incorrect_extended_resource_integrity(integrity);

        assert_matches!(error, TypeMetadataChainError::IntegrityAlgorithmInsecure(_));
    }

    #[test]
    fn test_unverified_type_metadata_chain_error_resource_integrity() {
        let integrity = IntegrityOpts::new()
            .algorithm(Algorithm::Sha1)
            .chain(EXAMPLE_V3_METADATA_BYTES)
            .result();
        let (_, documents) = TypeMetadataDocuments::example_with_extensions();
        let (_, leaf_document) = documents
            .into_normalized("https://sd_jwt_vc_metadata.example.com/example_credential_v3")
            .expect("parsing metadata document chain should succeed");
        let error = leaf_document
            .into_verified(integrity)
            .expect_err("veryfing leaf metadata document integrity should not succeed");

        assert_matches!(error, TypeMetadataChainError::IntegrityAlgorithmInsecure(_));
    }
}
