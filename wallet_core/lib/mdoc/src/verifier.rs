//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use p256::SecretKey;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde_with::serde_as;
use tracing::debug;
use tracing::warn;

use crypto::x509::CertificateUsage;
use http_utils::urls::HttpsUri;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;

use crate::Result;
use crate::iso::*;
use crate::utils::cose::ClonePayload;
use crate::utils::crypto::cbor_digest;
use crate::utils::crypto::dh_hmac_key;
use crate::utils::serialization::TaggedBytes;
use crate::utils::serialization::cbor_serialize;

/// Attributes of an mdoc that was disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
/// Grouped per namespace. Validity information and the attributes issuer's common_name is also included.
#[serde_as]
#[derive(Debug, Clone)]
pub struct DisclosedDocument {
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
    pub issuer_uri: HttpsUri,
    pub ca: String,
    pub validity_info: ValidityInfo,
}

/// All attributes that were disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
pub type DisclosedDocuments = IndexMap<DocType, DisclosedDocument>;

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
    #[error("missing ephemeral key")]
    EphemeralKeyMissing,
    #[error("validity error: {0}")]
    Validity(#[from] ValidityError),
    #[error("unexpected amount of CA Common Names in issuer certificate: expected 1, found {0}")]
    UnexpectedCACommonNameCount(usize),
    #[error("issuer URI {0} not found in SAN {1:?}")]
    IssuerUriNotFoundInSan(HttpsUri, VecNonEmpty<HttpsUri>),
    #[error("missing issuer URI")]
    MissingIssuerUri,
}

