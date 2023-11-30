use std::collections::HashSet;

use async_trait::async_trait;
use coset::{iana, CoseMac0Builder, Header, HeaderBuilder};
use futures::future::try_join_all;

use indexmap::IndexMap;
use p256::{PublicKey, SecretKey};
use wallet_common::keys::SecureEcdsaKey;

use crate::{
    errors::{Error, Result},
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::dh_hmac_key,
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::{cbor_serialize, CborSeq, TaggedBytes},
    },
};

use super::{HolderError, HttpClient, Mdoc, MdocRetriever, Wallet};

pub use session::{DisclosureMissingAttributes, DisclosureProposal, DisclosureSession, ProposedAttributes};

mod engagement;
mod issuer_signed;
mod proposed_document;
mod request;
mod session;

#[cfg(test)]
mod tests;

/// This trait needs to be implemented by an entity that stores mdocs.
#[async_trait]
pub trait MdocDataSource {
    // TODO: this trait should eventually replace MdocRetriever
    //       once disclosure is fully implemented.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    /// The result is a `Vec` of `Vec<Mdoc>` with the same `doc_type`. The order
    /// of the result is determined by the implementor.
    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error>;
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
                &key_factory.generate_existing(&self.private_key_id, self.issuer_signed.public_key()?),
                &cbor_serialize(&TaggedBytes(CborSeq(DeviceAuthenticationKeyed {
                    device_authentication: Default::default(),
                    session_transcript: session_transcript.clone(),
                    doc_type: self.doc_type.clone(),
                    device_name_spaces_bytes: Default::default(),
                })))?,
            )
            .await?,
            errors: None,
        };
        Ok(doc)
    }
}

impl DeviceSigned {
    pub async fn new_signature(private_key: &(impl SecureEcdsaKey + Sync), challenge: &[u8]) -> Result<DeviceSigned> {
        let cose = sign_cose(challenge, Header::default(), private_key, false).await?;

        let device_signed = DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceSignature(cose.into()),
        };

        Ok(device_signed)
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
