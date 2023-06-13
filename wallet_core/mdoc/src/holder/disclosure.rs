use coset::{iana, CoseMac0Builder, CoseSign1Builder, HeaderBuilder};
use indexmap::IndexMap;
use x509_parser::{nom::AsBytes, prelude::X509Certificate};

use crate::{
    cose::ClonePayload,
    crypto::dh_hmac_key,
    iso::*,
    serialization::cbor_deserialize,
    signer::{MdocEcdsaKey, SecureEcdsaKey},
    verifier::X509Subject,
    Error, Result,
};

use super::{Credential, CredentialStorage, HolderError, Wallet};

impl<C: CredentialStorage> Wallet<C> {
    pub fn disclose<K: MdocEcdsaKey>(
        &self,
        device_request: &DeviceRequest,
        challenge: &[u8],
    ) -> Result<DeviceResponse> {
        let mut docs: Vec<Document> = Vec::new();

        for doc_request in &device_request.doc_requests {
            let items_request = &doc_request.items_request.0;

            // This takes any mdoc of the specified doctype. TODO: allow user choice.
            let creds = self
                .credential_storage
                .get::<K>(&items_request.doc_type)
                .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                    items_request.doc_type.clone(),
                )))?;
            let cred = &creds
                .first()
                .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                    items_request.doc_type.clone(),
                )))?
                .cred_copies[0];
            docs.push(cred.disclose_document(items_request, challenge)?);
        }

        let response = DeviceResponse {
            version: "1.0".to_string(),
            documents: Some(docs),
            document_errors: None,
            status: 0,
        };
        Ok(response)
    }
}

impl<K: MdocEcdsaKey> Credential<K> {
    pub fn disclose_document(&self, items_request: &ItemsRequest, challenge: &[u8]) -> Result<Document> {
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
            device_signed: DeviceSigned::new_signature(&self.private_key(), challenge),
            errors: None,
        };
        Ok(doc)
    }
}

impl DeviceSigned {
    pub fn new_signature(private_key: &impl SecureEcdsaKey, challenge: &[u8]) -> DeviceSigned {
        let cose = CoseSign1Builder::new()
            .payload(Vec::from(challenge))
            .protected(HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build())
            .create_signature(&[], |data| private_key.sign(data).to_vec())
            .build()
            .clone_without_payload();

        DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceSignature(cose.into()),
        }
    }

    #[allow(dead_code)] // TODO test this
    pub fn new_mac(
        private_key: &ecdsa::SigningKey<p256::NistP256>,
        reader_pub_key: &ecdsa::VerifyingKey<p256::NistP256>,
        challenge: &[u8],
    ) -> Result<DeviceSigned> {
        let device_auth: DeviceAuthenticationBytes = cbor_deserialize(challenge)?;
        let key = dh_hmac_key(
            private_key,
            reader_pub_key,
            device_auth.0.session_transcript_bts()?.as_bytes(),
            "EMacKey",
            32,
        )?;

        let cose = CoseMac0Builder::new()
            .payload(Vec::from(challenge))
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
    pub fn verify(&self, ca_cert: &X509Certificate, reader_authentication_bts: &[u8]) -> Result<Option<X509Subject>> {
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        let mut reader: Option<X509Subject> = None;
        for doc_request in &self.doc_requests {
            let (_, found) = doc_request
                .reader_auth
                .as_ref()
                .unwrap()
                .clone_with_payload(reader_authentication_bts.to_vec())
                .verify_against_cert(ca_cert)?;
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
