//! RP software, for verifying mdoc disclosures, see [`DeviceResponse::verify()`].

use chrono::DateTime;
use chrono::Utc;
use derive_more::AsRef;
use indexmap::IndexMap;
use itertools::Itertools;
use p256::SecretKey;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use tracing::debug;
use tracing::warn;

use crypto::x509::CertificateUsage;
use dcql::ClaimPath;
use dcql::CredentialQueryFormat;
use dcql::normalized::AttributeRequest;
use dcql::normalized::NormalizedCredentialRequest;
use http_utils::urls::HttpsUri;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;

use crate::Error;
use crate::Result;
use crate::identifiers::AttributeIdentifier;
use crate::identifiers::AttributeIdentifierHolder;
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
    #[error("attributes mismatch: {0:?}")]
    MissingAttributes(Vec<AttributeIdentifier>),
    #[error("unexpected amount of CA Common Names in issuer certificate: expected 1, found {0}")]
    UnexpectedCACommonNameCount(usize),
    #[error("issuer URI {0} not found in SAN {1:?}")]
    IssuerUriNotFoundInSan(HttpsUri, VecNonEmpty<HttpsUri>),
    #[error("missing issuer URI")]
    MissingIssuerUri,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, AsRef)]
pub struct ItemsRequests(pub Vec<ItemsRequest>);
impl From<Vec<ItemsRequest>> for ItemsRequests {
    fn from(value: Vec<ItemsRequest>) -> Self {
        Self(value)
    }
}

impl ItemsRequests {
    /// Checks that all `requested` attributes are disclosed in this [`DeviceResponse`].
    pub fn match_against_response(&self, device_response: &DeviceResponse) -> Result<()> {
        let not_found: Vec<_> = self
            .0
            .iter()
            .flat_map(|items_request| {
                device_response
                    .documents
                    .as_ref()
                    .and_then(|docs| docs.iter().find(|doc| doc.doc_type == items_request.doc_type))
                    .map_or_else(
                        // If the entire document is missing then all requested attributes are missing
                        || Ok::<_, Error>(items_request.mdoc_attribute_identifiers()?.into_iter().collect_vec()),
                        |doc| Ok(items_request.match_against_issuer_signed(doc)?),
                    )
            })
            .flatten()
            .collect_vec();

        if not_found.is_empty() {
            Ok(())
        } else {
            Err(VerificationError::MissingAttributes(not_found).into())
        }
    }
}

// TODO: Remove in PVW-4530
impl From<ItemsRequest> for NormalizedCredentialRequest {
    fn from(source: ItemsRequest) -> Self {
        let doctype_value = source.doc_type;

        let format = CredentialQueryFormat::MsoMdoc { doctype_value };

        // unwrap below is safe because claims path is not empty
        let claims = source
            .name_spaces
            .into_iter()
            .flat_map(|(namespace, attrs)| {
                attrs
                    .into_iter()
                    .map(move |(attribute, intent_to_retain)| AttributeRequest {
                        path: vec![
                            ClaimPath::SelectByKey(namespace.clone()),
                            ClaimPath::SelectByKey(attribute.clone()),
                        ]
                        .try_into()
                        .unwrap(),
                        intent_to_retain,
                    })
            })
            .collect();

        NormalizedCredentialRequest { format, claims }
    }
}

