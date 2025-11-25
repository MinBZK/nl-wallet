//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use chrono::DateTime;
use chrono::Utc;
use futures::future::try_join_all;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::SecretKey;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use tracing::debug;
use tracing::warn;

use attestation_types::qualification::AttestationQualification;
use attestation_types::status_claim::StatusClaim;
use crypto::x509::CertificateUsage;
use http_utils::urls::HttpsUri;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;

use crate::Error;
use crate::Result;
use crate::iso::*;
use crate::utils::cose::ClonePayload;
use crate::utils::crypto::cbor_digest;
use crate::utils::crypto::dh_hmac_key;
use crate::utils::serialization::TaggedBytes;
use crate::utils::serialization::cbor_serialize;

/// Attributes of an mdoc that was disclosed in a [`DeviceResponse`], as computed by [`DeviceResponse::verify()`].
/// Grouped per namespace. Validity information and the attributes issuer's common_name is also included.
#[derive(Debug, Clone)]
pub struct DisclosedDocument {
    pub doc_type: String,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
    pub issuer_uri: HttpsUri,
    pub attestation_qualification: AttestationQualification,
    pub ca: String,
    pub validity_info: ValidityInfo,
    pub revocation_status: Option<RevocationStatus>,
    pub device_key: VerifyingKey,
}

#[derive(Debug, Clone)]
pub struct IssuerSignedVerificationResult {
    pub mso: MobileSecurityObject,
    pub attributes: IndexMap<NameSpace, IndexMap<DataElementIdentifier, DataElementValue>>,
    pub ca_common_name: String,
}

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
    #[error("missing attestation qualification")]
    MissingAttestationQualification,
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
    pub async fn verify<C>(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<C>,
    ) -> Result<Vec<DisclosedDocument>>
    where
        C: StatusListClient,
    {
        if let Some(errors) = &self.document_errors {
            return Err(VerificationError::DeviceResponseErrors(errors.clone()).into());
        }

        if self.status != 0 {
            return Err(VerificationError::UnexpectedStatus(self.status).into());
        }

        let disclosed_documents = try_join_all(
            self.documents
                .as_ref()
                .ok_or(Error::from(VerificationError::NoDocuments))?
                .iter()
                .map(|document| async {
                    debug!("verifying document with doc_type: {}", document.doc_type);

                    let disclosed_document = document
                        .verify(
                            eph_reader_key,
                            session_transcript,
                            time,
                            trust_anchors,
                            revocation_verifier,
                        )
                        .await
                        .inspect_err(|error| {
                            warn!("document verification failed: {error}");
                        })?;

                    debug!("document OK");

                    Ok::<_, Error>(disclosed_document)
                }),
        )
        .await?;

        Ok(disclosed_documents)
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
    pub fn verify(
        &self,
        validity: ValidityRequirement,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<IssuerSignedVerificationResult> {
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
        let ca_cns = signing_cert.issuer_common_names()?;
        let ca_common_name = ca_cns
            .into_iter()
            .exactly_one()
            .map_err(|error| VerificationError::UnexpectedCACommonNameCount(error.into_iter().len()))?;

        let san_dns_name_or_uris = signing_cert.san_dns_name_or_uris()?;
        match mso.issuer_uri {
            Some(ref uri) if san_dns_name_or_uris.as_ref().contains(uri) => {}
            Some(uri) => return Err(VerificationError::IssuerUriNotFoundInSan(uri, san_dns_name_or_uris).into()),
            None => return Err(VerificationError::MissingIssuerUri.into()),
        }

        let result = IssuerSignedVerificationResult {
            mso,
            attributes,
            ca_common_name: ca_common_name.to_string(),
        };

        Ok(result)
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
    pub async fn verify<C>(
        &self,
        eph_reader_key: Option<&SecretKey>,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<C>,
    ) -> Result<DisclosedDocument>
    where
        C: StatusListClient,
    {
        debug!("verifying document with doc_type: {:?}", &self.doc_type);
        debug!("verify issuer_signed");
        let IssuerSignedVerificationResult {
            mso,
            attributes,
            ca_common_name,
        } = self
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

        let attestation_qualification = mso
            .attestation_qualification
            .ok_or(VerificationError::MissingAttestationQualification)?;

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

        let revocation_status = match &mso.status {
            Some(StatusClaim::StatusList(status_list_claim)) => {
                let issuer_certificate = &self.issuer_signed.issuer_auth.signing_cert()?;
                let revocation_status = revocation_verifier
                    .verify(
                        trust_anchors,
                        issuer_certificate.distinguished_name_canonical()?,
                        status_list_claim.uri.clone(),
                        time,
                        status_list_claim.idx.try_into().unwrap(),
                    )
                    .await;
                Some(revocation_status)
            }
            _ => None,
        };

        let disclosed_document = DisclosedDocument {
            doc_type: mso.doc_type,
            attributes,
            // The presence of the `issuer_uri` is guaranteed by `IssuerSigned::verify()`.
            issuer_uri: mso.issuer_uri.unwrap(),
            attestation_qualification,
            ca: ca_common_name,
            validity_info: mso.validity_info,
            revocation_status,
            device_key,
        };

        Ok(disclosed_document)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::sync::Arc;

    use chrono::Duration;
    use chrono::Utc;

    use crypto::examples::Examples;
    use crypto::server_keys::generate::Ca;
    use token_status_list::verification::client::mock::StatusListClientStub;

    use crate::examples::EXAMPLE_ATTR_NAME;
    use crate::examples::EXAMPLE_ATTR_VALUE;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;
    use crate::examples::Example;
    use crate::examples::IsoCertTimeGenerator;
    use crate::iso::disclosure::DeviceResponse;
    use crate::iso::engagement::DeviceAuthenticationBytes;
    use crate::iso::mdocs::ValidityInfo;
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
                &RevocationVerifier::new(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock().unwrap(),
                ))),
            )
            .await
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
