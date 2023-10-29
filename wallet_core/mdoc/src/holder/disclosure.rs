use std::collections::{HashMap, HashSet};

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
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::{dh_hmac_key, SessionKey, SessionKeyUser},
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        serialization::{cbor_deserialize, cbor_serialize, CborSeq, TaggedBytes},
        x509::{Certificate, CertificateType, CertificateUsage},
    },
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

// TODO: not all of these fields may be necessary to finish the session.
#[allow(dead_code)]
pub struct DisclosureSession<H> {
    pub return_url: Option<Url>,
    client: H,
    verifier_url: Url,
    transcript: SessionTranscript,
    device_key: SessionKey,
    device_request: DeviceRequest,
    pub reader_registration: ReaderRegistration,
    mdocs: HashMap<DocType, Mdoc>,
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
            .ok_or(HolderError::VerifiedUrlMissing)?
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
        let transcript = SessionTranscript::new(&reader_engagement, &device_engagement)
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

        // Make a `HashSet` of doc types from the `DeviceRequest` to account
        // for potential duplicate doc types in the request, then fetch them
        // from our data source.
        let doc_types = device_request
            .doc_requests
            .iter()
            .map(|doc_request| doc_request.items_request.0.doc_type.as_str())
            .collect::<HashSet<_>>();
        let source_mdocs = mdoc_data_source
            .mdoc_by_doc_types(&doc_types)
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?;

        // Build a `HashMap` of the returned `Mdoc`s, based on their doc type.
        let mdocs = source_mdocs.into_iter().fold(
            HashMap::<_, Vec<_>>::with_capacity(doc_types.len()),
            |mut mdocs, mdoc| {
                // Sanity check, make sure this doc type is actually in the request.
                if let Some(doc_type) = doc_types.get(mdoc.doc_type.as_str()) {
                    mdocs.entry(*doc_type).or_default().push(mdoc);
                }

                mdocs
            },
        );

        // Choosing between multiple `Mdoc`s with the same doc type
        // is currently not supported, so return an error.
        // TODO: Support choosing which mdoc/attribute to disclose.
        // TODO: Should we at least support combining attributes from different `Mdoc`s?
        if mdocs.values().any(|typed_mdocs| typed_mdocs.len() > 1) {
            let multiple_mdoc_doc_types = mdocs
                .into_values()
                .filter(|typed_mdocs| typed_mdocs.len() > 1)
                .map(|mut typed_mdocs| typed_mdocs.pop().unwrap().doc_type)
                .collect::<Vec<_>>();

            return Err(HolderError::MultipleCandidates(multiple_mdoc_doc_types).into());
        }

        // Flatten the collection of `Mdoc`s, now that we
        // know that there is at most one per doc type.
        let mdocs = mdocs
            .into_iter()
            .flat_map(|(doc_type, mut typed_mdocs)| typed_mdocs.pop().map(|mdoc| (doc_type.to_string(), mdoc)))
            .collect::<HashMap<_, _>>();

        // Calculate missing attributes for the request, given the available `Mdoc`s.
        // If all attributes are present in all `Mdoc`s, this should be empty.
        let missing_attributes = device_request
            .doc_requests
            .iter()
            .filter_map(|doc_request| {
                let doc_type = doc_request.items_request.0.doc_type.as_str();
                let mdoc = mdocs.get(doc_type);
                let missing_mdoc_attributes = doc_request.missing_attributes_for_mdoc(mdoc);

                if missing_mdoc_attributes.is_empty() {
                    return None;
                }

                (doc_type.to_string(), missing_mdoc_attributes).into()
            })
            .collect::<Vec<_>>();

        // If there are any, return an error and include the `ReaderRegistration`.
        if !missing_attributes.is_empty() {
            let error = HolderError::AttributesNotAvailable {
                reader_registration,
                missing_attributes,
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
            reader_registration: *reader_registration,
            mdocs,
        };

        Ok(session)
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

pub type MissingAttributes = Vec<(DocType, MissingDocumentAttributes)>;

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

    /// Given a `HashSet` of `Mdocs`, calculate which attributes are missing
    /// and return these in a structured fashion. If all attributes are present,
    /// this returns an empty `Vec`.
    pub fn missing_attributes_for_mdocs(&self, mdocs: &HashMap<DocType, Mdoc>) -> MissingAttributes {
        self.doc_requests
            .iter()
            .filter_map(|doc_request| {
                let doc_type = doc_request.items_request.0.doc_type.as_str();
                let mdoc = mdocs.get(doc_type);
                let missing_mdoc_attributes = doc_request.missing_attributes_for_mdoc(mdoc);

                if missing_mdoc_attributes.is_empty() {
                    return None;
                }

                (doc_type.to_string(), missing_mdoc_attributes).into()
            })
            .collect::<Vec<_>>()
    }
}

pub type MissingDocumentAttributes = Vec<(NameSpace, Vec<DataElementIdentifier>)>;

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

    /// Calculate which attributes for which name spaces are missing from the request,
    /// given an optional `Mdoc` and return these in a structured fashion. If the `Mdoc`
    /// is not provided, all attributes will be returned as missing.
    pub fn missing_attributes_for_mdoc(&self, mdoc: Option<&Mdoc>) -> MissingDocumentAttributes {
        let mdoc_name_spaces = mdoc.and_then(|mdoc| mdoc.issuer_signed.name_spaces.as_ref());

        // Note that this `Vec` will be empty if all attributes
        // in all name spaces are present in the `Mdoc`.
        self.items_request
            .0
            .name_spaces
            .iter()
            .flat_map(|(name_space, attributes)| {
                // Calculate a `HasSet` of attributes that are present in the
                // `Mdoc` for the name space, which may be empty if there is no `Mdoc`.
                let mdoc_attribute_set = mdoc_name_spaces
                    .and_then(|mdoc_name_spaces| mdoc_name_spaces.get(name_space))
                    .map(|mdoc_attributes| {
                        mdoc_attributes
                            .0
                            .iter()
                            .map(|attribute| attribute.0.element_identifier.as_str())
                            .collect::<HashSet<_>>()
                    })
                    .unwrap_or_default();

                // Look up which of the attributes are missing from the `Mdoc`
                // and place those in a `Vec`.
                let missing_attributes = attributes
                    .keys()
                    .filter(|attribute| !mdoc_attribute_set.contains(attribute.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();

                if missing_attributes.is_empty() {
                    return None;
                }

                (name_space.clone(), missing_attributes).into()
            })
            .collect()
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
