use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;

use crate::{
    engagement::{DeviceAuthenticationKeyed, SessionTranscript},
    errors::Result,
    holder::HolderError,
    identifiers::{AttributeIdentifier, AttributeIdentifierHolder},
    mdocs::DocType,
    utils::serialization::{self, CborSeq, TaggedBytes},
    ItemsRequest,
};

use super::{proposed_document::ProposedDocument, MdocDataSource};

/// This type represents the result of matching an iterator of `ItemsRequest`
/// instances against all locally stored document. This result is one of two options:
/// * `DisclosureRequestMatch::Candidates` means that all of the attributes in the request can be satisfied. For each
///   `DocType` in the request, a list of matching documents is provided in an `IndexMap`.
/// * `DisclosureRequestMatch::MissingAttributes` when at least one of the attributes requested is not present in any of
///   the stored documents.
///
/// Please note the following:
/// * The input iterator of `ItemsRequest`s could contain multiple `ItemsRequest` entries with the same `DocType`. The
///   matching result coalesces all attributes that are requested for a particular `DocType`, which will result in a
///   `DeviceResponse` with only one `Document` per `DocType`. This assumes that the verifier can match this response
///   against its original request.
/// * The order of the `IndexMap` provided with `DisclosureRequestMatch::Candidates` tries to match the order of the
///   request as best as possible. However, considering the previous point the order is not an exact match when the
///   request contains the same `DocType` multiple times.
/// * It is a known limitation that `DisclosureRequestMatch::MissingAttributes` only contains the missing attributes for
///   one of the `Mdoc`s for a particular `DocType`. Which one it chooses is undefined.
#[derive(Debug)]
pub enum DisclosureRequestMatch<I> {
    Candidates(IndexMap<DocType, Vec<ProposedDocument<I>>>),
    MissingAttributes(Vec<AttributeIdentifier>), /* TODO: Report on missing attributes per `Mdoc` candidate.
                                                  * (PVW-1392) */
}

