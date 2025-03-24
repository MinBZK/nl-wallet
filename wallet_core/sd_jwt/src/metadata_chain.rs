use std::collections::HashMap;
use std::collections::HashSet;

use derive_more::AsRef;
use derive_more::IntoIterator;
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

use wallet_common::vec_at_least::VecNonEmpty;

use crate::metadata::TypeMetadata;

pub const COSE_METADATA_HEADER_LABEL: &str = "vctm";
pub const COSE_METADATA_INTEGRITY_HEADER_LABEL: &str = "type_metadata_integrity";

#[derive(Debug, thiserror::Error)]
pub enum TypeMetadataChainError {
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("metadata for requested \"vct\" not found: {0}")]
    VctNotFound(String),

    #[error("circular chain detected, caused by \"vct\": {0}")]
    CircularChain(String),

    #[error("excess metadata documents detected: {}", .0.join(", "))]
    ExcessMetadata(Vec<String>),

    #[error("resource integrity did not validate: {0}")]
    ResourceIntegrity(#[from] ssri::Error),

    #[error("insecure resource integrity algorithm used: {0}")]
    IntegrityAlgorithmInsecure(Algorithm),
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

/// Represents a JSON-encoded chain of SD-JWT VC Type Metadata documents, which themselves serialize to a JSON
/// array representation of URL-safe base64 strings, as described for the `vctm` array in the specifications:
/// https://www.ietf.org/archive/id/draft-ietf-oauth-sd-jwt-vc-08.html#section-6.3.5
///
/// Note that when transported using CBOR, the base64 en/decoding is skipped, as that format supports binary data.
///
/// The order of these documents is from the lowest extension to the highest extended document. That means that each
/// subsequent document is expected to reference the next one in its `extends*` fields and that these fields should be
/// absent for the last document, which constitutes the root of the chain.
///
/// Actually reading the contents of these metadata documents and validating the consistency of the chain is handled by
/// both the [`UnverifiedTypeMetadataChain`] and [`TypeMetadataChain`] types, as external data is required as input for
/// these validations. This type just represents the network representation of the chain.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, AsRef, IntoIterator, Serialize, Deserialize)]
pub struct TypeMetadataDocuments(
    #[serde_as(as = "IfIsHumanReadable<Vec<Base64<UrlSafe, Unpadded>>, Vec<Bytes>>")] VecNonEmpty<Vec<u8>>,
);

impl TypeMetadataDocuments {
    /// Construct a chain from JSON-encoded documents. Does not perform validation.
    pub fn new(documents: VecNonEmpty<Vec<u8>>) -> Self {
        Self(documents)
    }

    pub fn into_inner(self) -> VecNonEmpty<Vec<u8>> {
        let Self(inner) = self;

        inner
    }

    /// Parse and verify the internal consistency of the chain of SD-JWT metadata documents, except for checking the
    /// resource integrity of the first document. As this is meant to be done when a preview of the attestation is
    /// received, this first document's resource integrity is not yet available. It will be received later as part of
    /// the actual attestation. This method consumes the [`TypeMetadataDocuments`] type and turns it into a
    /// [`UnverifiedTypeMetadataChain`] and takes the first `vct` field as input.
    ///
    /// Note that, as the specification does not clearly specify the order of the documents within their array
    /// representation, we do not make assumptions about it. This means that the received document may be in any order.
    pub fn into_unverified_metadata_chain(
        self,
        vct: &str,
    ) -> Result<UnverifiedTypeMetadataChain, TypeMetadataChainError> {
        let Self(source_documents) = self;

        // Start by deserializing all of the metadata documents from JSON and map them by index into `source_documents`.
        // This also automatically performs some internal consistency checks on each individual metadata document.
        let mut metadata_by_index: HashMap<_, _> = source_documents
            .iter()
            .enumerate()
            .map(|(index, json)| serde_json::from_slice::<TypeMetadata>(json).map(|metadata| (index, metadata)))
            .try_collect()?;

        // Construct a map of `vct` fields to indices into `source_documents`, which we will consume later. The extra
        // indirection through indices both helps appease the borrow checker and determine the `first_document_index`
        // value later on.
        let mut index_by_vct: HashMap<_, _> = metadata_by_index
            .iter()
            .map(|(index, metadata)| (metadata.as_ref().vct.as_str(), *index))
            .collect();

        // Prepare variables to collect data and iterator over the whole chain, starting at the first `vct`.
        let documents_count = source_documents.len().get();
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

            // ...and if this not the first document in the chain, check the resource integrity of its source JSON.
            if let Some(integrity) = integrity {
                let json = source_documents.as_ref().get(index).unwrap().as_slice();
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

            return Err(TypeMetadataChainError::ExcessMetadata(excess_vcst));
        }

        // Specifically remember the index of the starting document within the original JSON array. There is guaranteed
        // to be at least one index found, because there is always at least one source documents and there are no
        // unprocessed documents remaining.
        let first_document_index = *metadata_chain_indices.first().unwrap();

        // Finally collect an owned `Vec` of the metadata documents by consuming the indices, which are all guaranteed
        // to exist.
        let metadata_chain = metadata_chain_indices
            .into_iter()
            .map(|index| metadata_by_index.remove(&index).unwrap())
            .collect_vec()
            // Converting to a `VecNonEmpty` cannot fail, as the input is also `VecNonEmpty`.
            .try_into()
            .unwrap();

        Ok(UnverifiedTypeMetadataChain {
            first_document_index,
            source_documents,
            metadata_chain,
        })
    }
}

/// Represent an ordered chain of SD-JWT Type Metadata documents that are internally consistent. The order of these
/// documents is from the lowest extension to the highest extended document. This chain is unverified in that the
/// resource integrity of the first document has not been checked, which should be done before accepting and storing
/// the metadata.
///
/// This type also wraps the received array of JSON source documents, which may need to be stored once verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnverifiedTypeMetadataChain {
    first_document_index: usize,
    source_documents: VecNonEmpty<Vec<u8>>,
    metadata_chain: VecNonEmpty<TypeMetadata>,
}

