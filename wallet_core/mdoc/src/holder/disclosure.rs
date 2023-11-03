use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use coset::{iana, CoseMac0Builder, Header, HeaderBuilder};
use futures::future::try_join_all;
use indexmap::IndexMap;
use p256::{elliptic_curve::rand_core::OsRng, PublicKey, SecretKey};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{
    generator::{Generator, TimeGenerator},
    keys::SecureEcdsaKey,
};

use crate::{
    basic_sa_ext::Entry,
    identifiers::AttributeIdentifier,
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::{dh_hmac_key, SessionKey, SessionKeyUser},
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        serialization::{cbor_deserialize, cbor_serialize, CborSeq, TaggedBytes},
        x509::{Certificate, CertificateType, CertificateUsage},
    },
    verifier::SessionType,
    Error, Result,
};

use super::{HolderError, HttpClient, Mdoc, MdocRetriever, Wallet};

const REFERRER_URL: &str = "https://referrer.url/";

/// This trait needs to be implemented by an entity that stores mdocs.
#[async_trait]
pub trait MdocDataSource {
    // TODO: this trait should eventually replace MdocRetriever
    //       once disclosure is fully implemented.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Mdoc>, Self::Error>;
}

pub type PropsedAttributes = IndexMap<DocType, IndexMap<NameSpace, Vec<Entry>>>;

// TODO: not all of these fields may be necessary to finish the session.
#[allow(dead_code)]
pub struct DisclosureSession<H> {
    return_url: Option<Url>,
    client: H,
    verifier_url: Url,
    transcript: SessionTranscript,
    device_key: SessionKey,
    device_request: DeviceRequest,
    mdocs: Vec<Mdoc>,
    request_attribute_identifiers: HashSet<AttributeIdentifier>,
    reader_registration: ReaderRegistration,
}