impl From<ItemsRequests> for VecNonEmpty<NormalizedCredentialRequest> {
    fn from(source: ItemsRequests) -> Self {
        source
            .0
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
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
    use rstest::rstest;

    use crypto::examples::Examples;
    use crypto::server_keys::generate::Ca;
    use wscd::mock_remote::MockRemoteEcdsaKey;

    use crate::DeviceAuthenticationBytes;
    use crate::DeviceResponse;
    use crate::Document;
    use crate::Error;
    use crate::ValidityInfo;
    use crate::examples::EXAMPLE_ATTR_NAME;
    use crate::examples::EXAMPLE_ATTR_VALUE;
    use crate::examples::EXAMPLE_DOC_TYPE;
    use crate::examples::EXAMPLE_NAMESPACE;
    use crate::examples::Example;
    use crate::examples::IsoCertTimeGenerator;
    use crate::examples::example_items_requests;
    use crate::holder::Mdoc;
    use crate::identifiers::AttributeIdentifierHolder;
    use crate::test;
    use crate::test::DebugCollapseBts;
    use crate::test::data::addr_street;
    use crate::test::data::pid_full_name;

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

    #[rstest]
    #[case(do_nothing())]
    #[case(swap_attributes())]
    #[case(remove_documents())]
    #[case(remove_document())]
    #[case(change_doctype())]
    #[case(change_namespace())]
    #[case(remove_attribute())]
    #[case(multiple_doc_types_swapped())]
    fn match_disclosed_attributes(
        #[case] testcase: (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>),
    ) {
        // Construct an items request that matches the example device response
        let (device_response, items_requests, expected_result) = testcase;
        assert_eq!(
            items_requests
                .match_against_response(&device_response)
                .map_err(|e| match e {
                    Error::Verification(VerificationError::MissingAttributes(e)) => e,
                    _ => panic!(),
                }),
            expected_result,
        );
    }

    /// Helper to compute all attribute identifiers contained in a bunch of [`ItemsRequest`]s.
    fn attribute_identifiers(items_requests: &ItemsRequests) -> Vec<AttributeIdentifier> {
        items_requests
            .0
            .iter()
            .flat_map(|request| request.mdoc_attribute_identifiers().unwrap())
            .collect()
    }

    // return an unmodified device response, which should verify
    fn do_nothing() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        (DeviceResponse::example(), example_items_requests(), Ok(()))
    }

    // Matching attributes is insensitive to swapped attributes, so verification succeeds
    fn swap_attributes() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_first_attributes(|attributes| {
            attributes.swap(0, 1);
        });

        (device_response, example_items_requests(), Ok(()))
    }

    // remove all disclosed documents
    fn remove_documents() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents = None;

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // remove a single disclosed document
    fn remove_document() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response.documents.as_mut().unwrap().pop();

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Change the first doctype so it is not the requested one
    fn change_doctype() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        device_response
            .documents
            .as_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .doc_type = "some_not_requested_doc_type".to_string();

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Change a namespace so it is not the requested one
    fn change_namespace() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_namespaces(|name_spaces| {
            let (_, attributes) = name_spaces.pop().unwrap();
            name_spaces.insert("some_not_requested_name_space".to_string(), attributes);
        });

        let items_requests = example_items_requests();
        let missing = attribute_identifiers(&items_requests);
        (device_response, items_requests, Err(missing))
    }

    // Remove one of the disclosed attributes
    fn remove_attribute() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let first_document = device_response.documents.as_mut().unwrap().first_mut().unwrap();
        let name_spaces = first_document.issuer_signed.name_spaces.as_mut().unwrap();

        name_spaces.modify_first_attributes(|attributes| {
            attributes.pop();
        });

        let items_requests = example_items_requests();
        let missing = vec![attribute_identifiers(&items_requests).last().unwrap().clone()];
        (device_response, items_requests, Err(missing))
    }

    // Add one extra document with doc_type "a", and swap the order in the items_requests
    fn multiple_doc_types_swapped() -> (DeviceResponse, ItemsRequests, Result<(), Vec<AttributeIdentifier>>) {
        let mut device_response = DeviceResponse::example();
        let mut cloned_doc: Document = device_response.documents.as_ref().unwrap()[0].clone();
        cloned_doc.doc_type = "a".to_string();
        device_response.documents.as_mut().unwrap().push(cloned_doc);

        let mut items_requests = example_items_requests();
        let mut cloned_items_request = items_requests.0[0].clone();
        cloned_items_request.doc_type = "a".to_string();
        items_requests.0.push(cloned_items_request);

        // swap the document order in items_requests
        items_requests.0.reverse();

        (device_response, items_requests, Ok(()))
    }

    #[rstest]
    #[case(
        pid_full_name().into_first().unwrap().into(),
        NormalizedCredentialRequest::pid_full_name(),
    )]
    #[case(
        addr_street().into_first().unwrap().into(),
        NormalizedCredentialRequest::addr_street(),
    )]
    fn items_requests_to_and_from_credential_requests(
        #[case] input: ItemsRequest,
        #[case] expected: NormalizedCredentialRequest,
    ) {
        let actual: NormalizedCredentialRequest = input.into();
        assert_eq!(actual, expected);
    }
}