impl UnverifiedTypeMetadataChain {
    // TODO (PVW-3824): Use normalized metadata across chain instead of simply the first one.
    pub fn as_metadata(&self) -> &TypeMetadata {
        self.metadata_chain.first()
    }

    // TODO (PVW-3824): Use normalized metadata across chain instead of simply the first one.
    pub fn into_metadata_and_source(self) -> (TypeMetadata, TypeMetadataDocuments) {
        (
            self.metadata_chain.into_first(),
            TypeMetadataDocuments(self.source_documents),
        )
    }

    /// Verify the resource integrity of the first document.
    pub fn verify(&self, first_metadata_integrity: Integrity) -> Result<(), TypeMetadataChainError> {
        check_resource_integrity(
            self.source_documents
                .as_ref()
                .get(self.first_document_index)
                .unwrap()
                .as_slice(),
            first_metadata_integrity,
        )?;

        Ok(())
    }

    /// Verify the resource integrity of the first document, which consumes the type and splits it off into a
    /// [`TypeMetadataChain`] and the original [`TypeMetadataDocuments`], which can be stored for later reference.
    pub fn into_metadata_chain_and_source(
        self,
        first_metadata_integrity: Integrity,
    ) -> Result<(TypeMetadataChain, TypeMetadataDocuments), TypeMetadataChainError> {
        self.verify(first_metadata_integrity)?;

        let chain = TypeMetadataChain(self.metadata_chain);
        let documents = TypeMetadataDocuments(self.source_documents);

        Ok((chain, documents))
    }
}

/// A fully verified ordered chain of SD-JWT Type Metadata documents. The order of these documents is from the lowest
/// extension to the highest extended document.
#[derive(Debug, Clone, PartialEq, Eq, IntoIterator)]
pub struct TypeMetadataChain(VecNonEmpty<TypeMetadata>);

impl TypeMetadataChain {
    // TODO (PVW-3824): Use normalized metadata across chain instead of simply the first one.
    pub fn as_metadata(&self) -> &TypeMetadata {
        let Self(chain) = self;

        chain.first()
    }

    // TODO (PVW-3824): Use normalized metadata across chain instead of simply the first one.
    pub fn into_metadata(self) -> TypeMetadata {
        let Self(chain) = self;

        chain.into_first()
    }
}

#[cfg(feature = "example_constructors")]
mod example_constructors {
    use ssri::Integrity;

    use wallet_common::vec_at_least::VecNonEmpty;

    use crate::metadata::MetadataExtends;
    use crate::metadata::TypeMetadata;

    use super::TypeMetadataDocuments;

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
    }
}
