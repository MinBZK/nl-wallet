use anyhow::{bail, Result};
use coset::{iana, CoseMac0Builder, CoseSign1Builder, HeaderBuilder};
use ecdsa::SigningKey;
use ecdsa::{elliptic_curve::rand_core::OsRng, signature::Signer};
use indexmap::IndexMap;
use p256::NistP256;
use x509_parser::nom::AsBytes;
use x509_parser::prelude::X509Certificate;

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, Entry, KeyGenerationResponseMessage, RequestKeyGenerationMessage, SparseIssuerSigned,
        UnsignedMdoc,
    },
    cose::ClonePayload,
    crypto::dh_hmac_key,
    iso::*,
    serialization::{cbor_serialize, TaggedBytes},
    verifier::X509Subject,
};

#[derive(Debug, Clone)]
pub struct Credentials(pub IndexMap<DocType, Credential>);

impl<const N: usize> From<[Credential; N]> for Credentials {
    fn from(m: [Credential; N]) -> Self {
        Credentials(IndexMap::from_iter(
            m.into_iter().map(move |cred| (cred.doc_type.clone(), cred)),
        ))
    }
}

impl Entry {
    fn to_issuer_signed_item(&self, index: usize, random: Vec<u8>) -> IssuerSignedItemBytes {
        IssuerSignedItem {
            digest_id: index as u32,
            random,
            element_identifier: self.name.clone(),
            element_value: self.value.clone(),
        }
        .into()
    }
}

impl SparseIssuerSigned {
    fn to_credential(
        &self,
        private_key: SigningKey<p256::NistP256>,
        unsigned: &UnsignedMdoc,
        doc_type: DocType,
        iss_cert: &X509Certificate,
    ) -> Result<Credential> {
        let name_spaces: IssuerNameSpaces = unsigned
            .attributes
            .iter()
            .map(|(namespace, attrs)| {
                (
                    namespace.clone(),
                    attrs
                        .iter()
                        .enumerate()
                        .map(|(index, attr)| attr.to_issuer_signed_item(index, self.randoms[namespace][index].to_vec()))
                        .collect::<Vec<_>>()
                        .into(),
                )
            })
            .collect();

        let mso = MobileSecurityObject {
            version: self.sparse_issuer_auth.version.clone(),
            digest_algorithm: self.sparse_issuer_auth.digest_algorithm.clone(),
            value_digests: (&name_spaces).try_into()?,
            device_key_info: private_key.verifying_key().try_into()?,
            doc_type: doc_type.clone(),
            validity_info: self.sparse_issuer_auth.validity_info.clone(),
        };
        let issuer_auth = self
            .sparse_issuer_auth
            .issuer_auth
            .clone_with_payload(cbor_serialize(&TaggedBytes::from(mso)).map_err(anyhow::Error::msg)?);

        let issuer_signed = IssuerSigned {
            name_spaces: Some(name_spaces),
            issuer_auth,
        };
        issuer_signed.verify(iss_cert)?;

        Ok(Credential {
            private_key,
            issuer_signed,
            doc_type,
        })
    }
}

