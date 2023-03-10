use crate::cose::ClonePayload;
use crate::crypto::{cbor_digest, dh_hmac_key};
use crate::iso::*;
use crate::serialization::{cbor_serialize, TaggedBytes};

use anyhow::{anyhow, bail, Context, Result};
use ciborium::value::Value;
use indexmap::IndexMap;
use p256::NistP256;
use x509_parser::certificate::X509Certificate;
use x509_parser::nom::AsBytes;

type DocumentDisclosedAttributes = IndexMap<NameSpace, IndexMap<DataElementIdentifier, Value>>;
type DisclosedAttributes = IndexMap<DocType, DocumentDisclosedAttributes>;

impl DeviceResponse {
    #[allow(dead_code)] // TODO use this in verifier
    pub(crate) fn verify(
        &self,
        eph_reader_key: Option<&ecdsa::SigningKey<NistP256>>,
        device_authentication_bts: &Vec<u8>,
        ca_cert: &X509Certificate,
    ) -> Result<DisclosedAttributes> {
        if let Some(errors) = &self.document_errors {
            bail!("errors in device response: {errors:#?}");
        }
        if self.status != 0 {
            // TODO section 8.3.2.1.2.3
            bail!("status was {}", self.status)
        }
        if self.documents.is_none() {
            bail!("no documents found")
        }

        let device_authentication: DeviceAuthenticationBytes =
            ciborium::de::from_reader(device_authentication_bts.as_slice())?;

        let mut attrs = IndexMap::new();
        for doc in self.documents.as_ref().unwrap() {
            let (doc_type, doc_attrs) = doc.verify(
                eph_reader_key,
                &device_authentication,
                device_authentication_bts,
                ca_cert,
            )?;
            if doc_type != doc.doc_type {
                bail!(
                    "wrong doc_type {} in device response: found {} in MSO",
                    doc.doc_type,
                    doc_type,
                )
            }
            attrs.insert(doc_type, doc_attrs);
        }

        Ok(attrs)
    }
}

pub type X509Subject = IndexMap<String, String>;

impl IssuerSigned {
    pub(crate) fn verify<'a>(
        &self,
        ca_cert: &X509Certificate<'a>,
    ) -> Result<(DocumentDisclosedAttributes, MobileSecurityObject)> {
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
                        .ok_or_else(|| anyhow!("namespace {namespace} not found in mso"))?;
                    let digest = digest_ids
                        .0
                        .get(&digest_id)
                        .ok_or_else(|| anyhow!("digest ID {digest_id} not found in mso"))?;
                    if *digest != cbor_digest(item)? {
                        bail!("attribute verification failed")
                    }
                    namespace_attrs.insert(
                        item.0.element_identifier.clone(),
                        item.0.element_value.clone(),
                    );
                }
            }
        }

        Ok((attrs, mso.0))
    }
}

impl Document {
    pub(crate) fn verify(
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
                    .verify(&device_key)
                    .context("mdoc validation failed")?;
            }
            DeviceAuth::DeviceMac(mac) => {
                let mac_key = dh_hmac_key(
                    eph_reader_key.ok_or_else(|| {
                        anyhow!("DeviceAuth::DeviceMac found but no ephemeral reader key specified")
                    })?,
                    &device_key,
                    device_authentication.0.session_transcript_bts()?.as_bytes(),
                    "EMacKey",
                    32,
                )?;
                mac.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&mac_key)
                    .context("mdoc validation failed")?;
            }
        }

        Ok((mso.doc_type, attrs))
    }
}

impl DeviceAuthentication {
    // TODO the reader should instead take this from earlier on in the protocol
    // TODO: maybe grab this from the DeviceAuthenticationBytes instead, so we can avoid deserialize -> serialize sequence
    pub(crate) fn session_transcript_bts(&self) -> Result<Vec<u8>> {
        let tagged: TaggedBytes<&SessionTranscript> = (&self.0.session_transcript).into();
        Ok(cbor_serialize(&tagged)?)
    }
}
