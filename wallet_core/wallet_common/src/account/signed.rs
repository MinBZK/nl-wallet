use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use crate::keys::{EcdsaKey, EphemeralEcdsaKey, SecureEcdsaKey};

use super::{
    errors::{Error, Result},
    serialization::DerSignature,
};

use raw_value::TypedRawValue;

mod raw_value {
    use std::marker::PhantomData;

    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::value::RawValue;

    /// Wraps a [`RawValue`], which internally holds a string slice Next to this, the type it serializes from and
    /// deserializes to is held using [`PhantomData`]. It is to be used as a helper type for JSON structs, where a
    /// signature needs to be generated over an exact piece of JSON string data.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct TypedRawValue<T>(Box<RawValue>, PhantomData<T>);

    impl<T> AsRef<[u8]> for TypedRawValue<T> {
        fn as_ref(&self) -> &[u8] {
            self.0.get().as_bytes()
        }
    }

    impl<T> TypedRawValue<T> {
        pub fn try_new(value: &T) -> Result<Self, serde_json::Error>
        where
            T: Serialize,
        {
            let json = serde_json::to_string(value)?;
            let raw_value = RawValue::from_string(json)?;

            Ok(Self(raw_value, PhantomData))
        }

        pub fn parse(&self) -> Result<T, serde_json::Error>
        where
            T: DeserializeOwned,
        {
            serde_json::from_str(self.0.get())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignedType {
    Pin,
    HW,
}

/// Wraps an arbitrary payload that can be represented as a byte slice and includes a signature and signature type. If
/// the payload uses [`TypedRawValue`], its data can be serialized and deserialized, while maintaining a stable string
/// representation. This is necessary, as JSON representation is not deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedMessage<T> {
    signed: T,
    r#type: SignedType,
    signature: DerSignature,
}

type RawValueSignedMessage<T> = SignedMessage<TypedRawValue<T>>;

impl<T> SignedMessage<T>
where
    T: AsRef<[u8]>,
{
    async fn sign<K>(payload: T, r#type: SignedType, signing_key: &K) -> Result<Self>
    where
        K: EcdsaKey,
    {
        let signature = signing_key
            .try_sign(payload.as_ref())
            .await
            .map_err(|err| Error::Signing(Box::new(err)))?
            .into();

        let signed_message = SignedMessage {
            signed: payload,
            r#type,
            signature,
        };

        Ok(signed_message)
    }

    fn verify(&self, r#type: SignedType, verifying_key: &VerifyingKey) -> Result<()> {
        verifying_key.verify(self.signed.as_ref(), &self.signature.0)?;

        if self.r#type != r#type {
            return Err(Error::TypeMismatch {
                expected: r#type,
                received: self.r#type,
            });
        }

        Ok(())
    }
}

impl<T> RawValueSignedMessage<T>
where
    T: Serialize + DeserializeOwned,
{
    async fn sign_raw<K>(payload: &T, r#type: SignedType, signing_key: &K) -> Result<Self>
    where
        K: EcdsaKey,
    {
        let raw_payload = TypedRawValue::try_new(payload)?;

        Self::sign(raw_payload, r#type, signing_key).await
    }

    fn dangerous_parse_unverified(&self) -> Result<T> {
        let value = self.signed.parse()?;

        Ok(value)
    }

    fn parse_and_verify(&self, r#type: SignedType, verifying_key: &VerifyingKey) -> Result<T> {
        self.verify(r#type, verifying_key)?;

        self.dangerous_parse_unverified()
    }
}

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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse<T> {
    pub payload: T,
    #[serde_as(as = "Base64")]
    pub challenge: Vec<u8>,
    pub sequence_number: u64,
}

impl<T> ChallengeResponse<T> {
    pub fn verify(&self, challenge: &[u8], sequence_number_comparison: SequenceNumberComparison) -> Result<()> {
        if challenge != self.challenge {
            return Err(Error::ChallengeMismatch);
        }

        if !sequence_number_comparison.verify(self.sequence_number) {
            return Err(Error::SequenceNumberMismatch);
        }

        Ok(())
    }
}

/// Wraps a [`ChallengeResponse`], which contains an arbitrary payload, and signs it with two keys. For the inner
/// signing the PIN key is used, while the outer signing is done with the device's hardware key.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedChallengeResponse<T>(RawValueSignedMessage<RawValueSignedMessage<ChallengeResponse<T>>>);

impl<T> SignedChallengeResponse<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn sign<HK, PK>(
        payload: T,
        challenge: Vec<u8>,
        sequence_number: u64,
        hardware_signing_key: &HK,
        pin_signing_key: &PK,
    ) -> Result<Self>
    where
        HK: SecureEcdsaKey,
        PK: EphemeralEcdsaKey,
    {
        let challenge_response = ChallengeResponse {
            payload,
            challenge,
            sequence_number,
        };
        let inner_signed =
            RawValueSignedMessage::sign_raw(&challenge_response, SignedType::Pin, pin_signing_key).await?;
        let outer_signed = RawValueSignedMessage::sign_raw(&inner_signed, SignedType::HW, hardware_signing_key).await?;

        Ok(Self(outer_signed))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<ChallengeResponse<T>> {
        let challenge_response = self.0.dangerous_parse_unverified()?.dangerous_parse_unverified()?;

        Ok(challenge_response)
    }

    pub fn parse_and_verify(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        hardware_verifying_key: &VerifyingKey,
        pin_verifying_key: &VerifyingKey,
    ) -> Result<ChallengeResponse<T>> {
        let inner_signed = self.0.parse_and_verify(SignedType::HW, hardware_verifying_key)?;
        let challenge_response = inner_signed.parse_and_verify(SignedType::Pin, pin_verifying_key)?;

        challenge_response.verify(challenge, sequence_number_comparison)?;

        Ok(challenge_response)
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    async fn test_signed_challenge_response() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = SignedChallengeResponse::sign(
            ToyMessage::default(),
            challenge.to_vec(),
            1337,
            &hw_privkey,
            &pin_privkey,
        )
        .await
        .unwrap();

        println!("{}", serde_json::to_string(&signed).unwrap());

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
