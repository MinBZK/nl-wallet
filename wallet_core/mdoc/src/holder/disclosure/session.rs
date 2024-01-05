use base64::{engine::general_purpose::STANDARD, Engine};
use futures::future::TryFutureExt;
use indexmap::IndexMap;
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{generator::TimeGenerator, utils};

use crate::{
    basic_sa_ext::Entry,
    device_retrieval::DeviceRequest,
    disclosure::{DeviceResponse, SessionData, SessionStatus},
    engagement::{DeviceEngagement, ReaderEngagement, SessionTranscript},
    errors::{Error, Result},
    holder::{HolderError, HttpClient},
    identifiers::AttributeIdentifier,
    mdocs::{DocType, NameSpace},
    utils::{
        crypto::SessionKey,
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        serialization::{self, CborError, TaggedBytes},
        x509::Certificate,
    },
    verifier::SessionType,
};

use super::{proposed_document::ProposedDocument, request::DeviceRequestMatch, MdocDataSource};

const REFERRER_URL: &str = "https://referrer.url/";
const TRANSCRIPT_HASH_PARAM: &str = "transcript_hash";

pub type ProposedAttributes = IndexMap<DocType, IndexMap<NameSpace, Vec<Entry>>>;

/// This represents a started disclosure session, which can be in one of two states.
/// Regardless of which state it is in, it provides the `ReaderRegistration` through
/// a method and allows the session to be terminated through the `terminate()` method.
///
/// The `MissingAttributes` state represents a session where not all attributes
/// requested by the verifier can be satisfied by the `Mdoc` instances stored by
/// the holder. The associated `DisclosureMissingAttributes` type only provides
/// the `missing_attributes()` method. The only thing a consumer can do in this state
/// is terminate the session, which requires user input to prevent the verifier gleaning
/// information about the holder missing the requested attributes.
///
/// The `Proposal` state represents a session where `Mdoc` candidates were selected
/// based on the requested attributes and we are waiting for user approval to disclose
/// these attributes to the verifier using the `disclose()` method. Information about
/// the proposal can be retrieved from the `DisclosureProposal` type using the
/// `proposed_attributes()` method.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DisclosureSession<H, I> {
    MissingAttributes(DisclosureMissingAttributes<H>),
    Proposal(DisclosureProposal<H, I>),
}

#[derive(Debug)]
pub struct DisclosureMissingAttributes<H> {
    data: CommonDisclosureData<H>,
    missing_attributes: Vec<AttributeIdentifier>,
}

#[derive(Debug)]
pub struct DisclosureProposal<H, I> {
    return_url: Option<Url>,
    data: CommonDisclosureData<H>,
    device_key: SessionKey,
    proposed_documents: Vec<ProposedDocument<I>>,
}

#[derive(Debug)]
struct CommonDisclosureData<H> {
    client: H,
    verifier_url: Url,
    certificate: Certificate,
    reader_registration: ReaderRegistration,
}

enum VerifierSessionDataCheckResult<I> {
    MissingAttributes(Vec<AttributeIdentifier>),
    ProposedDocuments(Vec<ProposedDocument<I>>),
}

