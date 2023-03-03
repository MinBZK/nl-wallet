use std::marker::PhantomData;

use anyhow::{bail, Ok, Result};
use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::{
    serialization::{Base64Bytes, DerSignature},
    wallet::pin_key::PinKey,
};

#[derive(Debug)]
pub struct SignedDouble<T>(pub String, PhantomData<T>);
#[derive(Debug)]
pub struct Signed<T>(pub String, PhantomData<T>);

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage<T> {
    pub signed: T,
    pub signature: DerSignature,
    #[serde(rename = "type")]
    pub typ: SignedType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedPayload<T> {
    pub payload: T,
    pub challenge: Base64Bytes,
    pub serial_number: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SignedType {
    Pin,
    HW,
}

fn verify_signed(
    signed: &str,
    challenge: &[u8],
    typ: SignedType,
    pubkey: &VerifyingKey,
) -> Result<()> {
    let msg: SignedMessage<&RawValue> = serde_json::from_str(signed)?;
    let json = msg.signed.get().as_bytes();
    pubkey.verify(json, &msg.signature.0)?;

    if msg.typ != typ {
        bail!("incorrect type")
    }

    let signed: SignedPayload<&RawValue> = serde_json::from_slice(json)?;
    if challenge != signed.challenge.0 {
        bail!("incorrect challenge")
    }

    Ok(())
}

fn sign<T: Serialize>(
    payload: T,
    challenge: &[u8],
    serial_number: u64,
    typ: SignedType,
    privkey: &impl Signer<Signature>,
) -> Result<Signed<T>> {
    let signed = serde_json::to_string(&SignedPayload {
        payload: &payload,
        challenge: challenge.to_vec().into(),
        serial_number,
    })?;
    let signature = privkey.try_sign(signed.as_bytes())?.into();
    Ok(serde_json::to_string(&SignedMessage {
        signed: &RawValue::from_string(signed)?,
        signature,
        typ,
    })?
    .into())
}

impl<'de, T> Signed<T>
where
    T: Serialize + Deserialize<'de>,
{
    /// Value of the `typ` field of [`SignedMessage<T>`].
    const TYP: SignedType = SignedType::HW;

    fn verify(&self, challenge: &[u8], pubkey: &VerifyingKey) -> Result<()> {
        verify_signed(&self.0, challenge, Signed::<T>::TYP, pubkey)
    }

    fn dangerous_parse_unverified(&'de self) -> Result<SignedPayload<T>> {
        Ok(serde_json::from_str::<SignedMessage<SignedPayload<T>>>(&self.0)?.signed)
    }

    pub fn parse_and_verify(
        &'de self,
        challenge: &[u8],
        pubkey: &VerifyingKey,
    ) -> Result<SignedPayload<T>> {
        self.verify(challenge, pubkey)?;
        self.dangerous_parse_unverified()
    }

    pub fn sign(
        payload: T,
        challenge: &[u8],
        serial_number: u64,
        privkey: &impl Signer<Signature>,
    ) -> Result<Signed<T>> {
        sign(payload, challenge, serial_number, Signed::<T>::TYP, privkey)
    }
}

impl<'de, T> SignedDouble<T>
where
    T: Serialize + Deserialize<'de>,
{
    fn verify(
        &self,
        challenge: &[u8],
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<()> {
        let outer: SignedMessage<&RawValue> = serde_json::from_str(&self.0)?;
        hw_pubkey.verify(outer.signed.get().as_bytes(), &outer.signature.0)?;
        verify_signed(outer.signed.get(), challenge, SignedType::Pin, pin_pubkey)
    }

    pub fn dangerous_parse_unverified(&'de self) -> Result<SignedPayload<T>> {
        Ok(
            serde_json::from_str::<SignedMessage<SignedMessage<SignedPayload<T>>>>(&self.0)?
                .signed
                .signed,
        )
    }

    pub fn parse_and_verify(
        &'de self,
        challenge: &[u8],
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<SignedPayload<T>> {
        self.verify(challenge, hw_pubkey, pin_pubkey)?;
        self.dangerous_parse_unverified()
    }

    pub fn sign(
        payload: T,
        challenge: &[u8],
        serial_number: u64,
        hw_privkey: &impl Signer<Signature>,
        pin: &str,
        salt: &[u8],
    ) -> Result<SignedDouble<T>> {
        let inner = sign(
            payload,
            challenge,
            serial_number,
            SignedType::Pin,
            &PinKey { pin, salt },
        )?
        .0;
        let signature = hw_privkey.try_sign(inner.as_bytes())?;
        Ok(serde_json::to_string(&SignedMessage {
            signed: RawValue::from_string(inner)?,
            signature: signature.into(),
            typ: SignedType::HW,
        })?
        .into())
    }
}

impl<T, S: Into<String>> From<S> for SignedDouble<T> {
    fn from(val: S) -> Self {
        SignedDouble(val.into(), PhantomData)
    }
}
impl<T, S: Into<String>> From<S> for Signed<T> {
    fn from(val: S) -> Self {
        Signed(val.into(), PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
    use serde::{Deserialize, Serialize};

    use crate::wallet::{pin_key::PinKey, signed::SignedDouble};

    use super::Signed;

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
    fn hw_signed() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);

        let signed = Signed::sign(ToyMessage::default(), challenge, 1337, &hw_privkey).unwrap();
        println!("{}", signed.0);

        let verified = signed
            .parse_and_verify(challenge, hw_privkey.verifying_key())
            .unwrap();

        dbg!(verified);
    }

    #[test]
    fn double_signed() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin = "123456";
        let salt = &[1, 2, 3, 4][..];

        let signed = SignedDouble::sign(
            ToyMessage::default(),
            challenge,
            1337,
            &hw_privkey,
            pin,
            salt,
        )
        .unwrap();
        println!("{}", signed.0);

        let verified = signed
            .parse_and_verify(
                challenge,
                hw_privkey.verifying_key(),
                &PinKey { salt, pin }.verifying_key(),
            )
            .unwrap();

        dbg!(verified);
    }
}