impl<H> DisclosureSession<H>
where
    H: HttpClient,
{
    pub async fn start<'a>(
        client: H,
        reader_engagement_bytes: &[u8],
        return_url: Option<Url>,
        mdoc_data_source: &impl MdocDataSource,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self> {
        let reader_engagement: ReaderEngagement = cbor_deserialize(reader_engagement_bytes)?;

        // Extract both the verifier URL and public key, return an error if either is missing.
        let verifier_url = reader_engagement
            .0
            .connection_methods
            .as_ref()
            .and_then(|methods| methods.first())
            .map(|method| &method.0.connection_options.0.uri)
            .ok_or(HolderError::VerifierUrlMissing)?
            .clone();

        let verifier_pubkey = reader_engagement
            .0
            .security
            .as_ref()
            .ok_or(HolderError::VerifierEphemeralKeyMissing)?
            .try_into()?;

        // Create a new `DeviceEngagement` message and private key. Use a
        // static referrer URL, as this is not a feature we actually use.
        let (device_engagement, ephemeral_privkey) =
            DeviceEngagement::new_device_engagement(Url::parse(REFERRER_URL).unwrap())?;

        // Create the session transcript so far based on both engagement payloads.
        // TODO: Distinguish between same device and cross device flows.
        let transcript = SessionTranscript::new(SessionType::SameDevice, &reader_engagement, &device_engagement)
            .map_err(|_| HolderError::VerifierEphemeralKeyMissing)?;

        // Derive the session key for both directions from the private and public keys and the session transcript.
        let reader_key = SessionKey::new(
            &ephemeral_privkey,
            &verifier_pubkey,
            &transcript,
            SessionKeyUser::Reader,
        )?;
        let device_key = SessionKey::new(
            &ephemeral_privkey,
            &verifier_pubkey,
            &transcript,
            SessionKeyUser::Device,
        )?;

        // Send `DeviceEngagement` to verifier and decrypt the returned `DeviceRequest`.
        let session_data: SessionData = client.post(&verifier_url, &device_engagement).await?;
        let device_request: DeviceRequest = session_data.decrypt_and_deserialize(&reader_key)?;

        // A device request without `DocumentRequest` entries is useless.
        if device_request.doc_requests.is_empty() {
            return Err(HolderError::NoDocumentRequests.into());
        }

        // Verify reader authentication and decode `ReaderRegistration` from it at the same time.
        // Reader authentication is required to be present at this time.
        let reader_registration = device_request
            .verify(&transcript, &TimeGenerator, trust_anchors)?
            .ok_or(HolderError::ReaderAuthMissing)?;

        device_request
            .verify_requested_attributes(reader_registration.as_ref())
            .map_err(HolderError::from)?;

        // Make a `HashSet` of doc types from the `DeviceRequest` to account
        // for potential duplicate doc types in the request, then fetch them
        // from our data source.
        let doc_types = device_request
            .doc_requests
            .iter()
            .map(|doc_request| doc_request.items_request.0.doc_type.as_str())
            .collect::<HashSet<_>>();
        let mdocs = mdoc_data_source
            .mdoc_by_doc_types(&doc_types)
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?
            .into_iter()
            .filter(|mdoc| {
                // Sanity check, make sure this doc type is actually in the request.
                doc_types.contains(mdoc.doc_type.as_str())
            })
            .collect::<Vec<_>>();

        // Determine which doc types occur more than once with the use of
        // two `HashSet`s, one for doc types that occur at all and one for
        // doc types that were seen more than once.
        let (_, duplicate_doc_types) = mdocs.iter().fold(
            (HashSet::with_capacity(doc_types.len()), HashSet::new()),
            |(mut occuring_doc_types, mut duplicate_doc_types), mdoc| {
                let doc_type = mdoc.doc_type.as_str();

                if !occuring_doc_types.contains(doc_type) {
                    occuring_doc_types.insert(doc_type);
                } else {
                    duplicate_doc_types.insert(doc_type);
                }

                (occuring_doc_types, duplicate_doc_types)
            },
        );

        // Processing multiple `Mdoc`s with the same doc type
        // is currently not supported, so return an error.
        // TODO: Support checking for missing attributes for
        //       multiple `Mdoc`s per doctype and then either:
        //       * Picking the one remaining.
        //       * Reporting on missing attributes for all of them.
        //       * Having the caller choose between several `Mdocs`
        //         that contain the requested attributes.
        if !duplicate_doc_types.is_empty() {
            let duplicate_doc_types = duplicate_doc_types
                .into_iter()
                .map(|doc_type| doc_type.to_string())
                .collect();

            return Err(HolderError::MultipleCandidates(duplicate_doc_types).into());
        }

        // Create a `HashSet` of all the available attributes in the provided `Mdoc`s.
        let mdoc_attributes = mdocs
            .iter()
            .flat_map(|mdoc| mdoc.issuer_signed.attribute_identifiers(&mdoc.doc_type))
            .collect::<HashSet<_>>();

        // Use this `HasSet` to compare against all of the attributes requested
        // and make two `HashSet`s, one with attributes that are available and one
        // with attributes that are missing.
        let (present_attributes, missing_attributes): (HashSet<_>, HashSet<_>) = device_request
            .attribute_identifiers()
            .into_iter()
            .partition(|attribute| mdoc_attributes.contains(attribute));

        // If any attributes are missing, return an error and include the `ReaderRegistration`.
        if !missing_attributes.is_empty() {
            let error = HolderError::AttributesNotAvailable {
                reader_registration,
                missing_attributes: missing_attributes.into_iter().collect(),
            };

            return Err(error.into());
        }

        // Retain all the necessary information to either abort or finish the disclosure session later.
        let session = DisclosureSession {
            client,
            return_url,
            verifier_url,
            transcript,
            device_key,
            device_request,
            mdocs,
            reader_registration: *reader_registration,
            request_attribute_identifiers: present_attributes,
        };

        Ok(session)
    }

    pub fn return_url(&self) -> Option<&Url> {
        self.return_url.as_ref()
    }

    pub fn proposed_attributes(&self) -> PropsedAttributes {
        // For every `Mdoc`, get the attributes contained and filter
        // only those that are present in the `DeviceRequest`.
        self.mdocs
            .iter()
            .map(|mdoc| {
                let name_spaces = mdoc
                    .attributes()
                    .into_iter()
                    .filter_map(|(name_space, entries)| {
                        let entries = entries
                            .into_iter()
                            .filter(|entry| {
                                let attribute_identifier = AttributeIdentifier {
                                    doc_type: mdoc.doc_type.clone(),
                                    namespace: name_space.clone(),
                                    attribute: entry.name.clone(),
                                };

                                self.request_attribute_identifiers.contains(&attribute_identifier)
                            })
                            .collect::<Vec<_>>();

                        if entries.is_empty() {
                            return None;
                        }

                        (name_space, entries).into()
                    })
                    .collect();

                (mdoc.doc_type.clone(), name_spaces)
            })
            .collect()
    }

    pub fn reader_registration(&self) -> &ReaderRegistration {
        &self.reader_registration
    }

    // TODO: Implement terminate and disclose methods.
}