impl DeviceResponse {
    /// Verify a [`DeviceResponse`], returning the verified attributes, grouped per doctype and namespace.
    ///
    /// # Arguments
    /// - `eph_reader_key` - the ephemeral reader public key in case the mdoc is authentication with a MAC.
    /// - `device_authentication_bts` - the [`DeviceAuthenticationBytes`] acting as the challenge, i.e., that have to be
    ///   signed by the holder.
    /// - `time` - a generator of the current time.
    /// - `trust_anchors` - trust anchors against which verification is done.
    pub fn verify(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<DisclosedDocuments> {
        if let Some(errors) = &self.document_errors {
            return Err(VerificationError::DeviceResponseErrors(errors.clone()).into());
        }

        if self.status != 0 {
            return Err(VerificationError::UnexpectedStatus(self.status).into());
        }

        if self.documents.is_none() {
            return Err(VerificationError::NoDocuments.into());
        }

        let mut attrs = IndexMap::new();
        for doc in self.documents.as_ref().unwrap() {
            debug!("verifying document with doc_type: {}", doc.doc_type);
            let (doc_type, doc_attrs) = doc
                .verify(eph_reader_key, session_transcript, time, trust_anchors)
                .map_err(|e| {
                    warn!("document verification failed: {e}");
                    e
                })?;
            attrs.insert(doc_type, doc_attrs);
            debug!("document OK");
        }

        Ok(attrs)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidityError {
    #[error("validity parsing failed: {0}")]
    ParsingFailed(#[from] chrono::ParseError),
    #[error("not yet valid: valid from {0}")]
    NotYetValid(String),
    #[error("expired at {0}")]
    Expired(String),
}

/// Indicate how a [`ValidityInfo`] should be verified against the current date.
#[derive(Debug, Clone, Copy)]
pub enum ValidityRequirement {
    /// The [`ValidityInfo`] must not be expired, but it is allowed to be not yet valid.
    AllowNotYetValid,
    /// The [`ValidityInfo`] must be valid now and not be expired.
    Valid,
}

impl ValidityInfo {
    pub fn verify_is_valid_at(
        &self,
        time: DateTime<Utc>,
        validity: ValidityRequirement,
    ) -> std::result::Result<(), ValidityError> {
        if matches!(validity, ValidityRequirement::Valid) && time < DateTime::<Utc>::try_from(&self.valid_from)? {
            Err(ValidityError::NotYetValid(self.valid_from.0.0.clone()))
        } else if time > DateTime::<Utc>::try_from(&self.valid_until)? {
            Err(ValidityError::Expired(self.valid_from.0.0.clone()))
        } else {
            Ok(())
        }
    }
}

impl IssuerSigned {
    pub fn public_key(&self) -> Result<VerifyingKey> {
        let public_key = self
            .issuer_auth
            .dangerous_parse_unverified()?
            .0
            .device_key_info
            .try_into()?;

        Ok(public_key)
    }

    pub fn verify(
        &self,
        validity: ValidityRequirement,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(DisclosedDocument, MobileSecurityObject)> {
        let TaggedBytes(mso) =
            self.issuer_auth
                .verify_against_trust_anchors(CertificateUsage::Mdl, time, trust_anchors)?;

        mso.validity_info
            .verify_is_valid_at(time.generate(), validity)
            .map_err(VerificationError::Validity)?;

        let attributes = self
            .name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .as_ref()
                    .iter()
                    .map(|(namespace, items)| Ok((namespace.clone(), mso.verify_attrs_in_namespace(items, namespace)?)))
                    .collect::<Result<_>>()
            })
            .transpose()?
            .unwrap_or_default();

        let signing_cert = self.issuer_auth.signing_cert()?;
        let mut ca_cns = signing_cert.issuer_common_names()?;
        if ca_cns.len() != 1 {
            return Err(VerificationError::UnexpectedCACommonNameCount(ca_cns.len()).into());
        }

        let san_dns_name_or_uris = signing_cert.san_dns_name_or_uris()?;
        let issuer_uri = match mso.issuer_uri {
            Some(ref uri) if san_dns_name_or_uris.as_ref().contains(uri) => uri.to_owned(),
            Some(uri) => return Err(VerificationError::IssuerUriNotFoundInSan(uri, san_dns_name_or_uris).into()),
            None => return Err(VerificationError::MissingIssuerUri.into()),
        };

        Ok((
            DisclosedDocument {
                attributes,
                issuer_uri,
                ca: String::from(ca_cns.pop().unwrap()),
                validity_info: mso.validity_info.clone(),
            },
            mso,
        ))
    }
}

impl MobileSecurityObject {
    fn verify_attrs_in_namespace(
        &self,
        attrs: &Attributes,
        namespace: &NameSpace,
    ) -> Result<IndexMap<DataElementIdentifier, DataElementValue>> {
        attrs
            .as_ref()
            .iter()
            .map(|item| {
                self.verify_attr_digest(namespace, item)?;
                Ok((item.0.element_identifier.clone(), item.0.element_value.clone()))
            })
            .collect::<Result<_>>()
    }

    /// Given an `IssuerSignedItem` i.e. an attribute, verify that its digest is correctly included in the MSO.
    fn verify_attr_digest(&self, namespace: &NameSpace, item: &IssuerSignedItemBytes) -> Result<()> {
        let digest_id = item.0.digest_id;
        let digest = self
            .value_digests
            .0
            .get(namespace)
            .ok_or_else(|| VerificationError::MissingNamespace(namespace.clone()))?
            .0
            .get(&digest_id)
            .ok_or_else(|| VerificationError::MissingDigestID(digest_id))?;
        if *digest != cbor_digest(item)? {
            return Err(VerificationError::AttributeVerificationFailed.into());
        }
        Ok(())
    }
}

impl Document {
    pub fn verify(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(DocType, DisclosedDocument)> {
        debug!("verifying document with doc_type: {:?}", &self.doc_type);
        debug!("verify issuer_signed");
        let (attrs, mso) = self
            .issuer_signed
            .verify(ValidityRequirement::Valid, time, trust_anchors)?;

        debug!("verifying mso.doc_type matches document doc_type");
        if self.doc_type != mso.doc_type {
            return Err(VerificationError::WrongDocType {
                document: self.doc_type.clone(),
                mso: mso.doc_type,
            }
            .into());
        }

        debug!("serializing session transcript");
        let session_transcript_bts = cbor_serialize(&TaggedBytes(session_transcript))?;
        debug!("serializing device_authentication");
        let device_authentication_bts = DeviceAuthenticationKeyed::challenge(&self.doc_type, session_transcript)?;

        debug!("extracting device_key");
        let device_key = (&mso.device_key_info.device_key).try_into()?;
        match &self.device_signed.device_auth {
            DeviceAuth::DeviceSignature(sig) => {
                debug!("verifying DeviceSignature");
                sig.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&device_key)?;
            }
            DeviceAuth::DeviceMac(mac) => {
                debug!("verifying DeviceMac");
                let mac_key = dh_hmac_key(
                    eph_reader_key.ok_or_else(|| VerificationError::EphemeralKeyMissing)?,
                    &device_key.into(),
                    &session_transcript_bts,
                    "EMacKey",
                    32,
                )?;
                mac.clone_with_payload(device_authentication_bts.to_vec())
                    .verify(&mac_key)?;
            }
        }
        debug!("signature valid");

        Ok((mso.doc_type, attrs))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::Duration;
    use chrono::Utc;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crypto::examples::Examples;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;

    use crate::DeviceAuthenticationBytes;
    use crate::DeviceResponse;
    use crate::ValidityInfo;
    use crate::examples::EXAMPLE_ATTR_NAME;
    use crate::examples::EXAMPLE_ATTR_VALUE;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;
    use crate::examples::Example;
    use crate::examples::IsoCertTimeGenerator;
    use crate::holder::Mdoc;
    use crate::test;
    use crate::test::DebugCollapseBts;

    use super::*;

    fn new_validity_info(add_from_days: i64, add_until_days: i64) -> ValidityInfo {
        let now = Utc::now();
        ValidityInfo {
            signed: now.into(),
            valid_from: now.add(Duration::days(add_from_days)).into(),
            valid_until: now.add(Duration::days(add_until_days)).into(),
            expected_update: None,
        }
    }

    #[test]
    fn validity_info() {
        let now = Utc::now();

        let validity = new_validity_info(-1, 1);
        validity.verify_is_valid_at(now, ValidityRequirement::Valid).unwrap();
        validity
            .verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid)
            .unwrap();

        let validity = new_validity_info(-2, -1);
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::Valid),
            Err(ValidityError::Expired(_))
        ));
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid),
            Err(ValidityError::Expired(_))
        ));

        let validity = new_validity_info(1, 2);
        assert!(matches!(
            validity.verify_is_valid_at(now, ValidityRequirement::Valid),
            Err(ValidityError::NotYetValid(_))
        ));
        validity
            .verify_is_valid_at(now, ValidityRequirement::AllowNotYetValid)
            .unwrap();
    }

    #[tokio::test]
    async fn test_issuer_signed_public_key() {
        let key = SigningKey::random(&mut OsRng);
        let key = MockRemoteEcdsaKey::new("identifier".to_string(), key);
        let mdoc = Mdoc::new_mock_with_key(&key).await;

        let public_key = mdoc
            .issuer_signed
            .public_key()
            .expect("Could not get public key from from IssuerSigned");

        // The example mdoc should contain the generated key.
        assert_eq!(public_key, *key.verifying_key());
    }

    /// Verify the example disclosure from the standard.
    #[tokio::test]
    async fn verify_iso_example_disclosure() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let device_response = DeviceResponse::example_resigned(&ca).await;

        println!("DeviceResponse: {:#?} ", DebugCollapseBts::from(&device_response));

        // Examine the first attribute in the device response
        let document = device_response.documents.as_ref().unwrap()[0].clone();
        assert_eq!(document.doc_type, EXAMPLE_DOC_TYPE);
        let namespaces = document.issuer_signed.name_spaces.as_ref().unwrap();
        let attrs = namespaces.as_ref().get(EXAMPLE_NAMESPACE).unwrap();
        let issuer_signed_attr = attrs.as_ref().first().unwrap().0.clone();
        assert_eq!(issuer_signed_attr.element_identifier, EXAMPLE_ATTR_NAME);
        assert_eq!(issuer_signed_attr.element_value, *EXAMPLE_ATTR_VALUE);
        println!("issuer_signed_attr: {:#?}", DebugCollapseBts::from(&issuer_signed_attr));

        // Do the verification
        let eph_reader_key = Examples::ephemeral_reader_key();

        let disclosed_attrs = device_response
            .verify(
                Some(&eph_reader_key),
                &DeviceAuthenticationBytes::example().0.0.session_transcript,
                &IsoCertTimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();
        println!("DisclosedAttributes: {:#?}", DebugCollapseBts::from(&disclosed_attrs));

        // The first disclosed attribute is the same as we saw earlier in the DeviceResponse
        test::assert_disclosure_contains(
            &disclosed_attrs,
            EXAMPLE_DOC_TYPE,
            EXAMPLE_NAMESPACE,
            EXAMPLE_ATTR_NAME,
            &EXAMPLE_ATTR_VALUE,
        );
    }
}
