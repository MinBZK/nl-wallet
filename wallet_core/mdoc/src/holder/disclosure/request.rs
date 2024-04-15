use chrono::{DateTime, Utc};

use indexmap::{IndexMap, IndexSet};
use wallet_common::generator::Generator;
use webpki::TrustAnchor;

use crate::{
    device_retrieval::{DeviceRequest, DocRequest, ReaderAuthenticationKeyed},
    engagement::{DeviceAuthenticationKeyed, SessionTranscript},
    errors::Result,
    holder::HolderError,
    identifiers::{AttributeIdentifier, AttributeIdentifierHolder},
    mdocs::DocType,
    utils::{
        cose::ClonePayload,
        reader_auth::ReaderRegistration,
        serialization::{self, CborSeq, TaggedBytes},
        x509::{Certificate, CertificateType, CertificateUsage},
    },
};

use super::{proposed_document::ProposedDocument, MdocDataSource};

/// This type represents the result of matching a `DeviceRequest` against
/// all locally stored document. This result is one of two options:
/// * `DeviceRequestMatch::Candidates` means that all of the attributes
///   in the request can be satisfied. For each `DocType` in the request,
///   a list of matching documents is provided in an `IndexMap`.
/// * `DeviceRequestMatch::MissingAttributes` when at least one of the
///   attributes requested is not present in any of the stored documents.
///
/// Please note the following:
/// * A `DeviceRequest` may contain multiple `ItemsRequest` entries with
///   the same `DocType`. The matching result coalesces all attributes
///   that are requested for a particular `DocType`, which will result in
///   a `DeviceResponse` with only one `Document` per `DocType`. This
///   assumes that the verifier can match this response against its original
///   request.
/// * The order of the `IndexMap` provided with `DeviceRequestMatch::Candidates`
///   tries to match the order of the request as best as possible. However,
///   considering the previous point the order is not an exact match when the
///   request contains the same `DocType` multiple times.
/// * It is a known limitation that `DeviceRequestMatch::MissingAttributes`
///   only contains the missing attributes for one of the `Mdoc`s for a
///   particular `DocType`. Which one it chooses is undefined.
#[derive(Debug)]
pub(super) enum DeviceRequestMatch<I> {
    Candidates(IndexMap<DocType, Vec<ProposedDocument<I>>>),
    MissingAttributes(Vec<AttributeIdentifier>), // TODO: Report on missing attributes per `Mdoc` candidate. (PVW-1392)
}

impl DeviceRequest {
    /// Returns `true` if this request has any attributes at all.
    pub fn has_attributes(&self) -> bool {
        self.doc_requests
            .iter()
            .flat_map(|doc_request| doc_request.items_request.0.name_spaces.values())
            .any(|name_space| !name_space.is_empty())
    }

    /// Verify reader authentication, if present.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the
    /// the DocRequests to be signed by distinct readers. TODO maybe support this (PVW-2368).
    /// For now, this function requires either none of the DocRequests to be signed, or all of them
    /// by the same reader.
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<(Certificate, ReaderRegistration)>> {
        // If there are no doc requests or none of them have reader authentication, return `None`.
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }

        // Otherwise, all of the doc requests need reader authentication.
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        // Verify all `DocRequest` entries and make sure the resulting certificates are all exactly equal.
        let certificate = self
            .doc_requests
            .iter()
            .try_fold(None, {
                |result_cert, doc_request| -> Result<_> {
                    // This `.unwrap()` is safe, because `.verify()` will only return `None`
                    // if `reader_auth` is absent, the presence of which we checked above.
                    let doc_request_cert = doc_request.verify(session_transcript, time, trust_anchors)?.unwrap();

                    // If there is a certificate from a previous iteration, compare our certificate to that.
                    if let Some(result_cert) = result_cert {
                        if doc_request_cert != result_cert {
                            return Err(HolderError::ReaderAuthsInconsistent.into());
                        }
                    }

                    Ok(doc_request_cert.into())
                }
            })?
            .unwrap(); // This `.unwrap()` is safe for the same reason stated above.