impl<H, I> DisclosureSession<H, I>
where
    H: HttpClient,
{
    pub async fn start<'a, S>(
        client: H,
        reader_engagement_bytes: &[u8],
        return_url: Option<Url>,
        session_type: SessionType,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        // Deserialize the `ReaderEngagement` from the received bytes.
        let reader_engagement: ReaderEngagement = serialization::cbor_deserialize(reader_engagement_bytes)?;

        // Extract the verifier URL, return an error if it is is missing.
        let verifier_url = reader_engagement.verifier_url()?;

        // Create a new `DeviceEngagement` message and private key. Use a
        // static referrer URL, as this is not a feature we actually use.
        let (device_engagement, ephemeral_privkey) =
            DeviceEngagement::new_device_engagement(Url::parse(REFERRER_URL).unwrap())?;

        // Derive the session transcript and keys in both directions from the
        // `ReaderEngagement`, the `DeviceEngagement` and the ephemeral private key.
        let (transcript, reader_key, device_key) = reader_engagement.transcript_and_keys_for_device_engagement(
            session_type,
            &device_engagement,
            ephemeral_privkey,
        )?;

        // Serialize the `SessionTranscript` for if we need to add it to the `return_url` later.
        let session_transcript_bytes = serialization::cbor_serialize(&TaggedBytes(&transcript))?;

        // Send the `DeviceEngagement` to the verifier and deserialize the expected `SessionData`.
        // If decoding fails, send a `SessionData` to the verifier to report this.
        let session_data: SessionData = client
            .post(verifier_url, &device_engagement)
            .or_else(|error| async {
                if matches!(error, Error::Cbor(CborError::Deserialization(_))) {
                    // Ignore the response or any errors.
                    let _: Result<SessionData> = client.post(verifier_url, &SessionData::new_decoding_error()).await;
                }

                Err(error)
            })
            .await?;

        // Decrypt and verify the received `DeviceRequest`. From this point onwards, we should end
        // the session by sending our own `SessionData` to the verifier if we
        // encounter an error.
        let return_url_ref = return_url.as_ref();
        let (check_result, certificate, reader_registration) =
            async { session_data.decrypt_and_deserialize(&reader_key) }
                .and_then(|device_request| async move {
                    Self::verify_device_request(
                        &device_request,
                        return_url_ref,
                        transcript,
                        mdoc_data_source,
                        trust_anchors,
                    )
                    .await
                })
                .or_else(|error| async { Self::report_error_back(error, &client, verifier_url).await })
                .await?;

        let data = CommonDisclosureData {
            client,
            verifier_url: verifier_url.clone(),
            certificate,
            reader_registration,
        };

        // Create the appropriate `DisclosureSession` invariant, which contains
        // all of the information needed to either abort of finish the session.
        let session = match check_result {
            VerifierSessionDataCheckResult::MissingAttributes(missing_attributes) => {
                DisclosureSession::MissingAttributes(DisclosureMissingAttributes {
                    data,
                    missing_attributes,
                })
            }
            VerifierSessionDataCheckResult::ProposedDocuments(proposed_documents) => {
                DisclosureSession::Proposal(DisclosureProposal {
                    // If we have a return URL, add the hash of the `SessionTranscript` to it.
                    return_url: return_url.map(|url| Self::add_transcript_hash_to_url(url, &session_transcript_bytes)),
                    data,
                    device_key,
                    proposed_documents,
                })
            }
        };

        Ok(session)
    }

    async fn report_error_back<T>(error: Error, client: &H, verifier_url: &Url) -> Result<T> {
        // Determine the category of the error, so we can report on it.
        let error_session_data = match error {
            Error::Cbor(CborError::Deserialization(_)) => SessionData::new_decoding_error(),
            Error::Crypto(_) => SessionData::new_encryption_error(),
            _ => SessionData::new_termination(),
        };

        // Ignore the response or any errors.
        let _: Result<SessionData> = client.post(verifier_url, &error_session_data).await;

        Err(error)
    }

    /// Internal helper function for processing and checking the contents of a
    /// `SessionData` received from the verifier, which should contain a `DeviceRequest`.
    async fn verify_device_request<'a, S>(
        device_request: &DeviceRequest,
        return_url: Option<&Url>,
        session_transcript: SessionTranscript,
        mdoc_data_source: &S,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<(VerifierSessionDataCheckResult<I>, Certificate, ReaderRegistration)>
    where
        S: MdocDataSource<MdocIdentifier = I>,
    {
        // A device request without any attributes is useless, so return an error.
        if !device_request.has_attributes() {
            return Err(HolderError::NoAttributesRequested.into());
        }

        // Verify reader authentication and decode `ReaderRegistration` from it at the same time.
        // Reader authentication is required to be present at this time.
        let (certificate, reader_registration) = device_request
            .verify(session_transcript.clone(), &TimeGenerator, trust_anchors)?
            .ok_or(HolderError::ReaderAuthMissing)?;

        // Verify the return URL against the prefix in the `ReaderRegistration`,
        // if it was provided when starting the disclosure session.
        if let Some(return_url) = return_url {
            if !reader_registration.return_url_prefix.matches_url(return_url) {
                let urls = Box::new((reader_registration.return_url_prefix.into(), return_url.clone()));

                return Err(HolderError::ReturnUrlPrefix(urls).into());
            }
        }

        // Fetch documents from the database, calculate which ones satisfy the request and
        // formulate proposals for those documents. If there is a mismatch, return an error.
        let candidates_by_doc_type = match device_request
            .match_stored_documents(mdoc_data_source, session_transcript)
            .await?
        {
            DeviceRequestMatch::Candidates(candidates) => candidates,
            DeviceRequestMatch::MissingAttributes(missing_attributes) => {
                // Attributes are missing, return these.
                let result = VerifierSessionDataCheckResult::MissingAttributes(missing_attributes);

                return Ok((result, certificate, reader_registration));
            }
        };

        // If we have multiple candidates for any of the doc types, return an error.
        // TODO: Support having the user choose between multiple candidates.
        if candidates_by_doc_type.values().any(|candidates| candidates.len() > 1) {
            let duplicate_doc_types = candidates_by_doc_type
                .into_iter()
                .filter(|(_, candidates)| candidates.len() > 1)
                .map(|(doc_type, _)| doc_type)
                .collect();

            return Err(HolderError::MultipleCandidates(duplicate_doc_types).into());
        }

        // Now that we know that we have exactly one candidate for every `doc_type`,
        // we can flatten these candidates to a 1-dimensional `Vec`.
        let proposed_documents = candidates_by_doc_type.into_values().flatten().collect::<Vec<_>>();

        let result = VerifierSessionDataCheckResult::ProposedDocuments(proposed_documents);

        Ok((result, certificate, reader_registration))
    }

    fn add_transcript_hash_to_url(mut url: Url, session_transcript_bytes: &[u8]) -> Url {
        let transcript_hash = utils::sha256(session_transcript_bytes);

        url.query_pairs_mut()
            .append_pair(TRANSCRIPT_HASH_PARAM, &STANDARD.encode(transcript_hash));

        url
    }

    fn data(&self) -> &CommonDisclosureData<H> {
        match self {
            DisclosureSession::MissingAttributes(session) => &session.data,
            DisclosureSession::Proposal(session) => &session.data,
        }
    }

    pub fn reader_registration(&self) -> &ReaderRegistration {
        &self.data().reader_registration
    }

    pub fn verifier_certificate(&self) -> &Certificate {
        &self.data().certificate
    }

    pub async fn terminate(self) -> Result<()> {
        // Ignore the response.
        _ = self.data().terminate().await?;

        Ok(())
    }
}

