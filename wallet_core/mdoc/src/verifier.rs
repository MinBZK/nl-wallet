use indexmap::IndexMap;
use p256::NistP256;
use x509_parser::certificate::X509Certificate;
use x509_parser::nom::AsBytes;

use crate::{
    cose::ClonePayload,
    crypto::{cbor_digest, dh_hmac_key},
    iso::*,
    serialization::{cbor_deserialize, cbor_serialize, TaggedBytes},
    Result,
};

type DocumentDisclosedAttributes = IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>;
type DisclosedAttributes = IndexMap<DocType, DocumentDisclosedAttributes>;

#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("errors in device response: {0:#?}")]
    DeviceResponseErrors(Vec<DocumentError>),
    #[error("unexpected status: {0}")]
    UnexpectedStatus(u64),
    #[error("no documents found in device response")]
    NoDocuments,
    #[error("inconsistent doctypes: document contained {document}, mso contained {mso}")]
    WrongDocType { document: DocType, mso: DocType },
    #[error("namespace {0} not found in mso")]
    MissingNamespace(NameSpace),
    #[error("digest ID {0} not found in mso")]
    MissingDigestID(DigestID),
    #[error("attribute verification failed: did not hash to the value in the MSO")]
    AttributeVerificationFailed,
    #[error("DeviceAuth::DeviceMac found but no ephemeral reader key specified")]
    EphemeralKeyMissing,
}

impl DeviceResponse {
    #[allow(dead_code)] // TODO use this in verifier
    pub fn verify(
        &self,
        eph_reader_key: Option<&ecdsa::SigningKey<NistP256>>,
        device_authentication_bts: &Vec<u8>,
        ca_cert: &X509Certificate,
    ) -> Result<DisclosedAttributes> {
        if let Some(errors) = &self.document_errors {
            return Err(VerificationError::DeviceResponseErrors(errors.clone()).into());
        }
        if self.status != 0 {
            // TODO section 8.3.2.1.2.3
            return Err(VerificationError::UnexpectedStatus(self.status).into());
        }
        if self.documents.is_none() {
            return Err(VerificationError::NoDocuments.into());
        }

        let device_authentication: DeviceAuthenticationBytes = cbor_deserialize(device_authentication_bts.as_slice())?;

        let mut attrs = IndexMap::new();
        for doc in self.documents.as_ref().unwrap() {
            let (doc_type, doc_attrs) = doc.verify(
                eph_reader_key,
                &device_authentication,
                device_authentication_bts,
                ca_cert,
            )?;
            if doc_type != doc.doc_type {
                return Err(VerificationError::WrongDocType {
                    document: doc.doc_type.clone(),
                    mso: doc_type,
                }
                .into());
            }
            attrs.insert(doc_type, doc_attrs);
        }

        Ok(attrs)
    }
}

pub type X509Subject = IndexMap<String, String>;

impl IssuerSigned {
    pub fn verify(&self, ca_cert: &X509Certificate<'_>) -> Result<(DocumentDisclosedAttributes, MobileSecurityObject)> {
        let (mso, _) = self.issuer_auth.verify_against_cert(ca_cert)?;

        let mut attrs: DocumentDisclosedAttributes = IndexMap::new();
        if let Some(namespaces) = &self.name_spaces {
            for (namespace, items) in namespaces {
                attrs.insert(namespace.clone(), IndexMap::new());
                let namespace_attrs = attrs.get_mut(namespace).unwrap();
                for item in &items.0 {
                    let digest_id = item.0.digest_id;
                    let digest_ids = mso
                        .0
                        .value_digests
                        .0
                        .get(namespace)
                        .ok_or_else(|| VerificationError::MissingNamespace(namespace.clone()))?;
                    let digest = digest_ids
                        .0
                        .get(&digest_id)
                        .ok_or_else(|| VerificationError::MissingDigestID(digest_id))?;
                    if *digest != cbor_digest(item)? {
                        return Err(VerificationError::AttributeVerificationFailed.into());
                    }
                    namespace_attrs.insert(item.0.element_identifier.clone(), item.0.element_value.clone());
                }
            }
        }

        Ok((attrs, mso.0))
    }
}

impl Document {
    pub fn verify(
        &self,
        eph_reader_key: Option<&ecdsa::SigningKey<NistP256>>,
        device_authentication: &DeviceAuthenticationBytes,
        device_authentication_bts: &[u8],
        ca_cert: &X509Certificate,
    ) -> Result<(DocType, DocumentDisclosedAttributes)> {
        let (attrs, mso) = self.issuer_signed.verify(ca_cert)?;

        let device_key = (&mso.device_key_info.device_key).try_into()?;
        match &self.device_signed.device_auth {
            DeviceAuth::DeviceSignature(sig) => {
                sig.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&device_key)?;
            }
            DeviceAuth::DeviceMac(mac) => {
                let mac_key = dh_hmac_key(
                    eph_reader_key.ok_or_else(|| VerificationError::EphemeralKeyMissing)?,
                    &device_key,
                    device_authentication.0.session_transcript_bts()?.as_bytes(),
                    "EMacKey",
                    32,
                )?;
                mac.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&mac_key)?;
            }
        }

        Ok((mso.doc_type, attrs))
    }
}

impl DeviceAuthentication {
    // TODO the reader should instead take this from earlier on in the protocol
    // TODO: maybe grab this from the DeviceAuthenticationBytes instead, so we can avoid deserialize -> serialize sequence
    pub fn session_transcript_bts(&self) -> Result<Vec<u8>> {
        let tagged: TaggedBytes<&SessionTranscript> = (&self.0.session_transcript).into();
        let bts = cbor_serialize(&tagged)?;
        Ok(bts)
    }
}
