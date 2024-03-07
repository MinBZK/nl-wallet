//! Data types shared between the issuer and the holder.

use coset::Header;
use futures::future;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        KeyGenerationResponseMessage, MdocResponses, RequestKeyGenerationMessage, Response, ResponseSignaturePayload,
        UnsignedMdoc,
    },
    server_state::SessionToken,
    utils::{
        cose::{ClonePayload, CoseKey, MdocCose},
        keys::{KeyFactory, MdocEcdsaKey},
        serialization::cbor_serialize,
    },
    DocType, Error, Result, SessionId,
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
    #[error("error with sessionstore: {0}")]
    SessionStore(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Response {
    async fn generate_keys_and_sign<K: MdocEcdsaKey>(
        challenge: &ByteBuf,
        number_of_keys: u64,
        key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<Vec<(K, Response)>> {
        let payload = ResponseSignaturePayload::new(challenge.to_vec());

        let coses: Vec<(K, MdocCose<_, _>)> =
            MdocCose::generate_keys_and_sign(&payload, Header::default(), number_of_keys, key_factory, false).await?;

        let responses: Vec<(K, Response)> = future::try_join_all(coses.into_iter().map(|(key, signature)| async {
            let verifying_key = key
                .verifying_key()
                .await
                .map_err(|e| IssuanceError::PrivatePublicKeyConversion(e.into()))?;

            let response =
                CoseKey::try_from(&verifying_key).map(|public_key| (key, Response { public_key, signature }))?;
            Ok::<(K, Response), Error>(response)
        }))
        .await?;

        Ok(responses)
    }

    pub fn verify(&self, challenge: &ByteBuf) -> Result<()> {
        let expected_payload = &ResponseSignaturePayload::new(challenge.to_vec());
        self.signature
            .clone_with_payload(cbor_serialize(&expected_payload)?)
            .verify(&(&self.public_key).try_into()?)?;
        Ok(())
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

    pub async fn new<K: MdocEcdsaKey>(
        request: &RequestKeyGenerationMessage,
        key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<(Vec<Vec<K>>, KeyGenerationResponseMessage)> {
        let keys_count: u64 = request.unsigned_mdocs.iter().map(|unsigned| unsigned.copy_count).sum();

        let mut keys_and_responses =
            Response::generate_keys_and_sign(&request.challenge, keys_count, key_factory).await?;

        let keys_and_mdoc_responses: Vec<(_, _)> = request
            .unsigned_mdocs
            .iter()
            .map(|unsigned| {
                let (keys, responses) = keys_and_responses.drain(..unsigned.copy_count as usize).unzip();

                let mdoc_responses = MdocResponses {
                    doc_type: unsigned.doc_type.clone(),
                    responses,
                };

                (keys, mdoc_responses)
            })
            .collect();

        assert!(
            keys_and_responses.is_empty(),
            "all keys_and_responses items should have been converted"
        );

        let empty_response = KeyGenerationResponseMessage {
            e_session_id: request.e_session_id.clone(),
            mdoc_responses: vec![],
        };

        let response = keys_and_mdoc_responses.into_iter().fold(
            (vec![], empty_response),
            |(mut acc_keys, mut response), (keys, mdoc_responses)| {
                response.mdoc_responses.push(mdoc_responses);
                acc_keys.push(keys);
                (acc_keys, response)
            },
        );

        Ok(response)
    }
}
