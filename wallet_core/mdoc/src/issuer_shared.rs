use std::fmt::Display;

use coset::Header;
use serde_bytes::ByteBuf;

use crate::{
    basic_sa_ext::{
        KeyGenerationResponseMessage, MdocResponses, RequestKeyGenerationMessage, Response, ResponseSignaturePayload,
        UnsignedMdoc,
    },
    utils::{
        cose::{ClonePayload, CoseKey, MdocCose},
        crypto::random_string,
        serialization::cbor_serialize,
        signer::SecureEcdsaKey,
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
}

/// Identifies an issuance session in a URL, as passed from the issuer to the holder using the `url` field of
/// [`iso::ServiceEngagement`]).
/// This token is distict from [`iso::SessionId`] because the `ServiceEngagement` may be transmitted over an insecure
/// channel (e.g. a QR code). By not using the `SessionId` for this, the issuer transmits this to the holder in response
/// to its first HTTPS request, so that it remains secret between them. Since in later protocol messages the issuer
/// enforces that the correct session ID is present, this means that only the party that sends the first HTTP request
/// can send later HTTP requests for the session.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(pub(crate) String);

impl SessionToken {
    pub fn new() -> Self {
        random_string(32).into()
    }
}

impl From<String> for SessionToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Response {
    fn sign(challenge: &ByteBuf, key: &impl SecureEcdsaKey) -> Result<Response> {
        let response = Response {
            public_key: CoseKey::try_from(
                &key.verifying_key()
                    .map_err(|e| IssuanceError::PrivatePublicKeyConversion(e.into()))?,
            )?,
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

    pub fn new(
        request: &RequestKeyGenerationMessage,
        keys: &[Vec<impl SecureEcdsaKey>],
    ) -> Result<KeyGenerationResponseMessage> {
        let responses = keys
            .iter()
            .zip(&request.unsigned_mdocs)
            .map(|(keys, unsigned)| Self::create_responses(keys, unsigned, &request.challenge))
            .collect::<Result<Vec<MdocResponses>>>()?;

        let response = KeyGenerationResponseMessage {
            e_session_id: request.e_session_id.clone(),
            mdoc_responses: responses,
        };
        Ok(response)
    }

    fn create_responses(
        keys: &[impl SecureEcdsaKey],
        unsigned: &UnsignedMdoc,
        challenge: &ByteBuf,
    ) -> Result<MdocResponses> {
        let responses = MdocResponses {
            doc_type: unsigned.doc_type.clone(),
            responses: keys
                .iter()
                .map(|key| Response::sign(challenge, key))
                .collect::<Result<Vec<_>>>()?,
        };
        Ok(responses)
    }
}
