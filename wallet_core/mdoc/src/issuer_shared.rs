//! Data types shared between the issuer and the holder.

use coset::Header;
use futures::future::try_join_all;
use serde_bytes::ByteBuf;

use wallet_common::keys::SecureEcdsaKey;

use crate::{
    basic_sa_ext::{
        KeyGenerationResponseMessage, MdocResponses, RequestKeyGenerationMessage, Response, ResponseSignaturePayload,
        UnsignedMdoc,
    },
    server_state::SessionToken,
    utils::{
        cose::{ClonePayload, CoseKey, MdocCose},
        serialization::cbor_serialize,
    },
    DocType, Result, SessionId,
};

#[derive(thiserror::Error, Debug)]
pub enum IssuanceError {
    #[error("missing session ID")]
    MissingSessionId,
    #[error("session IDs did not match: received {}, expected {}", received, expected)]
    MismatchedSessionIds { received: SessionId, expected: SessionId },
    #[error("received too many responses: {received}, max was {max}")]
    TooManyResponses { received: u64, max: u64 },
    #[error("received response for wrong doctype: {received}, expected {expected}")]
    WrongDocType { received: DocType, expected: DocType },
    #[error("unknown session ID: {0}")]
    UnknownSessionId(SessionToken),
    #[error("cannot process holder input: session has ended")]
    SessionEnded,
    #[error("unexpected message type: {received}, expected {expected}")]
    UnexpectedMessageType { received: String, expected: String },
    #[error("missing private key for doctype {0}")]
    MissingPrivateKey(DocType),
    #[error("failed to get public key from private key: {0}")]
    PrivatePublicKeyConversion(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("failed to parse DER-encoded private key: {0}")]
    DerPrivateKey(#[from] p256::pkcs8::Error),
}

impl Response {
    async fn sign(challenge: &ByteBuf, key: &(impl SecureEcdsaKey + Sync)) -> Result<Response> {
        let response = Response {
            public_key: CoseKey::try_from(
                &key.verifying_key()
                    .await
                    .map_err(|e| IssuanceError::PrivatePublicKeyConversion(e.into()))?,
            )?,
            signature: MdocCose::sign(
                &ResponseSignaturePayload::new(challenge.to_vec()),
                Header::default(),
                key,
                false,
            )
            .await?,
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

impl MdocResponses {
    async fn sign(
        keys: &[impl SecureEcdsaKey + Sync],
        unsigned: &UnsignedMdoc,
        challenge: &ByteBuf,
    ) -> Result<MdocResponses> {
        let responses = try_join_all(keys.iter().map(|key| Response::sign(challenge, key))).await?;

        let mdoc_responses = MdocResponses {
            doc_type: unsigned.doc_type.clone(),
            responses,
        };
        Ok(mdoc_responses)
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
            .find_map(|(responses, unsigned_mdoc)| {
                Self::check_responses(responses, unsigned_mdoc, &request.challenge).err()
            })
            .map_or(Ok(()), Err)
    }

    fn check_responses(
        doctype_responses: &MdocResponses,
        unsigned_mdoc: &UnsignedMdoc,
        challenge: &ByteBuf,
    ) -> Result<()> {
        if doctype_responses.responses.len() as u64 > unsigned_mdoc.copy_count {
            return Err(IssuanceError::TooManyResponses {
                received: doctype_responses.responses.len() as u64,
                max: unsigned_mdoc.copy_count,
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

    pub async fn new(
        request: &RequestKeyGenerationMessage,
        keys: &[&[impl SecureEcdsaKey + Sync]],
    ) -> Result<KeyGenerationResponseMessage> {
        let mdoc_responses = try_join_all(
            keys.iter()
                .zip(&request.unsigned_mdocs)
                .map(|(keys, unsigned)| MdocResponses::sign(keys, unsigned, &request.challenge)),
        )
        .await?;

        let response = KeyGenerationResponseMessage {
            e_session_id: request.e_session_id.clone(),
            mdoc_responses,
        };
        Ok(response)
    }
}
