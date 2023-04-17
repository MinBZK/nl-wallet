use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, Header, HeaderBuilder};
use indexmap::IndexMap;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, KeyGenerationResponseMessage, MobileIDDocuments, RequestKeyGenerationMessage, Response,
        ResponseSignaturePayload, Responses, SparseIssuerAuth, SparseIssuerSigned, UnsignedMdoc,
    },
    cose::{ClonePayload, CoseKey, MdocCose},
    iso::*,
    serialization::{cbor_serialize, TaggedBytes},
};

pub struct Issuer {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    cert_bts: Vec<u8>,
    request: RequestKeyGenerationMessage,
}

impl Issuer {
    pub fn new(
        request: RequestKeyGenerationMessage,
        private_key: ecdsa::SigningKey<p256::NistP256>,
        cert_bts: Vec<u8>,
    ) -> Issuer {
        Issuer {
            request,
            cert_bts,
            private_key,
        }
    }

    fn issue_cred(
        &self,
        doc_type: DocType,
        unsigned_mdoc: &UnsignedMdoc,
        response: &Response,
    ) -> Result<SparseIssuerSigned> {
        let attrs = unsigned_mdoc
            .attributes
            .clone()
            .into_iter()
            .map(|(namespace, attrs)| Ok((namespace, Attributes::try_from(attrs)?)))
            .collect::<Result<IssuerNameSpaces>>()?;

        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: now.into(),
            valid_until: unsigned_mdoc.valid_until.clone(),
            expected_update: None,
        };
        let mso = MobileSecurityObject {
            version: "1.0".to_string(),
            digest_algorithm: "SHA-256".to_string(),
            doc_type,
            value_digests: (&attrs).try_into()?,
            device_key_info: response.public_key.clone().into(),
            validity_info: validity.clone(),
        };

        let headers = HeaderBuilder::new()
            .value(33, Value::Bytes(self.cert_bts.clone()))
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso.into(), headers, &self.private_key)?;

        Ok(SparseIssuerSigned {
            randoms: attrs
                .into_iter()
                .map(|(namespace, attrs)| {
                    (
                        namespace,
                        attrs.0.into_iter().map(|attr| ByteBuf::from(attr.0.random)).collect(),
                    )
                })
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: "1.0".to_string(),
                digest_algorithm: "SHA-256".to_string(),
                validity_info: validity,
                issuer_auth: cose.clone_without_payload(),
            },
        })
    }

    fn issue_creds(&self, doc_type: &DocType, responses: &[Response]) -> Result<Vec<SparseIssuerSigned>> {
        responses
            .iter()
            .map(|response| self.issue_cred(doc_type.clone(), &self.request.unsigned_mdocs[doc_type], response))
            .collect()
    }

    pub fn issue(self, device_response: &KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        device_response.verify(&self.request)?;

        let docs = device_response
            .responses
            .iter()
            .map(|(doc_type, responses)| Ok((doc_type.clone(), self.issue_creds(doc_type, responses)?)))
            .collect::<Result<MobileIDDocuments>>()?;

        Ok(DataToIssueMessage {
            e_session_id: self.request.e_session_id,
            mobile_id_documents: docs,
        })
    }
}

impl KeyGenerationResponseMessage {
    pub fn verify(&self, request: &RequestKeyGenerationMessage) -> Result<()> {
        if self.e_session_id != request.e_session_id {
            bail!("session IDs did not match")
        }

        request
            .unsigned_mdocs
            .iter()
            .find_map(|(doc_type, unsigned_mdoc)| {
                check_responses(self.responses.get(doc_type), unsigned_mdoc, &request.challenge).err()
            })
            .map_or(Ok(()), Err)
    }

    pub fn new(
        session_id: SessionId,
        challenge: ByteBuf,
        keys: &IndexMap<DocType, Vec<ecdsa::SigningKey<p256::NistP256>>>,
    ) -> anyhow::Result<KeyGenerationResponseMessage> {
        let responses = keys
            .iter()
            .map(|(doc_type, keys)| {
                Ok((
                    doc_type.clone(),
                    keys.iter()
                        .map(|key| Response::sign(&challenge, key))
                        .collect::<Result<Vec<_>>>()?,
                ))
            })
            .collect::<Result<Responses>>()?;

        let response = KeyGenerationResponseMessage {
            e_session_id: session_id,
            responses,
        };
        Ok(response)
    }
}

fn check_responses(responses: Option<&Vec<Response>>, unsigned_mdoc: &UnsignedMdoc, challenge: &ByteBuf) -> Result<()> {
    let responses = responses.ok_or(anyhow!("response not found"))?;
    if responses.len() as u64 > unsigned_mdoc.count {
        bail!("too many responses")
    }

    responses
        .iter()
        .find_map(|response| response.verify(challenge).err())
        .map_or(Ok(()), Err)
}

impl Response {
    fn sign(challenge: &ByteBuf, key: &ecdsa::SigningKey<p256::NistP256>) -> Result<Response> {
        let response = Response {
            public_key: CoseKey::try_from(&key.verifying_key())?,
            signature: MdocCose::sign(
                &ResponseSignaturePayload::new(challenge.to_vec()),
                Header::default(),
                key,
            )?
            .clone_without_payload(),
        };
        Ok(response)
    }

    pub fn verify(&self, challenge: &ByteBuf) -> Result<()> {
        let expected_payload = &ResponseSignaturePayload::new(challenge.to_vec());
        self.signature
            .clone_with_payload(cbor_serialize(&expected_payload)?)
            .verify(&(&self.public_key).try_into()?)
    }
}
