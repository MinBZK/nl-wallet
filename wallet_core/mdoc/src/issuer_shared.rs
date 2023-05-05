use coset::Header;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        KeyGenerationResponseMessage, MdocResponses, RequestKeyGenerationMessage, Response, ResponseSignaturePayload,
        UnsignedMdoc,
    },
    cose::{ClonePayload, CoseKey, MdocCose},
    serialization::cbor_serialize,
    DocType, Result, SessionId,
};

#[derive(thiserror::Error, Debug)]
pub enum IssuanceError {
    #[error(
        "session IDs did not match: received {}, expected {}",
        hex::encode(received),
        hex::encode(expected)
    )]
    MismatchedSessionIds { received: SessionId, expected: SessionId },
    #[error("received too many responses: {received}, max was {max}")]
    TooManyResponses { received: u64, max: u64 },
    #[error("received response for wrong doctype: {received}, expected {expected}")]
    WrongDocType { received: DocType, expected: DocType },
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

impl KeyGenerationResponseMessage {
    pub fn verify(&self, request: &RequestKeyGenerationMessage) -> Result<()> {
        if self.e_session_id != request.e_session_id {
            return Err(IssuanceError::MismatchedSessionIds {
                received: self.e_session_id.clone(),
                expected: request.e_session_id.clone(),
            }
            .into());
        }

        self.mdoc_responses
            .iter()
            .zip(&request.unsigned_mdocs)
            .find_map(|(responses, unsigned_mdoc)| check_responses(responses, unsigned_mdoc, &request.challenge).err())
            .map_or(Ok(()), Err)
    }

    pub fn new(
        request: &RequestKeyGenerationMessage,
        keys: &[Vec<ecdsa::SigningKey<p256::NistP256>>],
    ) -> Result<KeyGenerationResponseMessage> {
        let responses = keys
            .iter()
            .zip(&request.unsigned_mdocs)
            .map(|(keys, unsigned)| {
                Ok(MdocResponses {
                    doc_type: unsigned.doc_type.clone(),
                    responses: keys
                        .iter()
                        .map(|key| Response::sign(&request.challenge, key))
                        .collect::<Result<Vec<_>>>()?,
                })
            })
            .collect::<Result<Vec<MdocResponses>>>()?;

        let response = KeyGenerationResponseMessage {
            e_session_id: request.e_session_id.clone(),
            mdoc_responses: responses,
        };
        Ok(response)
    }
}

fn check_responses(doctype_responses: &MdocResponses, unsigned_mdoc: &UnsignedMdoc, challenge: &ByteBuf) -> Result<()> {
    if doctype_responses.responses.len() as u64 > unsigned_mdoc.count {
        return Err(IssuanceError::TooManyResponses {
            received: doctype_responses.responses.len() as u64,
            max: unsigned_mdoc.count,
        }
        .into());
    }
    if doctype_responses.doc_type != unsigned_mdoc.doc_type {
        return Err(IssuanceError::WrongDocType {
            received: doctype_responses.doc_type.clone(),
            expected: unsigned_mdoc.doc_type.clone(),
        }
        .into());
    }

    doctype_responses
        .responses
        .iter()
        .find_map(|response| response.verify(challenge).err())
        .map_or(Ok(()), Err)
}