impl<H> DisclosureMissingAttributes<H> {
    pub fn missing_attributes(&self) -> &[AttributeIdentifier] {
        &self.missing_attributes
    }
}

impl<H, I> DisclosureProposal<H, I>
where
    H: HttpClient,
    I: Clone,
{
    pub fn return_url(&self) -> Option<&Url> {
        self.return_url.as_ref()
    }

    pub fn proposed_source_identifiers(&self) -> Vec<&I> {
        self.proposed_documents
            .iter()
            .map(|document| &document.source_identifier)
            .collect()
    }

    pub fn proposed_attributes(&self) -> ProposedAttributes {
        // Get all of the attributes to be disclosed from the
        // prepared `IssuerSigned` on the `ProposedDocument`s.
        self.proposed_documents
            .iter()
            .map(|document| (document.doc_type.clone(), document.name_spaces()))
            .collect()
    }

    pub async fn disclose<KF, K>(&self, key_factory: &KF) -> Result<()>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        // Clone the proposed documents and construct a `DeviceResponse` by
        // signing these, then encrypt the response with the device key.
        let proposed_documents = self.proposed_documents.to_vec();
        let device_response = DeviceResponse::from_proposed_documents(proposed_documents, key_factory).await?;
        let session_data = SessionData::serialize_and_encrypt(&device_response, &self.device_key)?;

        // Send the `SessionData` containing the encrypted `DeviceResponse`.
        let response = self.data.send_session_data(&session_data).await?;

        // If we received a `SessionStatus` that is not a
        // termination in the response, return this as an error.
        match response.status {
            Some(status) if status != SessionStatus::Termination => Err(HolderError::DisclosureResponse(status).into()),
            _ => Ok(()),
        }
    }
}

impl<H> CommonDisclosureData<H>
where
    H: HttpClient,
{
    async fn send_session_data(&self, session_data: &SessionData) -> Result<SessionData> {
        self.client.post(&self.verifier_url, &session_data).await
    }

    async fn terminate(&self) -> Result<SessionData> {
        self.send_session_data(&SessionData::new_termination()).await
    }
}

#[cfg(test)]
mod tests {
    use std::convert::identity;

    use assert_matches::assert_matches;
    use http::StatusCode;
    use indexmap::IndexSet;
    use p256::{ecdsa::VerifyingKey, elliptic_curve::rand_core::OsRng, SecretKey};
    use tokio::sync::mpsc;

    use crate::{
        examples::{EXAMPLE_DOC_TYPE, EXAMPLE_NAMESPACE},
        identifiers::AttributeIdentifierHolder,
        iso::{
            disclosure::{DeviceAuth, SessionStatus},
            engagement::DeviceAuthentication,
        },
        mock::SoftwareKeyFactory,
        utils::{
            cose::{ClonePayload, CoseError},
            crypto::SessionKeyUser,
            serialization::TaggedBytes,
            x509::CertificateType,
        },
    };

    use super::{super::test_utils::*, *};

    fn test_payload_session_data_error(payload: &[u8], expected_session_status: SessionStatus) {
        let session_data: SessionData =
            serialization::cbor_deserialize(payload).expect("Sent message is not SessionData");

        assert!(session_data.data.is_none());
        assert_matches!(session_data.status, Some(session_status) if session_status == expected_session_status);
    }

    // This is the full happy path test of `DisclosureSession`.
    #[tokio::test]
    async fn test_disclosure_session() {
        // Starting a disclosure session should succeed.
        let mut payloads = Vec::with_capacity(1);
        let (disclosure_session, verifier_session, mut payload_receiver) = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            identity,
            identity,
        )
        .await
        .expect("Could not start DisclosureSession");

        // Remember the `AttributeIdentifier`s that were in the request,
        // as well as what is needed to reconstruct the `SessionTranscript`.
        let request_identifiers = verifier_session
            .items_requests
            .iter()
            .flat_map(|items_request| items_request.attribute_identifiers())
            .collect::<IndexSet<_>>();
        let session_type = verifier_session.session_type;
        let reader_engagement = verifier_session.reader_engagement.clone();

