use std::marker::PhantomData;

use anyhow::{bail, Context, Ok, Result};
use p256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::serialization::{Base64Bytes, DerSignature};

use super::pin_key::PinKey;

/// A payload signed by the wallet, both with its PIN key and its hardware-bound key.
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletSignedMessage<T> {
    pub hw_signed: T,
    pub hw_signature: DerSignature,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletSignedInner<T> {
    pub pin_signed: T,
    pub pin_signature: DerSignature,
}

#[derive(Debug)]
pub struct WalletSigned<T>(pub String, PhantomData<T>);
impl<T> From<String> for WalletSigned<T> {
    fn from(val: String) -> Self {
        WalletSigned(val, PhantomData)
    }
}

/// Signed data within a [`WalletSigned<T>`].
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletSignedPayload<T> {
    pub payload: T,
    pub challenge: Base64Bytes,
    pub serial_number: u64,
}

impl<'de, T> WalletSigned<T>
where
    T: Serialize + Deserialize<'de>,
{
    pub fn verify(
        &self,
        challenge: &[u8],
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<WalletSignedPayload<&RawValue>> {
        let outer: WalletSignedMessage<&RawValue> = serde_json::from_str(&self.0)?;
        hw_pubkey.verify(outer.hw_signed.get().as_bytes(), &outer.hw_signature.0)?;

        let inner: WalletSignedInner<&RawValue> = serde_json::from_str(outer.hw_signed.get())?;
        pin_pubkey.verify(inner.pin_signed.get().as_bytes(), &inner.pin_signature.0)?;

        let signed: WalletSignedPayload<&RawValue> =
            serde_json::from_slice(inner.pin_signed.get().as_bytes())?;
        if challenge != &signed.challenge.0 {
            bail!("incorrect challenge")
        }

        Ok(signed)
    }

    pub fn dangerous_parse_unverified(&'de self) -> Result<WalletSignedPayload<T>> {
        Ok(
            serde_json::from_str::<WalletSignedMessage<WalletSignedInner<WalletSignedPayload<T>>>>(
                &self.0,
            )?
            .hw_signed
            .pin_signed,
        )
    }

    pub fn parse_and_verify(
        &'de self,
        challenge: &[u8],
        hw_pubkey: &VerifyingKey,
        pin_pubkey: &VerifyingKey,
    ) -> Result<WalletSignedPayload<T>> {
        let signed = self.verify(challenge, hw_pubkey, pin_pubkey)?;
        Ok(WalletSignedPayload {
            payload: serde_json::from_str(signed.payload.get())
                .context("payload deserialization failed")?,
            challenge: signed.challenge,
            serial_number: signed.serial_number,
        })
    }

    pub fn sign(
        payload: T,
        challenge: &[u8],
        serial_number: u64,
        hw_privkey: &impl Signer<Signature>,
        pin: &str,
        salt: &[u8],
    ) -> Result<WalletSigned<T>> {
        let pin_signed = serde_json::to_string(&WalletSignedPayload {
            payload: &payload,
            challenge: challenge.to_vec().into(),
            serial_number,
        })?;
        let pin_signature = PinKey { pin, salt }.try_sign(pin_signed.as_bytes())?;

        // Create inner (pin) signature
        let hw_signed = WalletSignedInner {
            pin_signed: &RawValue::from_string(pin_signed)?,
            pin_signature: pin_signature.into(),
        };
        let hw_signature = hw_privkey.try_sign(&serde_json::to_vec(&hw_signed)?)?;

        Ok(serde_json::to_string(&WalletSignedMessage {
            hw_signed,
            hw_signature: hw_signature.into(),
        })?
        .into())
    }
}

#[cfg(test)]
mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
    use serde::{Deserialize, Serialize};

    use crate::wallet::{pin_key::PinKey, signed::WalletSigned, HWBoundSigningKey};

    #[test]
    fn it_works() {
        #[derive(Serialize, Deserialize, Debug)]
        struct Message {
            number: u8,
            string: String,
        }
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin = "123456";
        let salt = &[1, 2, 3, 4][..];

        let signed = WalletSigned::sign(
            Message {
                number: 42,
                string: "Hello, world!".to_string(),
            },
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
                &hw_privkey.verifying_key(),
                &PinKey { salt, pin }.verifying_key(),
            )
            .unwrap();

        dbg!(verified);
    }
}
