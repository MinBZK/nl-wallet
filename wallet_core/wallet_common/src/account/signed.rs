use std::marker::PhantomData;

use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_with::{base64::Base64, serde_as};

use crate::keys::{EcdsaKey, EphemeralEcdsaKey, SecureEcdsaKey};

use super::{
    errors::{Error, Result},
    serialization::DerSignature,
};

// Signed data by the wallet, with both the hardware and PIN keys.
// It is generic over the data type that it contains, so that the signed data type is encoded in the type structure
// of users of `SignedDouble<T>`, and so that all methods of `SignedDouble<T>` for verification and deserialization
// also have access to the same type `T`. Instead of containing T directly, however, `SignedDouble<T>` wraps strings
// containing a JSON-serialized version of T, because that stores not only the data itself but also the order of the
// JSON maps. We need that information for the signature verification, but it would be lost as soon as we
// JSON-deserialize the data. We use `PhantomData<T>` to prevent the compiler from complaining about `T` being unused.
#[derive(Debug)]
pub struct SignedDouble<T>(pub String, PhantomData<T>);
#[derive(Debug)]
pub(crate) struct SignedInner<T>(pub String, PhantomData<T>);

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage<T> {
    pub signed: T,
    pub signature: DerSignature,
    #[serde(rename = "type")]
    pub typ: SignedType,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeResponsePayload<T> {
    pub payload: T,
    #[serde_as(as = "Base64")]
    pub challenge: Vec<u8>,
    pub sequence_number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedPayload<T> {
    pub payload: T,
    pub issuer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SignedType {
    Pin,
    HW,
}

#[derive(Debug, Clone, Copy)]
pub enum SequenceNumberComparison {
    EqualTo(u64),
    LargerThan(u64),
}

impl SequenceNumberComparison {
    pub fn verify(&self, expected_sequence_number: u64) -> bool {
        match self {
            SequenceNumberComparison::EqualTo(sequence_number) => expected_sequence_number == *sequence_number,
            SequenceNumberComparison::LargerThan(sequence_number) => expected_sequence_number > *sequence_number,
        }
    }
}

fn parse_and_verify_message<'a>(
    signed: &'a str,
    typ: SignedType,
    pubkey: &VerifyingKey,
) -> Result<SignedMessage<&'a RawValue>> {
    let message: SignedMessage<&RawValue> = serde_json::from_str(signed)?;
    let json = message.signed.get().as_bytes();
    pubkey.verify(json, &message.signature.0).map_err(Error::Ecdsa)?;

    if message.typ != typ {
        return Err(Error::TypeMismatch {
            expected: typ,
            received: message.typ,
        });
    }

    Ok(message)
}

async fn sign_message(message: String, typ: SignedType, privkey: &impl EcdsaKey) -> Result<String> {
    let signature = privkey
        .try_sign(message.as_bytes())
        .await
        .map_err(|err| Error::Signing(Box::new(err)))?
        .into();
    Ok(serde_json::to_string(&SignedMessage {
        signed: &RawValue::from_string(message)?,
        signature,
        typ,
    })?)
}

impl<'de, T> SignedDouble<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn verify(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<()> {
        let outer = parse_and_verify_message(&self.0, SignedType::HW, hw_pubkey)?;
        let inner = parse_and_verify_message(outer.signed.get(), SignedType::Pin, pin_pubkey)?;

        let signed: ChallengeResponsePayload<&RawValue> = serde_json::from_str(inner.signed.get())?;

        if challenge != signed.challenge {
            return Err(Error::ChallengeMismatch);
        }

        if !sequence_number_comparison.verify(signed.sequence_number) {
            return Err(Error::SequenceNumberMismatch);
        }

        Ok(())
    }

    pub fn dangerous_parse_unverified(&'de self) -> Result<ChallengeResponsePayload<T>> {
        let payload = serde_json::from_str::<SignedMessage<SignedMessage<ChallengeResponsePayload<T>>>>(&self.0)?
            .signed
            .signed;
        Ok(payload)
    }

    pub fn parse_and_verify(
        &'de self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<ChallengeResponsePayload<T>> {
        self.verify(challenge, sequence_number_comparison, hw_pubkey, pin_pubkey)?;
        self.dangerous_parse_unverified()
    }

    pub async fn sign(
        payload: T,
        challenge: &[u8],
        serial_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
    ) -> Result<SignedDouble<T>> {
        let message = serde_json::to_string(&ChallengeResponsePayload {
            payload: &payload,
            challenge: challenge.to_vec(),
            sequence_number: serial_number,
        })?;
        let signed_inner = sign_message(message, SignedType::Pin, pin_privkey).await?;
        let signed_double = sign_message(signed_inner, SignedType::HW, hw_privkey).await?;
        Ok(signed_double.into())
    }
}

impl<T, S: Into<String>> From<S> for SignedDouble<T> {
    fn from(val: S) -> Self {
        SignedDouble(val.into(), PhantomData)
    }
}
impl<T, S: Into<String>> From<S> for SignedInner<T> {
    fn from(val: S) -> Self {
        SignedInner(val.into(), PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ToyMessage {
        number: u8,
        string: String,
    }
    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                number: 42,
                string: "Hello, world!".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn double_signed() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = SignedDouble::sign(ToyMessage::default(), challenge, 1337, &hw_privkey, &pin_privkey)
            .await
            .unwrap();
        println!("{}", signed.0);

        let verified = signed
            .parse_and_verify(
                challenge,
                SequenceNumberComparison::LargerThan(1336),
                hw_privkey.verifying_key(),
                pin_privkey.verifying_key(),
            )
            .unwrap();

        assert_eq!(ToyMessage::default(), verified.payload);
    }
}