        // Make sure starting the session resulted in a proposal, get the
        // device `SessionKey`, a list of public keys and the return URL
        // from that, then actually perform disclosure.
        let (device_key, public_keys, return_url) = match disclosure_session {
            DisclosureSession::Proposal(proposal) => {
                let device_key = proposal.device_key.clone();

                // Extract the public keys from the `MobileSecurityObject`
                let public_keys: Vec<VerifyingKey> = proposal
                    .proposed_documents
                    .iter()
                    .map(|proposed_document| {
                        let TaggedBytes(mso) = proposed_document
                            .issuer_signed
                            .issuer_auth
                            .dangerous_parse_unverified()
                            .unwrap();

                        (&mso.device_key_info.device_key).try_into().unwrap()
                    })
                    .collect();

                proposal
                    .disclose(&SoftwareKeyFactory::default())
                    .await
                    .expect("Could not disclose DisclosureSession");

                (device_key, public_keys, proposal.return_url)
            }
            _ => panic!("Disclosure session should not have missing attributes"),
        };

        // Fill up `payloads` with any further messages sent.
        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        assert_eq!(payloads.len(), 2);

        // Check that the payloads are a `DeviceEngagement` and `SessionData` respectively.
        let device_engagement: DeviceEngagement = serialization::cbor_deserialize(payloads[0].as_slice())
            .expect("First message sent is not DeviceEngagement");
        let session_data: SessionData =
            serialization::cbor_deserialize(payloads[1].as_slice()).expect("Second message sent is not SessionData");

        // Decrypt the `DeviceResponse` from the `SessionData` using the device key.
        assert!(session_data.data.is_some());
        assert!(session_data.status.is_none());

        let device_response: DeviceResponse = session_data
            .decrypt_and_deserialize(&device_key)
            .expect("Could not decrypt and deserialize sent DeviceResponse");

        // Check that the attributes contained in the response match those in the request.
        let documents = device_response
            .documents
            .expect("No documents contained in DeviceResponse");

        let response_identifiers = documents
            .iter()
            .flat_map(|document| document.issuer_signed_attribute_identifiers())
            .collect::<IndexSet<_>>();

        assert_eq!(response_identifiers, request_identifiers);

        // Use the `DeviceEngagement` sent to reconstruct the `SessionTranscript`.
        let session_transcript = SessionTranscript::new(session_type, &reader_engagement, &device_engagement).unwrap();

        // Use the reconstructed `SessionTranscript` to check the generated
        // `transcript_hash` on the return URL.
        let transcript_hash = return_url
            .expect("return URL should be provided by session")
            .query_pairs()
            .find(|(key, _)| key == TRANSCRIPT_HASH_PARAM)
            .map(|(_, value)| STANDARD.decode(value.as_ref()))
            .expect("return URL should contain \"transcript_hash\" query parameter")
            .expect("return URL \"transcript_hash\" query parameter should be base64 encoded");
        let expected_transcript_hash =
            utils::sha256(&serialization::cbor_serialize(&TaggedBytes(&session_transcript)).unwrap());

        assert_eq!(transcript_hash, expected_transcript_hash);

        // Also use the `SessionTranscript` to reconstruct the `DeviceAuthentication`
        // for every `Document` received in order to verify the signatures received
        // for each of these.
        assert_eq!(documents.len(), public_keys.len());