impl<H: HttpClient> Wallet<H> {
    pub async fn disclose<'a, K: MdocEcdsaKey + Sync>(
        &self,
        device_request: &DeviceRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
        mdoc_retriever: &impl MdocRetriever,
    ) -> Result<DeviceResponse> {
        let docs: Vec<Document> = try_join_all(device_request.doc_requests.iter().map(|doc_request| {
            self.disclose_document::<K>(doc_request, session_transcript, key_factory, mdoc_retriever)
        }))
        .await?;

        let response = DeviceResponse {
            version: DeviceResponseVersion::V1_0,
            documents: Some(docs),
            document_errors: None, // TODO: consider using this for reporting errors per document/mdoc
            status: 0,
        };
        Ok(response)
    }

    async fn disclose_document<'a, K: MdocEcdsaKey + Sync>(
        &self,
        doc_request: &DocRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
        mdoc_retriever: &impl MdocRetriever,
    ) -> Result<Document> {
        let items_request = &doc_request.items_request.0;

        // This takes any mdoc of the specified doctype. TODO: allow user choice.
        let creds =
            mdoc_retriever
                .get(&items_request.doc_type)
                .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                    items_request.doc_type.clone(),
                )))?;
        let cred = &creds
            .first()
            .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                items_request.doc_type.clone(),
            )))?
            .cred_copies[0];
        let document = cred
            .disclose_document(items_request, session_transcript, key_factory)
            .await?;
        Ok(document)
    }
}

impl Mdoc {
    pub async fn disclose_document<'a, K: MdocEcdsaKey + Sync>(
        &self,
        items_request: &ItemsRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
    ) -> Result<Document> {
        let disclosed_namespaces: IssuerNameSpaces = self
            .issuer_signed
            .name_spaces
            .as_ref()
            .unwrap()
            .iter()
            .filter(|&(namespace, _)| items_request.name_spaces.contains_key(namespace))
            .map(|(namespace, attributes)| {
                (
                    namespace.clone(),
                    attributes.filter(items_request.name_spaces.get(namespace).unwrap()),
                )
            })
            .collect();

        let doc = Document {
            doc_type: items_request.doc_type.clone(),
            issuer_signed: IssuerSigned {
                name_spaces: Some(disclosed_namespaces),
                issuer_auth: self.issuer_signed.issuer_auth.clone(),
            },
            device_signed: DeviceSigned::new_signature(
                &key_factory.generate_existing(&self.private_key_id, self.public_key()?),
                &cbor_serialize(&TaggedBytes(CborSeq(DeviceAuthenticationKeyed {
                    device_authentication: Default::default(),
                    session_transcript: session_transcript.clone(),
                    doc_type: self.doc_type.clone(),
                    device_name_spaces_bytes: TaggedBytes(IndexMap::new()),
                })))?,
            )
            .await,
            errors: None,
        };
        Ok(doc)
    }
}

