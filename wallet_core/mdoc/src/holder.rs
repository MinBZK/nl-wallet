use crate::cose::ClonePayload;
use crate::crypto::dh_hmac_key;
use crate::iso::*;
use crate::issuer::IssuanceDeviceResponse;

use anyhow::{bail, Result};
use coset::{iana, CoseMac0Builder, CoseSign1Builder, HeaderBuilder};
use ecdsa::{elliptic_curve::rand_core::OsRng, signature::Signer};
use indexmap::IndexMap;
use x509_parser::nom::AsBytes;
use x509_parser::prelude::X509Certificate;

#[derive(Debug, Clone)]
pub struct Credentials(pub IndexMap<DocType, Credential>);

impl<const N: usize> From<[Credential; N]> for Credentials {
    fn from(m: [Credential; N]) -> Self {
        Credentials(IndexMap::from_iter(
            m.into_iter().map(move |cred| (cred.doc_type.clone(), cred)),
        ))
    }
}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials(IndexMap::new())
    }

    pub fn add(&mut self, cred: Credential) {
        self.0.insert(cred.doc_type.clone(), cred);
    }

    pub fn start_issuance(
        &self,
        challenge: &[u8],
    ) -> Result<(ecdsa::SigningKey<p256::NistP256>, IssuanceDeviceResponse)> {
        let device_key = ecdsa::SigningKey::<p256::NistP256>::random(&mut OsRng);
        let response = IssuanceDeviceResponse::sign(challenge, &device_key)?;
        Ok((device_key, response))
    }

    pub fn disclose(
        &self,
        device_request: &DeviceRequest,
        challenge: &[u8],
    ) -> Result<DeviceResponse> {
        let mut docs: Vec<Document> = Vec::new();

        for doc_request in &device_request.doc_requests {
            let items_request = &doc_request.items_request.0;
            if !self.0.contains_key(&items_request.doc_type) {
                bail!(
                    "unsatisfiable request: DocType {} not in wallet",
                    &items_request.doc_type
                )
            }

            let cred = self.0.get(&items_request.doc_type).unwrap();
            docs.push(cred.disclose_document(&items_request, challenge)?);
        }

        Ok(DeviceResponse {
            version: "1.0".to_string(),
            documents: Some(docs),
            document_errors: None,
            status: 0,
        })
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
        let (_, mso) = issuer_signed.verify(&ca_cert)?;
        Ok(Credential {
            private_key,
            issuer_signed,
            doc_type: mso.doc_type,
        })
    }

    pub fn disclose_document(
        &self,
        items_request: &ItemsRequest,
        challenge: &[u8],
    ) -> Result<Document> {
        let disclosed_namespaces: IssuerNameSpaces = self
            .issuer_signed
            .name_spaces
            .as_ref()
            .unwrap()
            .iter()
            .filter(|&(namespace, _)| items_request.name_spaces.contains_key(namespace))
            .map(|(namespace, issuer_signed)| {
                (
                    namespace.clone(),
                    Attributes(
                        issuer_signed
                            .0
                            .clone()
                            .into_iter()
                            .filter(|i| {
                                items_request
                                    .name_spaces
                                    .get(namespace)
                                    .unwrap()
                                    .contains_key(&i.0.element_identifier)
                            })
                            .collect(),
                    ),
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

impl DeviceSigned {
    pub(crate) fn new_signature(
        private_key: &ecdsa::SigningKey<p256::NistP256>,
        challenge: &[u8],
    ) -> DeviceSigned {
        let cose = CoseSign1Builder::new()
            .payload(Vec::from(challenge))
            .protected(
                HeaderBuilder::new()
                    .algorithm(iana::Algorithm::ES256)
                    .build(),
            )
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
            .protected(
                HeaderBuilder::new()
                    .algorithm(iana::Algorithm::ES256)
                    .build(),
            )
            .create_tag(&[], |data| ring::hmac::sign(&key, data).as_ref().into())
            .build()
            .clone_without_payload();

        Ok(DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceMac(cose.into()),
        })
    }
}
