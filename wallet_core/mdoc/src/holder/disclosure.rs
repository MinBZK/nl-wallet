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

#[allow(dead_code)]
pub struct DisclosureSession<H> {
    pub return_url: Option<Url>,
    client: H,
    verifier_url: Url,
    transcript: SessionTranscript,
    device_key: SessionKey,
    device_request: DeviceRequest,
}

impl<H> DisclosureSession<H>
where
    H: HttpClient,
{
    pub async fn start<'a>(
        client: H,
        reader_engagement_bytes: &[u8],
        return_url: Option<Url>,
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
        // Note that this will panic if the `ReaderEngagement` does not contain a `Security`
        // object, which we already checked above when extracting the verifier public key.
        let transcript = SessionTranscript::new(&reader_engagement, &device_engagement);

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
        let _reader_registration = device_request.verify(&transcript, &TimeGenerator, trust_anchors)?;

        // TODO: Check requested attributes against mdocs in database.

        let session = DisclosureSession {
            client,
            return_url,
            verifier_url,
            transcript,
            device_key,
            device_request,
        };

        Ok(session)
    }

    // TODO: Add terminate and disclose methods.
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
    ) -> Result<Option<ReaderRegistration>> {
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

        Ok((*reader_registration).into())
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