impl DeviceSigned {
    pub async fn new_signature(private_key: &(impl SecureEcdsaKey + Sync), challenge: &[u8]) -> DeviceSigned {
        let cose = sign_cose(challenge, Header::default(), private_key, false).await;

        DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceSignature(cose.into()),
        }
    }

    #[allow(dead_code)] // TODO test this
    pub fn new_mac(
        private_key: &SecretKey,
        reader_pub_key: &PublicKey,
        session_transcript: &SessionTranscript,
        device_auth: &DeviceAuthenticationBytes,
    ) -> Result<DeviceSigned> {
        let key = dh_hmac_key(
            private_key,
            reader_pub_key,
            &cbor_serialize(&TaggedBytes(session_transcript))?,
            "EMacKey",
            32,
        )?;

        let cose = CoseMac0Builder::new()
            .payload(cbor_serialize(device_auth)?)
            .protected(HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build())
            .create_tag(&[], |data| ring::hmac::sign(&key, data).as_ref().into())
            .build()
            .clone_without_payload();

        let device_signed = DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceMac(cose.into()),
        };
        Ok(device_signed)
    }
}

impl DeviceRequest {
    /// Verify reader authentication, if present.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the
    /// the DocRequests to be signed by distinct readers. TODO maybe support this.
    /// For now, this function requires either none of the DocRequests to be signed, or all of them
    /// by the same reader.
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<Box<ReaderRegistration>>> {
        // If there are no doc requests or none of them have reader authentication, return `None`.
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }

        // Otherwise, all of the doc requests need reader authentication.
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        // Verify all `DocRequest` entries and make sure the resulting certificates are all exactly equal.
        // Note that the unwraps are safe, since we checked for the presence of reader authentication above.
        let certificate = self
            .doc_requests
            .iter()
            .try_fold(None, {
                |result_cert, doc_request| -> Result<_> {
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
            .unwrap();

        // Extract `ReaderRegistration` from the one certificate.
        let reader_registration = match CertificateType::from_certificate(&certificate).map_err(HolderError::from)? {
            Some(CertificateType::ReaderAuth(reader_registration)) => reader_registration,
            _ => return Err(HolderError::NoReaderRegistration(certificate).into()),
        };

        Ok(reader_registration.into())
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
                let reader_auth_payload = ReaderAuthenticationKeyed {
                    reader_auth_string: Default::default(),
                    session_transcript: session_transcript.clone(),
                    items_request_bytes: self.items_request.clone(),
                };
                let reader_auth_payload = TaggedBytes(CborSeq(reader_auth_payload));

                // Perform verification and return the `Certificate`.
                let cose = reader_auth.clone_with_payload(cbor_serialize(&reader_auth_payload)?);
                cose.verify_against_trust_anchors(CertificateUsage::ReaderAuth, time, trust_anchors)?;
                let cert = cose.signing_cert()?;

                Ok(cert)
            })
            .transpose()
    }
}

impl Attributes {
    /// Return a copy that contains only the items requested in `items_request`.
    fn filter(&self, requested: &DataElements) -> Attributes {
        self.0
            .clone()
            .into_iter()
            .filter(|attr| requested.contains_key(&attr.0.element_identifier))
            .collect::<Vec<_>>()
            .into()
    }
}

impl DeviceEngagement {
    pub fn new_device_engagement(referrer_url: Url) -> Result<(DeviceEngagement, SecretKey)> {
        let privkey = SecretKey::random(&mut OsRng);

        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: None,
            origin_infos: vec![
                OriginInfo {
                    cat: OriginInfoDirection::Received,
                    typ: OriginInfoType::Website(referrer_url),
                },
                OriginInfo {
                    cat: OriginInfoDirection::Delivered,
                    typ: OriginInfoType::MessageData,
                },
            ],
        };

        Ok((engagement.into(), privkey))
    }
}