#[derive(Debug)]
pub struct IssuanceState<'a> {
    pub request: &'a RequestKeyGenerationMessage,
    pub private_keys: IndexMap<String, Vec<SigningKey<NistP256>>>,
    pub response: KeyGenerationResponseMessage,
}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials(IndexMap::new())
    }

    pub fn add(&mut self, creds: IndexMap<DocType, Vec<Credential>>) {
        for (doc_type, creds) in creds {
            for cred in creds {
                self.0.insert(doc_type.clone(), cred);
            }
        }
    }

    fn generate_keys(count: u64) -> Vec<SigningKey<p256::NistP256>> {
        (0..count)
            .map(|_| SigningKey::<p256::NistP256>::random(OsRng))
            .collect()
    }

    pub fn issuance_start(request: &RequestKeyGenerationMessage) -> Result<IssuanceState> {
        let private_keys = request
            .unsigned_mdocs
            .iter()
            .map(|(doc_type, unsigned_mdoc)| (doc_type.clone(), Credentials::generate_keys(unsigned_mdoc.count)))
            .collect();
        let response =
            KeyGenerationResponseMessage::new(request.e_session_id.clone(), request.challenge.clone(), &private_keys)?;

        Ok(IssuanceState {
            request,
            private_keys,
            response,
        })
    }

    pub fn issuance_finish(
        state: IssuanceState,
        issuer_response: DataToIssueMessage,
        issuer_cert: &X509Certificate,
    ) -> Result<IndexMap<DocType, Vec<Credential>>> {
        issuer_response
            .mobile_id_documents
            .iter()
            .map(|(doc_type, iss_signature)| {
                Ok((
                    doc_type.clone(),
                    iss_signature
                        .iter()
                        .enumerate()
                        .map(|(i, iss_signature)| {
                            iss_signature.to_credential(
                                state.private_keys[doc_type][i].clone(),
                                &state.request.unsigned_mdocs[doc_type],
                                doc_type.clone(),
                                issuer_cert,
                            )
                        })
                        .collect::<Result<_>>()?,
                ))
            })
            .collect()
    }

    pub fn disclose(&self, device_request: &DeviceRequest, challenge: &[u8]) -> Result<DeviceResponse> {
        let mut docs: Vec<Document> = Vec::new();

        for doc_request in &device_request.doc_requests {
            let items_request = &doc_request.items_request.0;
            if !self.0.contains_key(&items_request.doc_type) {
                bail!(
                    "unsatisfiable request: DocType {} not in wallet",
                    &items_request.doc_type
                )
            }

            let cred = &self.0[&items_request.doc_type];
            docs.push(cred.disclose_document(items_request, challenge)?);
        }

        Ok(DeviceResponse {
            version: "1.0".to_string(),
            documents: Some(docs),
            document_errors: None,
            status: 0,
        })
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Credential {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    issuer_signed: IssuerSigned,
    doc_type: String,
}

impl Credential {
    pub fn new(
        private_key: ecdsa::SigningKey<p256::NistP256>,
        issuer_signed: IssuerSigned,
        ca_cert: &X509Certificate,
    ) -> Result<Credential> {
        let (_, mso) = issuer_signed.verify(ca_cert)?;
        Ok(Credential {
            private_key,
            issuer_signed,
            doc_type: mso.doc_type,
        })
    }

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

        Ok(Document {
            doc_type: items_request.doc_type.clone(),
            issuer_signed: IssuerSigned {
                name_spaces: Some(disclosed_namespaces),
                issuer_auth: self.issuer_signed.issuer_auth.clone(),
            },
            device_signed: DeviceSigned::new_signature(&self.private_key, challenge),
            errors: None,
        })
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

impl DeviceSigned {
    pub(crate) fn new_signature(private_key: &ecdsa::SigningKey<p256::NistP256>, challenge: &[u8]) -> DeviceSigned {
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
    pub(crate) fn new_mac(
        private_key: &ecdsa::SigningKey<p256::NistP256>,
        reader_pub_key: &ecdsa::VerifyingKey<p256::NistP256>,
        challenge: &[u8],
    ) -> Result<DeviceSigned> {
        let device_auth: DeviceAuthenticationBytes = ciborium::de::from_reader(challenge)?;
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

        Ok(DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceMac(cose.into()),
        })
    }
}

impl DeviceRequest {
    /// Verify reader authentication, if present.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the
    /// the DocRequests to be signed by distinct readers. TODO maybe support this.
    /// For now, this function requires either none of the DocRequests to be signed, or all of them
    /// by the same reader.
    #[allow(dead_code)] // TODO use in client
    pub(crate) fn verify(
        &self,
        ca_cert: &X509Certificate,
        reader_authentication_bts: &[u8],
    ) -> Result<Option<X509Subject>> {
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            bail!("readerAuth not present for all documents")
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
                bail!("document requests were signed by different readers")
            }
        }

        Ok(reader)
    }
}
