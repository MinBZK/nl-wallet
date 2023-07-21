//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use p256::NistP256;
use x509_parser::nom::AsBytes;

use crate::{
    basic_sa_ext::Entry,
    iso::*,
    utils::{
        cose::ClonePayload,
        crypto::{cbor_digest, dh_hmac_key},
        serialization::{cbor_deserialize, cbor_serialize, TaggedBytes},
        x509::{CertificateUsage, TrustAnchors},
        Generator,
    },
    Result,
};

/// Attributes of an mdoc that was disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
/// Grouped per namespace.
type DocumentDisclosedAttributes = IndexMap<NameSpace, Vec<Entry>>;
/// All attributes that were disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
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
    #[error("validity error: {0}")]
    Validity(#[from] ValidityError),
}

impl DeviceResponse {
    /// Verify a [`DeviceResponse`], returning the verified attributes, grouped per doctype and namespace.
    ///
    /// # Arguments
    /// - `eph_reader_key` - the ephemeral reader public key in case the mdoc is authentication with a MAC.
    /// - `device_authentication_bts` - the [`DeviceAuthenticationBytes`] acting as the challenge, i.e., that have
    ///   to be signed by the holder.
    /// - `ca_cert` - the CA certificate of the issuer against which the disclosure must verify.
    #[allow(dead_code)] // TODO use this in verifier
    pub fn verify(
        &self,
        eph_reader_key: Option<&ecdsa::SigningKey<NistP256>>,
        device_authentication_bts: &Vec<u8>,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &TrustAnchors,
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
                time,
                trust_anchors,
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

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidityError {
    #[error("validity parsing failed: {0}")]
    ParsingFailed(#[from] chrono::ParseError),
    #[error("not yet valid: valid from {0}")]
    NotYetValid(String),
    #[error("expired at {0}")]
    Expired(String),
}

impl ValidityInfo {
    pub fn verify_is_valid_at(&self, time: DateTime<Utc>) -> std::result::Result<(), ValidityError> {
        if time < DateTime::<Utc>::try_from(&self.valid_from)? {
            Err(ValidityError::NotYetValid(self.valid_from.0 .0.clone()))
        } else if time > DateTime::<Utc>::try_from(&self.valid_until)? {
            Err(ValidityError::Expired(self.valid_from.0 .0.clone()))
        } else {
            Ok(())
        }
    }
}

impl TryFrom<&Tdate> for DateTime<Utc> {
    type Error = ValidityError;
    fn try_from(value: &Tdate) -> std::result::Result<DateTime<Utc>, ValidityError> {
        let parsed = DateTime::parse_from_rfc3339(&value.0 .0).map(|t| t.with_timezone(&Utc))?;
        Ok(parsed)
    }
}

impl IssuerSigned {
    pub fn verify(
        &self,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &TrustAnchors,
    ) -> Result<(DocumentDisclosedAttributes, MobileSecurityObject)> {
        let (TaggedBytes(mso), _) =
            self.issuer_auth
                .verify_against_trust_anchors(CertificateUsage::Mdl, time, trust_anchors)?;

        mso.validity_info
            .verify_is_valid_at(time.generate())
            .map_err(VerificationError::Validity)?;

        let mut attrs: DocumentDisclosedAttributes = IndexMap::new();
        if let Some(namespaces) = &self.name_spaces {
            for (namespace, items) in namespaces {
                attrs.insert(namespace.clone(), Vec::new());
                let namespace_attrs = attrs.get_mut(namespace).unwrap();
                for item in &items.0 {
                    let digest_id = item.0.digest_id;
                    let digest_ids = mso
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
                    namespace_attrs.push(Entry {
                        name: item.0.element_identifier.clone(),
                        value: item.0.element_value.clone(),
                    });
                }
            }
        }

        Ok((attrs, mso))
    }
}

impl Document {
    pub fn verify(
        &self,
        eph_reader_key: Option<&ecdsa::SigningKey<NistP256>>,
        device_authentication: &DeviceAuthenticationBytes,
        device_authentication_bts: &[u8],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &TrustAnchors,
    ) -> Result<(DocType, DocumentDisclosedAttributes)> {
        let (attrs, mso) = self.issuer_signed.verify(time, trust_anchors)?;

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