impl<I> DisclosureRequestMatch<I> {
    pub async fn new<'a>(
        items_requests: impl IntoIterator<Item = &'a ItemsRequest> + Clone,
        mdoc_data_source: &impl MdocDataSource<MdocIdentifier = I>,
        session_transcript: &SessionTranscript,
    ) -> Result<DisclosureRequestMatch<I>> {
        // Make a `HashSet` of doc types from the `DeviceRequest` to account
        // for potential duplicate doc types in the request, then fetch them
        // from our data source.
        let doc_types = items_requests
            .clone()
            .into_iter()
            .map(|items_request| items_request.doc_type.as_str())
            .collect();

        let stored_mdocs = mdoc_data_source
            .mdoc_by_doc_types(&doc_types)
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?;

        // For each `doc_type`, calculate the set of `AttributeIdentifier`s that
        // are needed to satisfy the request. Note that a `doc_type` may occur more
        // than once in a `DeviceRequest`, so we combine all attributes and then split
        // them out by `doc_type`.
        let mut requested_attributes_by_doc_type = items_requests.attribute_identifiers().into_iter().fold(
            HashMap::<_, IndexSet<_>>::with_capacity(doc_types.len()),
            |mut requested_attributes, attribute_identifier| {
                // This unwrap is safe, as `doc_types` is derived from the same `DeviceRequest`.
                let doc_type = *doc_types.get(attribute_identifier.doc_type.as_str()).unwrap();
                requested_attributes
                    .entry(doc_type)
                    .or_default()
                    .insert(attribute_identifier);

                requested_attributes
            },
        );

        // Each `Vec<Mdoc>` that is returned from storage should contain `Mdoc`s
        // that have the same `doc_type`. Below, we iterate over all of these
        // `Vec`s and perform the following steps:
        //
        // * Filter out any empty `Vec<Mdoc>`.
        // * Get the `doc_type` from the first `Mdoc` entry.
        // * Remove the value for this `doc_type` from `requested_attributes_by_doc_type`.
        // * Do some sanity checks, as the request should actually contain this `doc_type` and any subsequent `Mdoc`s
        //   should have the same `doc_type`. This is part of the contract of `MdocDataSource` that is not enforceable.
        // * Calculate the challenge needed to create the `DeviceSigned` for this `doc_type` later on during actual
        //   disclosure.
        // * Convert all `Mdoc`s that satisfy the requirement to `ProposedDocument`, while collecting any missing
        //   attributes separately.
        // * Collect the candidates in a `IndexMap` per `doc_type`.
        //
        // Note that we consume the requested attributes from
        // `requested_attributes_by_doc_type` for the following reasons:
        //
        // * A `doc_type` should not occur more than once in the top-level
        //  `Vec` returned by `MdocDataSource`.
        // * After gathering all the candidates, any requested attributes that still remain in
        //   `requested_attributes_by_doc_type` are not satisfied, which means that all of them count as missing
        //   attributes.
        let mut all_missing_attributes = Vec::<Vec<AttributeIdentifier>>::new();

        let stored_mdocs = stored_mdocs
            .into_iter()
            .filter(|doc_type_mdocs| !doc_type_mdocs.is_empty())
            .collect::<Vec<_>>();

        let candidates_by_doc_type = stored_mdocs
            .into_iter()
            .map(|doc_type_stored_mdocs| {
                // First, remove the `IndexSet` of attributes that are required for this
                // `doc_type` from the global `HashSet`. If this cannot be found, then
                // `MdocDataSource` did not obey the contract as noted in the comment above.
                let first_doc_type = doc_type_stored_mdocs.first().unwrap().mdoc.doc_type.as_str();
                let (doc_type, requested_attributes) = requested_attributes_by_doc_type
                    .remove_entry(first_doc_type)
                    .expect("Received mdoc candidate with unexpected doc_type from storage");

                // Do another sanity check, all of the remaining `Mdoc`s
                // in the `Vec` should have the same `doc_type`.
                for stored_mdoc in &doc_type_stored_mdocs {
                    if stored_mdoc.mdoc.doc_type != doc_type {
                        panic!("Received mdoc candidate with inconsistent doc_type from storage");
                    }
                }

                // Calculate the `DeviceAuthentication` for this `doc_type` and turn it into bytes,
                // so that it can be used as a challenge when constructing `DeviceSigned` later on.
                let device_authentication = DeviceAuthenticationKeyed::new(doc_type, session_transcript);
                let device_signed_challenge =
                    serialization::cbor_serialize(&TaggedBytes(CborSeq(device_authentication)))?;

                // Get all the candidates and missing attributes from the provided `Mdoc`s.
                let (candidates, missing_attributes) =
                    ProposedDocument::candidates_and_missing_attributes_from_stored_mdocs(
                        doc_type_stored_mdocs,
                        &requested_attributes,
                        device_signed_challenge,
                    )?;

                // If we have multiple `Mdoc`s with missing attributes, just record the first one.
                // TODO: Report on missing attributes for multiple `Mdoc` candidates. (PVW-1392)
                if let Some(missing_attributes) = missing_attributes.into_iter().next() {
                    all_missing_attributes.push(missing_attributes);
                }

                Ok((doc_type.to_string(), candidates))
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        // If we cannot find a suitable candidate for any of the doc types
        // or one of the doc types is missing entirely, collect all of the
        // attributes that are missing and return this as the
        // `DisclosureRequestMatch::MissingAttributes` invariant.
        if candidates_by_doc_type.values().any(|candidates| candidates.is_empty())
            || !requested_attributes_by_doc_type.is_empty()
        {
            // Combine the missing attributes from the processed `Mdoc`s with
            // the requested attributes for any `doc_type` we did not see at all.
            let missing_attributes = all_missing_attributes
                .into_iter()
                .flatten()
                .chain(
                    // Get the `doc_type`s from the original request so that we
                    // can preserve the original order as much as possible.
                    items_requests
                        .into_iter()
                        .map(|items_request| items_request.doc_type.as_str())
                        .unique()
                        // Get all of the requested attributes that are still remaining from
                        // `requested_attributes_by_doc_type`, ignoring any `None` entries.
                        // Note that this removes the attributes from that `HashMap`, so that
                        // we can take ownership and avoid cloning the `AttributeIdentifier`s.
                        .flat_map(|doc_type| requested_attributes_by_doc_type.remove(doc_type))
                        .flatten(),
                )
                .collect();

            return Ok(DisclosureRequestMatch::MissingAttributes(missing_attributes));
        }

        // Each `doc_type` has at least one candidates, return these now.
        Ok(DisclosureRequestMatch::Candidates(candidates_by_doc_type))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use futures::future;
    use rstest::rstest;

    use crate::{
        holder::mock::MockMdocDataSource,
        iso::{
            mdocs::{Attributes, IssuerNameSpaces, IssuerSignedItem},
            unsigned::Entry,
        },
        server_keys::KeyPair,
        software_key_factory::SoftwareKeyFactory,
        test::{
            data::{addr_street, empty, pid_family_name, pid_full_name, pid_given_name},
            TestDocument, TestDocuments,
        },
    };

    use super::*;

    #[rstest]
    #[case(empty(), empty(), candidates(empty()))]
    #[case(pid_full_name(), pid_full_name(), candidates(pid_full_name()))]
    #[case(pid_given_name(), pid_given_name() + pid_given_name(), candidates(pid_given_name()))]
    #[case(pid_given_name() + pid_given_name(), pid_given_name(), candidates(pid_given_name() + pid_given_name()))]
    #[case(pid_full_name() + pid_given_name() + addr_street(), addr_street(), candidates(addr_street()))]
    #[case(pid_full_name() + pid_given_name() + addr_street(), pid_given_name(), candidates(pid_given_name() + pid_given_name()))]
    #[case(pid_full_name() + pid_given_name() + addr_street(), empty(), candidates(empty()))]
    #[case(empty(), pid_given_name(), missing_attributes(pid_given_name()))]
    #[case(
        empty(),
        pid_given_name() + addr_street(),
        missing_attributes(pid_given_name() + addr_street())
    )]
    #[case(pid_given_name(), pid_full_name(), missing_attributes(pid_family_name()))]
    #[case(pid_full_name(), addr_street(), missing_attributes(addr_street()))]
    #[tokio::test]
    async fn test_match_stored_documents(
        #[case] stored_documents: TestDocuments,
        #[case] requested_documents: TestDocuments,
        #[case] expected_match: ExpectedDisclosureRequestMatch,
    ) {
        use crate::DeviceRequest;

        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let key_factory = SoftwareKeyFactory::default();

        let mdoc_data_source = MockMdocDataSource::new(
            future::join_all(
                stored_documents
                    .into_iter()
                    .map(|document| document.sign(&ca, &key_factory, NonZeroU8::new(1).unwrap())),
            )
            .await,
        );

        let device_request = DeviceRequest::from(requested_documents);

        let session_transcript = SessionTranscript::new_mock();
        let match_result =
            DisclosureRequestMatch::new(device_request.items_requests(), &mdoc_data_source, &session_transcript)
                .await
                .expect("Could not match device request with stored documents");

        let match_result: ExpectedDisclosureRequestMatch = match_result.into();
        assert_eq!(match_result, expected_match);
    }

    #[derive(Debug, PartialEq)]
    enum ExpectedDisclosureRequestMatch {
        Candidates(TestDocuments),
        MissingAttributes(IndexSet<AttributeIdentifier>),
    }

    fn candidates(candidates: TestDocuments) -> ExpectedDisclosureRequestMatch {
        ExpectedDisclosureRequestMatch::Candidates(candidates)
    }
    fn missing_attributes(missing_attributes: TestDocuments) -> ExpectedDisclosureRequestMatch {
        ExpectedDisclosureRequestMatch::MissingAttributes(missing_attributes.attribute_identifiers())
    }

    impl<T> From<DisclosureRequestMatch<T>> for ExpectedDisclosureRequestMatch {
        fn from(value: DisclosureRequestMatch<T>) -> Self {
            match value {
                DisclosureRequestMatch::Candidates(candidates) => {
                    let candidates: Vec<TestDocument> = candidates
                        .into_iter()
                        .flat_map(|(_, namespaces)| namespaces)
                        .map(convert_proposed_document)
                        .collect();
                    Self::Candidates(candidates.into())
                }
                DisclosureRequestMatch::MissingAttributes(missing) => {
                    Self::MissingAttributes(missing.into_iter().collect())
                }
            }
        }
    }

    fn convert_proposed_document<I>(
        ProposedDocument {
            doc_type,
            issuer_signed,
            ..
        }: ProposedDocument<I>,
    ) -> TestDocument {
        let name_spaces = issuer_signed.name_spaces.expect("Expected namespaces");

        TestDocument {
            doc_type,
            namespaces: convert_namespaces(name_spaces),
        }
    }

    fn convert_namespaces(namespaces: IssuerNameSpaces) -> IndexMap<String, Vec<Entry>> {
        namespaces
            .into_inner()
            .into_iter()
            .map(|(namespace, attributes)| (namespace, convert_attributes(attributes)))
            .collect()
    }

    fn convert_attributes(attributes: Attributes) -> Vec<Entry> {
        attributes.into_inner().into_iter().map(convert_attribute).collect()
    }

    fn convert_attribute(attribute: TaggedBytes<IssuerSignedItem>) -> Entry {
        Entry {
            name: attribute.0.element_identifier,
            value: attribute.0.element_value,
        }
    }
}