        // Extract `ReaderRegistration` from the one certificate.
        let reader_registration = match CertificateType::from_certificate(&certificate).map_err(HolderError::from)? {
            CertificateType::ReaderAuth(Some(reader_registration)) => *reader_registration,
            _ => return Err(HolderError::NoReaderRegistration(certificate).into()),
        };

        // Verify that the requested attributes are included in the reader authentication.
        self.verify_requested_attributes(&reader_registration)
            .map_err(HolderError::from)?;

        Ok((certificate, reader_registration).into())
    }

    pub(super) async fn match_stored_documents<S, I>(
        &self,
        mdoc_data_source: &S,
        session_transcript: &SessionTranscript,
    ) -> Result<DeviceRequestMatch<I>>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        // Make a `HashSet` of doc types from the `DeviceRequest` to account
        // for potential duplicate doc types in the request, then fetch them
        // from our data source.
        let doc_types = self
            .doc_requests
            .iter()
            .map(|doc_request| doc_request.items_request.0.doc_type.as_str())
            .collect();

        let stored_mdocs = mdoc_data_source
            .mdoc_by_doc_types(&doc_types)
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?;

        // For each `doc_type`, calculate the set of `AttributeIdentifier`s that
        // are needed to satisfy the request. Note that a `doc_type` may occur more
        // than once in a `DeviceRequest`, so we combine all attributes and then split
        // them out by `doc_type`.
        let mut requested_attributes_by_doc_type = self.attribute_identifiers().into_iter().fold(
            IndexMap::<_, IndexSet<_>>::with_capacity(doc_types.len()),
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
        // * Do some sanity checks, as the request should actually contain this `doc_type`
        //   and any subsequent `Mdoc`s should have the same `doc_type`. This is part of
        //   the contract of `MdocDataSource` that is not enforceable.
        // * Calculate the challenge needed to create the `DeviceSigned` for this
        //   `doc_type` later on during actual disclosure.
        // * Convert all `Mdoc`s that satisfy the requirement to `ProposedDocument`,
        //   while collecting any missing attributes separately.
        // * Collect the candidates in a `IndexMap` per `doc_type`.
        //
        // Note that we consume the requested attributes from
        // `requested_attributes_by_doc_type` for the following reasons:
        //
        // * A `doc_type` should not occur more than once in the top-level
        //  `Vec` returned by `MdocDataSource`.
        // * After gathering all the candidates, any requested attributes that
        //   still remain in `requested_attributes_by_doc_type` are not satisfied,
        //   which means that all of them count as missing attributes.
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
        // `DeviceRequestMatch::MissingAttributes` invariant.
        if candidates_by_doc_type.values().any(|candidates| candidates.is_empty())
            || !requested_attributes_by_doc_type.is_empty()
        {
            // Combine the missing attributes from the processed `Mdoc`s with
            // the requested attributes for any `doc_type` we did not see at all.
            let missing_attributes = all_missing_attributes
                .into_iter()
                .flatten()
                .chain(requested_attributes_by_doc_type.into_values().flatten())
                .collect();

            return Ok(DeviceRequestMatch::MissingAttributes(missing_attributes));
        }

        // Each `doc_type` has at least one candidates, return these now.
        Ok(DeviceRequestMatch::Candidates(candidates_by_doc_type))
    }
}