        documents
            .into_iter()
            .zip(public_keys)
            .for_each(|(document, public_key)| {
                let device_authentication =
                    DeviceAuthentication::from_session_transcript(session_transcript.clone(), document.doc_type);
                let device_authentication_bytes =
                    serialization::cbor_serialize(&TaggedBytes(device_authentication)).unwrap();

                match document.device_signed.device_auth {
                    DeviceAuth::DeviceSignature(signature) => signature
                        .clone_with_payload(device_authentication_bytes)
                        .verify(&public_key)
                        .expect("Device authentication for document does not match public key"),
                    _ => panic!("Unexpected device authentication in DeviceResponse"),
                }
            });
    }

    #[tokio::test]
    async fn test_disclosure_session_start_proposal() {
        // Starting a disclosure session should succeed with a disclosure proposal.
        let mut payloads = Vec::with_capacity(1);
        let (disclosure_session, verifier_session, _) = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            identity,
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let proposal_session = match disclosure_session {
            DisclosureSession::MissingAttributes(_) => panic!("Disclosure session should not have missing attributes"),
            DisclosureSession::Proposal(ref session) => session,
        };

        // Test if the return `Url` scheme and host match the input and that the
        // resulting return URL now includes a `transcript_hash` query parameter.
        let return_url = proposal_session.return_url.as_ref().unwrap();
        let expected_return_url = verifier_session.return_url.as_ref().unwrap();

        assert_eq!(return_url.scheme(), expected_return_url.scheme());
        assert_eq!(return_url.host(), expected_return_url.host());
        assert!(return_url.query_pairs().any(|(key, _)| key == TRANSCRIPT_HASH_PARAM));

        // Test if the `ReaderRegistration` matches the input.
        assert_eq!(
            disclosure_session.reader_registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        // Test that a `DeviceEngagement` was sent.
        assert_eq!(payloads.len(), 1);
        let _device_engagement: DeviceEngagement =
            serialization::cbor_deserialize(payloads.first().unwrap().as_slice())
                .expect("Sent message is not DeviceEngagement");

        // Test that the proposal session contains the example mdoc identifier.
        assert_eq!(proposal_session.proposed_source_identifiers(), ["id_1"]);

        // Test that the proposal for disclosure contains the example attributes, in order.
        let entry_keys = proposal_session
            .proposed_attributes()
            .remove(EXAMPLE_DOC_TYPE)
            .and_then(|mut name_space| name_space.remove(EXAMPLE_NAMESPACE))
            .map(|entries| entries.into_iter().map(|entry| entry.name).collect::<Vec<_>>())
            .unwrap_or_default();

        assert_eq!(entry_keys, EXAMPLE_ATTRIBUTES);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_missing_attributes_one() {
        // Starting a disclosure session should succeed with missing attributes.
        let mut payloads = Vec::with_capacity(1);
        let (disclosure_session, verifier_session, _) = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            |mut mdoc_source| {
                // Remove the last attribute.
                mdoc_source
                    .mdocs
                    .first_mut()
                    .unwrap()
                    .issuer_signed
                    .name_spaces
                    .as_mut()
                    .unwrap()
                    .get_mut(EXAMPLE_NAMESPACE)
                    .unwrap()
                    .0
                    .pop();

                mdoc_source
            },
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let missing_attr_session = match disclosure_session {
            DisclosureSession::MissingAttributes(ref session) => session,
            DisclosureSession::Proposal(_) => panic!("Disclosure session should have missing attributes"),
        };

        // Test if `ReaderRegistration` matches the input.
        assert_eq!(
            disclosure_session.reader_registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        // Test that a `DeviceEngagement` was sent.
        assert_eq!(payloads.len(), 1);
        let _device_engagement: DeviceEngagement =
            serialization::cbor_deserialize(payloads.first().unwrap().as_slice())
                .expect("Sent message is not DeviceEngagement");

        let expected_missing_attributes = example_identifiers_from_attributes(["driving_privileges"]);

        itertools::assert_equal(
            missing_attr_session.missing_attributes().iter(),
            expected_missing_attributes.iter(),
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_missing_attributes_all() {
        // Starting a disclosure session should succeed with missing attributes.
        let mut payloads = Vec::with_capacity(1);
        let (disclosure_session, verifier_session, _) = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            |mut mdoc_source| {
                mdoc_source.mdocs.clear();

                mdoc_source
            },
            identity,
        )
        .await
        .expect("Could not start disclosure session");

        // Check that the correct session type is returned.
        let missing_attr_session = match disclosure_session {
            DisclosureSession::MissingAttributes(ref session) => session,
            DisclosureSession::Proposal(_) => panic!("Disclosure session should have missing attributes"),
        };

        // Test if `ReaderRegistration` matches the input.
        assert_eq!(
            disclosure_session.reader_registration(),
            verifier_session.reader_registration.as_ref().unwrap()
        );

        // Test that a `DeviceEngagement` was sent.
        assert_eq!(payloads.len(), 1);
        let _device_engagement: DeviceEngagement =
            serialization::cbor_deserialize(payloads.first().unwrap().as_slice())
                .expect("Sent message is not DeviceEngagement");

        let expected_missing_attributes = example_identifiers_from_attributes([
            "family_name",
            "issue_date",
            "expiry_date",
            "document_number",
            "driving_privileges",
        ]);

        itertools::assert_equal(
            missing_attr_session.missing_attributes().iter(),
            expected_missing_attributes.iter(),
        );
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_decode_reader_engagement() {
        // Starting a `DisclosureSession` with invalid `ReaderEngagement`
        // bytes should result in a `Error::Cbor` error.
        let mut payloads = Vec::new();
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.reader_engagement_bytes_override = vec![].into();

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Cbor(_));
        assert!(payloads.is_empty());
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_verifier_url_mising() {
        // Starting a `DisclosureSession` with a `ReaderEngagement` that
        // does not contain a verifier URL should result in an error.
        let mut payloads = Vec::new();
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                if let Some(methods) = verifier_session.reader_engagement.0.connection_methods.as_mut() {
                    methods.clear()
                }

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::VerifierUrlMissing));
        assert!(payloads.is_empty());
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_verifier_ephemeral_key_missing() {
        // Starting a `DisclosureSession` with a `ReaderEngagement` that does not
        // contain an ephemeral verifier public key should result in an error.
        let mut payloads = Vec::new();
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.reader_engagement.0.security = None;

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::VerifierEphemeralKeyMissing));
        assert!(payloads.is_empty());
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_session_type() {
        // Starting a `DisclosureSession` with the wrong `SessionType`
        // should result in a decryption error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::CrossDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Crypto(_));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::EncryptionError);
    }

    async fn test_disclosure_session_start_error_http_client<F>(error_factory: F) -> (Error, Vec<Vec<u8>>)
    where
        F: Fn() -> Error,
    {
        // Set up a `MockHttpClient` with the receiver `error_factory`.
        let (payload_sender, mut payload_receiver) = mpsc::channel(256);
        let client = MockHttpClient {
            response_factory: || MockHttpClientResponse::Error(error_factory()),
            payload_sender,
        };

        // Set up a basic `ReaderEngagement` and `MdocDataSource` (which is not actually consulted).
        let (reader_engagement, _) = ReaderEngagement::new_reader_engagement(SESSION_URL.parse().unwrap()).unwrap();
        let mdoc_data_source = MockMdocDataSource::default();

        let error = DisclosureSession::start(
            client,
            &serialization::cbor_serialize(&reader_engagement).unwrap(),
            None,
            SessionType::SameDevice,
            &mdoc_data_source,
            &[],
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        // Collect the serialized payloads sent through the `HttpClient`.
        let mut payloads = Vec::with_capacity(2);

        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        (error, payloads)
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_http_client_data_serialization() {
        // Set up the `MockHttpClient` to return a `CborError::Serialization`.
        let (error, payloads) = test_disclosure_session_start_error_http_client(|| {
            CborError::from(ciborium::ser::Error::Value("".to_string())).into()
        })
        .await;

        // Test that we got the expected error and that no `SessionData`
        // was sent to the verifier to report the error.
        assert_matches!(error, Error::Cbor(CborError::Serialization(_)));
        assert_eq!(payloads.len(), 1);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_http_client_request() {
        // Set up the `MockHttpClient` to return a `HolderError::Serialization`.
        let (error, payloads) = test_disclosure_session_start_error_http_client(|| {
            let response = http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("")
                .unwrap();
            let reqwest_error = reqwest::Response::from(response).error_for_status().unwrap_err();

            HolderError::from(reqwest_error).into()
        })
        .await;

        // Test that we got the expected error and that no `SessionData`
        // was sent to the verifier to report the error.
        assert_matches!(error, Error::Holder(HolderError::RequestError(_)));
        assert_eq!(payloads.len(), 1);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_http_client_data_deserialization() {
        // Set up the `MockHttpClient` to return a `CborError::Deserialization`.
        let (error, payloads) = test_disclosure_session_start_error_http_client(|| {
            CborError::from(ciborium::de::Error::RecursionLimitExceeded).into()
        })
        .await;

        // Test that we got the expected error and that the last payload
        // is a `SessionData` containing the expected `SessionStatus`.
        assert_matches!(error, Error::Cbor(CborError::Deserialization(_)));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::DecodingError);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_no_attributes_requested() {
        // Starting a `DisclosureSession` in which a `DeviceRequest` with no
        // `DocRequest` entries is received should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.items_requests.clear();

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::NoAttributesRequested));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);

        // Starting a `DisclosureSession` in which a `DeviceRequest` with an
        // empty `DocRequest` entry is received should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.items_requests = vec![emtpy_items_request()];

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::NoAttributesRequested));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_auth_missing() {
        // Starting a `DisclosureSession` where the received `DeviceRequest`
        // does not have reader auth should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            identity,
            |mut device_request| {
                device_request
                    .doc_requests
                    .iter_mut()
                    .for_each(|doc_request| doc_request.reader_auth = None);

                device_request
            },
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::ReaderAuthMissing));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);

        // Starting a `DisclosureSession` where not all of the `DocRequest`s in the
        // received `DeviceRequest` contain reader auth should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            identity,
            |mut device_request| {
                let mut doc_request = device_request.doc_requests.first().unwrap().clone();
                doc_request.reader_auth = None;
                device_request.doc_requests.push(doc_request);

                device_request
            },
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::ReaderAuthMissing));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_auth_invalid() {
        // Starting a `DisclosureSession` without trust anchors should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.trust_anchors.clear();

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Cose(CoseError::Certificate(_)));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_reader_registration_validation() {
        // Starting a `DisclosureSession` where the `DeviceRequest` contains an attribute
        // that is not in the `ReaderRegistration` should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session
                    .items_requests
                    .first_mut()
                    .unwrap()
                    .name_spaces
                    .get_mut(EXAMPLE_NAMESPACE)
                    .unwrap()
                    .insert("foobar".to_string(), false);

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::ReaderRegistrationValidation(_)));
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_return_url_prefix() {
        // Starting a `DisclosureSession` where the the return URL does not match the return
        // URL prefix contained in the verifier certificate should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            |mut verifier_session| {
                verifier_session.return_url = Url::parse("https://not-example.com/return").unwrap().into();

                verifier_session
            },
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        let expected_urls = (
            "https://example.com/".parse().unwrap(),
            "https://not-example.com/return".parse().unwrap(),
        )
            .into();
        assert_matches!(error, Error::Holder(HolderError::ReturnUrlPrefix(urls)) if urls == expected_urls);
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_mdoc_data_source() {
        // Starting a `DisclosureSession` when the database returns
        // an error should result in that error being forwarded.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            |mut mdoc_source| {
                mdoc_source.has_error = true;

                mdoc_source
            },
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            Error::Holder(HolderError::MdocDataSource(mdoc_error)) if mdoc_error.is::<MdocDataSourceError>()
        );
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_multiple_candidates() {
        // Starting a `DisclosureSession` when the database contains multiple
        // candidates for the same `doc_type` should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::WithReaderRegistration,
            &mut payloads,
            identity,
            |mut mdoc_source| {
                mdoc_source.mdocs.push(mdoc_source.mdocs.first().unwrap().clone());

                mdoc_source
            },
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(
            error,
            Error::Holder(HolderError::MultipleCandidates(doc_types)) if doc_types == vec![EXAMPLE_DOC_TYPE.to_string()]
        );
        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_no_reader_registration() {
        // Starting a `DisclosureSession` with a `ReaderEngagement` that contains a valid
        // reader certificate but no `ReaderRegistration` should result in an error.
        let mut payloads = Vec::with_capacity(2);
        let error = disclosure_session_start(
            SessionType::SameDevice,
            ReaderCertificateKind::NoReaderRegistration,
            &mut payloads,
            identity,
            identity,
            identity,
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Holder(HolderError::NoReaderRegistration(_)));

        assert_eq!(payloads.len(), 2);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);
    }

    fn create_disclosure_session_proposal<F>(
        response_factory: F,
    ) -> (
        DisclosureSession<MockHttpClient<F>, MdocIdentifier>,
        mpsc::Receiver<Vec<u8>>,
    )
    where
        F: Fn() -> MockHttpClientResponse,
    {
        let privkey = SecretKey::random(&mut OsRng);
        let pubkey = SecretKey::random(&mut OsRng).public_key();
        let session_transcript = create_basic_session_transcript();
        let device_key = SessionKey::new(&privkey, &pubkey, &session_transcript, SessionKeyUser::Device).unwrap();

        let (payload_sender, payload_receiver) = mpsc::channel(256);
        let client = MockHttpClient {
            response_factory,
            payload_sender,
        };

        let proposal_session = DisclosureSession::Proposal(DisclosureProposal {
            return_url: Url::parse(RETURN_URL).unwrap().into(),
            data: CommonDisclosureData {
                client,
                verifier_url: SESSION_URL.parse().unwrap(),
                certificate: vec![].into(),
                reader_registration: Default::default(),
            },
            device_key,
            proposed_documents: vec![create_example_proposed_document()],
        });

        (proposal_session, payload_receiver)
    }

    async fn test_disclosure_session_terminate<H>(
        session: DisclosureSession<H, MdocIdentifier>,
        mut payload_receiver: mpsc::Receiver<Vec<u8>>,
    ) -> Result<()>
    where
        H: HttpClient,
    {
        let result = session.terminate().await;

        let mut payloads = Vec::with_capacity(1);

        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        assert_eq!(payloads.len(), 1);

        test_payload_session_data_error(payloads.last().unwrap(), SessionStatus::Termination);

        result
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_terminate() {
        let (proposal_session, payload_receiver) =
            create_disclosure_session_proposal(|| MockHttpClientResponse::SessionStatus(SessionStatus::Termination));

        // Terminating a `DisclosureSession` with a proposal should succeed.
        test_disclosure_session_terminate(proposal_session, payload_receiver)
            .await
            .expect("Could not terminate DisclosureSession with proposal");

        let (proposal_session, payload_receiver) = create_disclosure_session_proposal(|| {
            MockHttpClientResponse::Error(CborError::from(ciborium::ser::Error::Value("".to_string())).into())
        });

        // Terminating a `DisclosureSession` with a proposal where the `HttpClient`
        // gives an error should result in that error being forwarded.
        let error = test_disclosure_session_terminate(proposal_session, payload_receiver)
            .await
            .expect_err("Terminating DisclosureSession with proposal should have resulted in an error");

        assert_matches!(error, Error::Cbor(_));
    }

    #[tokio::test]
    async fn test_disclosure_session_missing_attributes_terminate() {
        let (payload_sender, payload_receiver) = mpsc::channel(256);
        let client = MockHttpClient {
            response_factory: || MockHttpClientResponse::SessionStatus(SessionStatus::Termination),
            payload_sender,
        };

        let (ca_cert, ca_key) = Certificate::new_ca("test-ca").unwrap();
        let (certificate, _) = Certificate::new(
            &ca_cert,
            &ca_key,
            "test-certificate",
            CertificateType::ReaderAuth(Some(Box::default())),
        )
        .unwrap();

        // Terminating a `DisclosureSession` with missing attributes should succeed.
        let missing_attr_session = DisclosureSession::MissingAttributes(DisclosureMissingAttributes {
            data: CommonDisclosureData {
                client,
                verifier_url: SESSION_URL.parse().unwrap(),
                certificate: certificate.clone(),
                reader_registration: Default::default(),
            },
            missing_attributes: Default::default(),
        });

        test_disclosure_session_terminate(missing_attr_session, payload_receiver)
            .await
            .expect("Could not terminate DisclosureSession with missing attributes");

        let (payload_sender, payload_receiver) = mpsc::channel(256);
        let client = MockHttpClient {
            response_factory: || {
                MockHttpClientResponse::Error(CborError::from(ciborium::ser::Error::Value("".to_string())).into())
            },
            payload_sender,
        };

        let missing_attr_session = DisclosureSession::MissingAttributes(DisclosureMissingAttributes {
            data: CommonDisclosureData {
                client,
                verifier_url: SESSION_URL.parse().unwrap(),
                certificate,
                reader_registration: Default::default(),
            },
            missing_attributes: Default::default(),
        });

        // Terminating a `DisclosureSession` with missing attributes where the
        // `HttpClient` gives an error should result in that error being forwarded.
        let error = test_disclosure_session_terminate(missing_attr_session, payload_receiver)
            .await
            .expect_err("Terminating DisclosureSession with missing attributes should have resulted in an error");

        assert_matches!(error, Error::Cbor(_));
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose() {
        let (proposal_session, mut payload_receiver) =
            create_disclosure_session_proposal(|| MockHttpClientResponse::SessionStatus(SessionStatus::Termination));

        // Signing a `DisclosureSession` with a proposal should succeed.
        let device_key = match proposal_session {
            DisclosureSession::Proposal(proposal) => {
                let device_key = proposal.device_key.clone();

                proposal
                    .disclose(&SoftwareKeyFactory::default())
                    .await
                    .expect("Could not disclose DisclosureSession");

                device_key
            }
            _ => unreachable!(),
        };

        // Check that this resulted in exactly one payload being sent.
        let mut payloads = Vec::with_capacity(1);

        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        assert_eq!(payloads.len(), 1);

        // Deserialize the `SessionData` and decrypt its contents with the reader key.
        let session_data: SessionData = serialization::cbor_deserialize(payloads.last().unwrap().as_slice())
            .expect("Sent message is not SessionData");

        assert!(session_data.data.is_some());
        assert!(session_data.status.is_none());

        let device_response: DeviceResponse = session_data
            .decrypt_and_deserialize(&device_key)
            .expect("Could not decrypt and deserialize sent DeviceResponse");

        // The identifiers of the `DeviceResponse` should match those in the example `Mdoc`.
        let identifiers = device_response
            .documents
            .unwrap()
            .first()
            .unwrap()
            .issuer_signed_attribute_identifiers();

        assert_eq!(identifiers, example_mdoc_attribute_identifiers());
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_http_client_request() {
        // Create a `DisclosureSession` containing a proposal
        // and a `HttpClient` that will return a `reqwest::Error`.
        let (proposal_session, mut payload_receiver) = create_disclosure_session_proposal(|| {
            let response = http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("")
                .unwrap();
            let reqwest_error = reqwest::Response::from(response).error_for_status().unwrap_err();

            MockHttpClientResponse::Error(HolderError::from(reqwest_error).into())
        });

        // Disclosing this session should result in the payload
        // being sent while returning the wrapped HTTP error.
        let error = match proposal_session {
            DisclosureSession::Proposal(proposal) => proposal
                .disclose(&SoftwareKeyFactory::default())
                .await
                .expect_err("Disclosing DisclosureSession should have resulted in an error"),
            _ => unreachable!(),
        };

        let mut payloads = Vec::with_capacity(1);

        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        assert_matches!(error, Error::Holder(HolderError::RequestError(_)));
        assert_eq!(payloads.len(), 1);
    }

    #[tokio::test]
    async fn test_disclosure_session_proposal_disclose_error_disclosure_response() {
        // Create a `DisclosureSession` containing a proposal and a `HttpClient`
        // that will return a `SessionStatus` that is not `Termination`.
        let (proposal_session, mut payload_receiver) =
            create_disclosure_session_proposal(|| MockHttpClientResponse::SessionStatus(SessionStatus::DecodingError));

        // Disclosing this session should result in the payload
        // being sent while returning a `DisclosureResponse` error.
        let error = match proposal_session {
            DisclosureSession::Proposal(proposal) => proposal
                .disclose(&SoftwareKeyFactory::default())
                .await
                .expect_err("Disclosing DisclosureSession should have resulted in an error"),
            _ => unreachable!(),
        };

        let mut payloads = Vec::with_capacity(1);

        while let Ok(payload) = payload_receiver.try_recv() {
            payloads.push(payload);
        }

        assert_matches!(
            error,
            Error::Holder(HolderError::DisclosureResponse(SessionStatus::DecodingError))
        );
        assert_eq!(payloads.len(), 1);
    }
}
