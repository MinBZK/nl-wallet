use chrono::{DateTime, Utc};
use coset::{iana, CoseMac0Builder, Header, HeaderBuilder};
use futures::future::try_join_all;
use indexmap::IndexMap;
use p256::{elliptic_curve::rand_core::OsRng, PublicKey, SecretKey};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, keys::SecureEcdsaKey};

use crate::{
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::dh_hmac_key,
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::{cbor_deserialize, cbor_serialize, CborSeq, TaggedBytes},
        x509::CertificateUsage,
    },
    verifier::X509Subject,
    Error, Result,
};

use super::{HolderError, HttpClient, Mdoc, MdocRetriever, Wallet};

// TODO: Implement actual disclosure.
#[allow(dead_code)]
pub struct DisclosureSession {
    reader_engagement: ReaderEngagement,
    pub return_url: Option<Url>,
}

impl DisclosureSession {
    pub fn start(reader_engagement_bytes: &[u8], return_url: Option<Url>) -> Result<Self> {
        let reader_engagement = cbor_deserialize(reader_engagement_bytes)?;

        let session = DisclosureSession {
            reader_engagement,
            return_url,
        };

        Ok(session)
    }
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
    // TODO use in client
    pub fn verify(
        &self,
        reader_authentication_bts: &[u8],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<X509Subject>> {
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        let mut reader: Option<X509Subject> = None;
        for doc_request in &self.doc_requests {
            let cose = doc_request
                .reader_auth
                .as_ref()
                .unwrap()
                .clone_with_payload(reader_authentication_bts.to_vec());
            cose.verify_against_trust_anchors(CertificateUsage::ReaderAuth, time, trust_anchors)?;
            let found = cose.signing_cert()?.subject().map_err(HolderError::CertificateError)?;
            if reader.is_none() {
                reader.replace(found);
            } else if *reader.as_ref().unwrap() != found {
                return Err(HolderError::ReaderAuthsInconsistent.into());
            }
        }

        Ok(reader)
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