impl DocRequest {
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<Certificate>> {
        // If reader authentication is present, verify it and return the certificate.
        self.reader_auth
            .as_ref()
            .map(|reader_auth| {
                // Reconstruct the reader authentication bytes for this `DocRequest`,
                // based on the item requests and session transcript.
                let reader_auth_payload = ReaderAuthenticationKeyed::new(session_transcript, &self.items_request);
                let reader_auth_payload = TaggedBytes(CborSeq(reader_auth_payload));

                // Perform verification and return the `Certificate`.
                let cose = reader_auth.clone_with_payload(serialization::cbor_serialize(&reader_auth_payload)?);
                cose.verify_against_trust_anchors(CertificateUsage::ReaderAuth, time, trust_anchors)?;
                let cert = cose.signing_cert()?;

                Ok(cert)
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use assert_matches::assert_matches;

    use rstest::rstest;
    use wallet_common::{generator::TimeGenerator, trust_anchor::DerTrustAnchor};

    use crate::{
        errors::Error,
        iso::device_retrieval::DeviceRequestVersion,
        server_keys::KeyPair,
        software_key_factory::SoftwareKeyFactory,
        test::{
            data::{addr_street, empty, pid_family_name, pid_full_name, pid_given_name},
            TestDocument, TestDocuments,
        },
        unsigned::Entry,
        Attributes, IssuerSignedItem,
    };

    use super::{super::test::*, *};

    #[tokio::test]
    async fn test_device_request_verify() {
        // Create two certificates and private keys.
        let ca = KeyPair::generate_reader_mock_ca().unwrap();
        let der_trust_anchors = [DerTrustAnchor::from_der(ca.certificate().as_bytes().to_vec()).unwrap()];
        let reader_registration = ReaderRegistration::new_mock();
        let private_key1 = ca.generate_reader_mock(reader_registration.clone().into()).unwrap();
        let private_key2 = ca.generate_reader_mock(reader_registration.clone().into()).unwrap();

        let session_transcript = create_basic_session_transcript();

        // Create an empty `ItemsRequest` and generate `DeviceRequest` with two `DocRequest`s
        // from it, each signed with the same certificate.
        let items_request = emtpy_items_request();

        let device_request = DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests: vec![
                create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
                create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
            ],
        };

        // Verifying this `DeviceRequest` should succeed and return the `ReaderRegistration`.
        let trust_anchors = der_trust_anchors
            .iter()
            .map(|anchor| (&anchor.owned_trust_anchor).into())
            .collect::<Vec<_>>();

        let verified_reader_registration = device_request
            .verify(&session_transcript, &TimeGenerator, &trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert_eq!(
            verified_reader_registration,
            Some((private_key1.certificate().clone(), reader_registration))
        );

        // Verifying a `DeviceRequest` that has no reader auth at all should succeed and return `None`.
        let device_request = DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests: vec![
                DocRequest {
                    items_request: items_request.clone().into(),
                    reader_auth: None,
                },
                DocRequest {
                    items_request: items_request.clone().into(),
                    reader_auth: None,
                },
            ],
        };

        let no_reader_registration = device_request
            .verify(&session_transcript, &TimeGenerator, &trust_anchors)
            .expect("Could not verify DeviceRequest");

        assert!(no_reader_registration.is_none());

        // Generate `DeviceRequest` with two `DocRequest`s, each signed
        // with a different key and including a different certificate.
        let device_request = DeviceRequest {
            version: DeviceRequestVersion::V1_0,
            doc_requests: vec![
                create_doc_request(items_request.clone(), &session_transcript, &private_key1).await,
                create_doc_request(items_request, &session_transcript, &private_key2).await,
            ],
        };

        // Verifying this `DeviceRequest` should result in a `HolderError::ReaderAuthsInconsistent` error.
        let error = device_request
            .verify(&session_transcript, &TimeGenerator, &trust_anchors)
            .expect_err("Verifying DeviceRequest should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::ReaderAuthsInconsistent));
    }

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
        #[case] expected_match: ExpectedDeviceRequestMatch,
    ) {
        let ca = KeyPair::generate_issuer_mock_ca().unwrap();
        let key_factory = SoftwareKeyFactory::default();

        let mut mdoc_data_source = MockMdocDataSource::new();
        for document in stored_documents.into_iter() {
            mdoc_data_source
                .mdocs
                .push(document.sign(&ca, &key_factory, NonZeroU8::new(1).unwrap()).await);
        }

        let device_request = DeviceRequest::from(requested_documents);

        let session_transcript = create_basic_session_transcript();
        let match_result = device_request
            .match_stored_documents(&mdoc_data_source, &session_transcript)
            .await
            .expect("Could not match device request with stored documents");

        let match_result: ExpectedDeviceRequestMatch = match_result.into();
        assert_eq!(match_result, expected_match);
    }

    #[tokio::test]
    async fn test_doc_request_verify() {
        // Create a CA, certificate and private key and trust anchors.
        let ca = KeyPair::generate_reader_mock_ca().unwrap();
        let reader_registration = ReaderRegistration::new_mock();
        let private_key = ca.generate_reader_mock(reader_registration.into()).unwrap();
        let der_trust_anchor = DerTrustAnchor::from_der(ca.certificate().as_bytes().to_vec()).unwrap();

        // Create a basic session transcript, item request and a `DocRequest`.
        let session_transcript = create_basic_session_transcript();
        let items_request = emtpy_items_request();
        let doc_request = create_doc_request(items_request.clone(), &session_transcript, &private_key).await;

        // Verification of the `DocRequest` should succeed and return the certificate contained within it.
        let certificate = doc_request
            .verify(
                &session_transcript,
                &TimeGenerator,
                &[(&der_trust_anchor.owned_trust_anchor).into()],
            )
            .expect("Could not verify DeviceRequest");

        assert_matches!(certificate, Some(cert) if cert == private_key.into());

        let other_ca = KeyPair::generate_reader_mock_ca().unwrap();
        let other_der_trust_anchor = DerTrustAnchor::from_der(other_ca.certificate().as_bytes().to_vec()).unwrap();
        let error = doc_request
            .verify(
                &session_transcript,
                &TimeGenerator,
                &[(&other_der_trust_anchor.owned_trust_anchor).into()],
            )
            .expect_err("Verifying DeviceRequest should have resulted in an error");

        assert_matches!(error, Error::Cose(_));

        // Verifying a `DocRequest` that has no reader auth should succeed and return `None`.
        let doc_request = DocRequest {
            items_request: items_request.into(),
            reader_auth: None,
        };

        let no_certificate = doc_request
            .verify(
                &session_transcript,
                &TimeGenerator,
                &[(&der_trust_anchor.owned_trust_anchor).into()],
            )
            .expect("Could not verify DeviceRequest");

        assert!(no_certificate.is_none());
    }

    #[derive(Debug, PartialEq)]
    enum ExpectedDeviceRequestMatch {
        Candidates(TestDocuments),
        MissingAttributes(IndexSet<AttributeIdentifier>),
    }

    fn candidates(candidates: TestDocuments) -> ExpectedDeviceRequestMatch {
        ExpectedDeviceRequestMatch::Candidates(candidates)
    }
    fn missing_attributes(missing_attributes: TestDocuments) -> ExpectedDeviceRequestMatch {
        ExpectedDeviceRequestMatch::MissingAttributes(missing_attributes.attribute_identifiers())
    }

    impl<T> From<DeviceRequestMatch<T>> for ExpectedDeviceRequestMatch {
        fn from(value: DeviceRequestMatch<T>) -> Self {
            match value {
                DeviceRequestMatch::Candidates(candidates) => {
                    let candidates: Vec<TestDocument> = candidates
                        .into_iter()
                        .flat_map(|(_, namespaces)| namespaces)
                        .map(convert_proposed_document)
                        .collect();
                    Self::Candidates(candidates.into())
                }
                DeviceRequestMatch::MissingAttributes(missing) => {
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

    fn convert_namespaces(namespaces: IndexMap<String, Attributes>) -> IndexMap<String, Vec<Entry>> {
        namespaces
            .into_iter()
            .map(|(namespace, attributes)| (namespace, convert_attributes(attributes)))
            .collect()
    }

    fn convert_attributes(attributes: Attributes) -> Vec<Entry> {
        attributes.0.into_iter().map(convert_attribute).collect()
    }

    fn convert_attribute(attribute: TaggedBytes<IssuerSignedItem>) -> Entry {
        Entry {
            name: attribute.0.element_identifier,
            value: attribute.0.element_value,
        }
    }
}
