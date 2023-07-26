use std::marker::PhantomData;

use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::{
    serialization::{Base64Bytes, DerSignature},
    signing_key::{EphemeralEcdsaKey, SecureEcdsaKey},
};
use crate::errors::{Error, Result, SigningError, ValidationError};

// Signed data by the wallet, either with both the hardware and PIN keys, or just the hardware key.
// They are generic over the data type that they contain, so that the signed data type is encoded in the type structure
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeResponsePayload<T> {
    pub payload: T,
    pub challenge: Base64Bytes,
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

fn verify_signed(signed: &str, challenge: &[u8], typ: SignedType, pubkey: &VerifyingKey) -> Result<()> {
    let msg: SignedMessage<&RawValue> = serde_json::from_str(signed)?;
    let json = msg.signed.get().as_bytes();
    pubkey.verify(json, &msg.signature.0).map_err(ValidationError::from)?;

    if msg.typ != typ {
        return Err(Error::TypeMismatch {
            expected: typ,
            received: msg.typ,
        });
    }

    let signed: ChallengeResponsePayload<&RawValue> = serde_json::from_slice(json)?;
    if challenge != signed.challenge.0 {
        return Err(Error::ChallengeMismatch);
    }

    Ok(())
}

fn sign_inner<T: Serialize>(
    payload: T,
    challenge: &[u8],
    serial_number: u64,
    typ: SignedType,
    privkey: &impl Signer<Signature>,
) -> Result<SignedInner<T>> {
    let message = serde_json::to_string(&ChallengeResponsePayload {
        payload: &payload,
        challenge: challenge.to_vec().into(),
        sequence_number: serial_number,
    })?;
    Ok(sign_message(message, typ, privkey)?.into())
}

fn sign_message(message: String, typ: SignedType, privkey: &impl Signer<Signature>) -> Result<String> {
    let signature = privkey.try_sign(message.as_bytes()).map_err(SigningError::from)?.into();
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
    fn verify(&self, challenge: &[u8], hw_pubkey: &VerifyingKey, pin_pubkey: &VerifyingKey) -> Result<()> {
        let outer: SignedMessage<&RawValue> = serde_json::from_str(&self.0)?;
        hw_pubkey
            .verify(outer.signed.get().as_bytes(), &outer.signature.0)
            .map_err(ValidationError::from)?;
        verify_signed(outer.signed.get(), challenge, SignedType::Pin, pin_pubkey)
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
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<ChallengeResponsePayload<T>> {
        self.verify(challenge, hw_pubkey, pin_pubkey)?;
        self.dangerous_parse_unverified()
    }

    pub fn sign(
        payload: T,
        challenge: &[u8],
        serial_number: u64,
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
    ) -> Result<SignedDouble<T>> {
        let inner = sign_inner(payload, challenge, serial_number, SignedType::Pin, pin_privkey)?.0;
        let signed_message = sign_message(inner, SignedType::HW, hw_privkey)?;
        Ok(signed_message.into())
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
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
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

    #[test]
    fn double_signed() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = SignedDouble::sign(ToyMessage::default(), challenge, 1337, &hw_privkey, &pin_privkey).unwrap();
        println!("{}", signed.0);

        let verified = signed
            .parse_and_verify(challenge, hw_privkey.verifying_key(), pin_privkey.verifying_key())
            .unwrap();

        dbg!(verified);
    }
}
