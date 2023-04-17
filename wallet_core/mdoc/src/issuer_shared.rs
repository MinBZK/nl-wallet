use anyhow::{bail, Result};
use coset::Header;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        DocTypeResponses, KeyGenerationResponseMessage, RequestKeyGenerationMessage, Response,
        ResponseSignaturePayload, UnsignedMdoc,
    },
    cose::{ClonePayload, CoseKey, MdocCose},
    serialization::cbor_serialize,
};

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

impl KeyGenerationResponseMessage {
    pub fn verify(&self, request: &RequestKeyGenerationMessage) -> Result<()> {
        if self.e_session_id != request.e_session_id {
            bail!("session IDs did not match")
        }

        self.doc_type_responses
            .iter()
            .zip(&request.unsigned_mdocs)
            .find_map(|(responses, unsigned_mdoc)| check_responses(responses, unsigned_mdoc, &request.challenge).err())
            .map_or(Ok(()), Err)
    }

    pub fn new(
        request: &RequestKeyGenerationMessage,
        keys: &[Vec<ecdsa::SigningKey<p256::NistP256>>],
    ) -> anyhow::Result<KeyGenerationResponseMessage> {
        let responses = keys
            .iter()
            .zip(&request.unsigned_mdocs)
            .map(|(keys, unsigned)| {
                Ok(DocTypeResponses {
                    doc_type: unsigned.doc_type.clone(),
                    responses: keys
                        .iter()
                        .map(|key| Response::sign(&request.challenge, key))
                        .collect::<Result<Vec<_>>>()?,
                })
            })
            .collect::<Result<Vec<DocTypeResponses>>>()?;

        let response = KeyGenerationResponseMessage {
            e_session_id: request.e_session_id.clone(),
            doc_type_responses: responses,
        };
        Ok(response)
    }
}

fn check_responses(
    doctype_responses: &DocTypeResponses,
    unsigned_mdoc: &UnsignedMdoc,
    challenge: &ByteBuf,
) -> Result<()> {
    if doctype_responses.responses.len() as u64 > unsigned_mdoc.count {
        bail!("too many responses")
    }
    if doctype_responses.doc_type != unsigned_mdoc.doc_type {
        bail!("wrong doctype")
    }

    doctype_responses
        .responses
        .iter()
        .find_map(|response| response.verify(challenge).err())
        .map_or(Ok(()), Err)
}
