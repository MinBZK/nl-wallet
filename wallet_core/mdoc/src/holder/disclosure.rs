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

    /// For every doctype, return a vec of `Mdoc` entities
    /// represent it. This means that the returned `Vec` should
    /// be the same length as the `doctypes` iterator.
    async fn mdoc_by_doctypes(
        &self,
        doctypes: impl Iterator<Item = impl AsRef<str>> + Send,
    ) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error>;
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
    mdocs: Vec<Mdoc>,
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

        // Get a `Vec` of doctypes from the `DeviceRequest` and get them from our data source.
        let doc_types = device_request
            .doc_requests
            .iter()
            .map(|doc_request| doc_request.items_request.0.doc_type.as_str())
            .collect::<Vec<_>>();
        let mdocs = mdoc_data_source
            .mdoc_by_doctypes(doc_types.iter())
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?;

        // Do a sanity check, every returned `Mdoc` should match the requested doctype.
        doc_types.iter().zip(&mdocs).for_each(|(doc_type, typed_mdocs)| {
            typed_mdocs.iter().for_each(|mdoc| {
                if mdoc.doc_type != *doc_type {
                    panic!(
                        "Inconsistent mdoc doc_type received from storage, expected \"{}\", received \"{}\"",
                        doc_type, mdoc.doc_type
                    );
                }
            })
        });

        // Choosing between multiple `Mdoc`s with the same doctype
        // is currently not supported, so return an error.
        if mdocs.iter().any(|typed_mdocs| typed_mdocs.len() > 1) {
            let multiple_mdoc_doctypes = mdocs
                .into_iter()
                .filter(|typed_mdocs| typed_mdocs.len() > 1)
                .map(|mut typed_mdocs| typed_mdocs.pop().unwrap().doc_type)
                .collect::<Vec<_>>();

            return Err(HolderError::MultipleCandidates(multiple_mdoc_doctypes).into());
        }

        // Filter out empty `Vec<Mdoc>` results (meaning the length no longer matches
        // the requested doctypes) and take the one `Mdoc` out of its `Vec`.
        let mdocs = mdocs
            .into_iter()
            .filter_map(|mut typed_mdocs| typed_mdocs.pop())
            .collect::<Vec<_>>();

        // Calculate missing attributes for the request, given the available `Mdoc`s.
        let missing_attributes = device_request.missing_attributes_for_mdocs(mdocs.iter());

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

pub type MissingAttributes = Vec<(DocType, Vec<(NameSpace, Vec<DataElementIdentifier>)>)>;

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

    /// Given a set of `Mdocs`, calculate which attributes are missing and return this in a structured fashion.
    pub fn missing_attributes_for_mdocs<'a>(&self, mdocs: impl Iterator<Item = &'a Mdoc>) -> MissingAttributes {
        // Create a pair of nested `HashMaps`, which in turn contains a `HashSet`.
        // The maps are indexed by `DocType` and `NameSpace` respectively, while
        // the set contains `DataElementIdentifier`s (these are all strings).
        let mdoc_attributes = mdocs
            .into_iter()
            .map(|mdoc| {
                let name_spaces = mdoc
                    .issuer_signed
                    .name_spaces
                    .as_ref()
                    .map(|name_spaces| {
                        name_spaces
                            .iter()
                            .map(|(name_space, attributes)| {
                                let attributes = attributes
                                    .0
                                    .iter()
                                    .map(|item| item.0.element_identifier.as_str())
                                    .collect::<HashSet<_>>();

                                (name_space.as_str(), attributes)
                            })
                            .collect::<HashMap<_, _>>()
                    })
                    .unwrap_or_default();

                (mdoc.doc_type.as_str(), name_spaces)
            })
            .collect::<HashMap<_, _>>();

        // Create the `MissingAttributes` nested `Vecs` based on our
        // `DocRequest`s. Note that, if all attributes are present in
        // the `Mdocs`, the root level `Vec` should be empty.
        self.doc_requests
            .iter()
            .map(|doc_request| &doc_request.items_request.0)
            .flat_map(|item_request| {
                let doc_type = &item_request.doc_type;

                let name_spaces = item_request
                    .name_spaces
                    .iter()
                    .flat_map(|(name_space, attributes)| {
                        let attributes = attributes
                            .keys()
                            .filter(|attribute| {
                                // At the attribute level, use the `HashMap` created earlier to
                                // do a nested lookup, to see if the attribute is present.
                                let attribute_present = mdoc_attributes
                                    .get(doc_type.as_str())
                                    .and_then(|name_spaces| name_spaces.get(name_space.as_str()))
                                    .map(|attributes| attributes.contains(attribute.as_str()))
                                    .unwrap_or_default();

                                !attribute_present
                            })
                            .cloned()
                            .collect::<Vec<_>>();

                        if attributes.is_empty() {
                            return None;
                        }

                        (name_space.clone(), attributes).into()
                    })
                    .collect::<Vec<_>>();

                if name_spaces.is_empty() {
                    return None;
                }

                (doc_type.clone(), name_spaces).into()
            })
            .collect::<Vec<_>>()
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
