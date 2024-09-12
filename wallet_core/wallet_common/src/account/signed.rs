use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use crate::keys::{EcdsaKey, EphemeralEcdsaKey, SecureEcdsaKey};

use super::{
    errors::{Error, Result},
    raw_value::TypedRawValue,
    serialization::DerSignature,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignedType {
    Pin,
    HW,
}

/// Wraps an arbitrary payload that can be represented as a byte slice and includes a signature and signature type. Its
/// data can be serialized and deserialized, while maintaining a stable string representation. This is necessary, as
/// JSON representation is not deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedMessage<T> {
    signed: TypedRawValue<T>,
    r#type: SignedType,
    signature: DerSignature,
}

impl<T> SignedMessage<T>
where
    T: Serialize + DeserializeOwned,
{
    async fn sign<K>(payload: &T, r#type: SignedType, signing_key: &K) -> Result<Self>
    where
        K: EcdsaKey,
    {
        let signed = TypedRawValue::try_new(payload)?;
        let signature = signing_key
            .try_sign(signed.as_ref())
            .await
            .map_err(|err| Error::Signing(Box::new(err)))?
            .into();

        let signed_message = SignedMessage {
            signed,
            r#type,
            signature,
        };

        Ok(signed_message)
    }

    fn dangerous_parse_unverified(&self) -> Result<T> {
        let value = self.signed.parse()?;

        Ok(value)
    }

    fn parse_and_verify(&self, r#type: SignedType, verifying_key: &VerifyingKey) -> Result<T> {
        verifying_key.verify(self.signed.as_ref(), &self.signature.0)?;

        if self.r#type != r#type {
            return Err(Error::TypeMismatch {
                expected: r#type,
                received: self.r#type,
            });
        }

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChallengeRequestPayload {
    pub sequence_number: u64,
}

impl ChallengeRequestPayload {
    pub fn new(sequence_number: u64) -> Self {
        Self { sequence_number }
    }

    pub fn verify(&self, sequence_number_comparison: SequenceNumberComparison) -> Result<()> {
        if !sequence_number_comparison.verify(self.sequence_number) {
            return Err(Error::SequenceNumberMismatch);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeRequest(SignedMessage<ChallengeRequestPayload>);

impl ChallengeRequest {
    pub async fn sign<K>(sequence_number: u64, hardware_signing_key: &K) -> Result<Self>
    where
        K: SecureEcdsaKey,
    {
        let challenge_request = ChallengeRequestPayload::new(sequence_number);
        let signed = SignedMessage::sign(&challenge_request, SignedType::HW, hardware_signing_key).await?;

        Ok(Self(signed))
    }

    pub fn parse_and_verify(
        &self,
        sequence_number_comparison: SequenceNumberComparison,
        hardware_verifying_key: &VerifyingKey,
    ) -> Result<ChallengeRequestPayload> {
        let request = self.0.parse_and_verify(SignedType::HW, hardware_verifying_key)?;
        request.verify(sequence_number_comparison)?;

        Ok(request)
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponsePayload<T> {
    pub payload: T,
    #[serde_as(as = "Base64")]
    pub challenge: Vec<u8>,
    pub sequence_number: u64,
}

impl<T> ChallengeResponsePayload<T> {
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
pub struct ChallengeResponse<T>(SignedMessage<SignedMessage<ChallengeResponsePayload<T>>>);

impl<T> ChallengeResponse<T>
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
        let challenge_response = ChallengeResponsePayload {
            payload,
            challenge,
            sequence_number,
        };
        let inner_signed = SignedMessage::sign(&challenge_response, SignedType::Pin, pin_signing_key).await?;
        let outer_signed = SignedMessage::sign(&inner_signed, SignedType::HW, hardware_signing_key).await?;

        Ok(Self(outer_signed))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<ChallengeResponsePayload<T>> {
        let challenge_response = self.0.dangerous_parse_unverified()?.dangerous_parse_unverified()?;

        Ok(challenge_response)
    }

    pub fn parse_and_verify(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        hardware_verifying_key: &VerifyingKey,
        pin_verifying_key: &VerifyingKey,
    ) -> Result<ChallengeResponsePayload<T>> {
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
    async fn test_challenge_request() {
        let hw_privkey = SigningKey::random(&mut OsRng);

        let signed = ChallengeRequest::sign(42, &hw_privkey).await.unwrap();

        println!("{}", serde_json::to_string(&signed).unwrap());

        let request = signed
            .parse_and_verify(SequenceNumberComparison::EqualTo(42), hw_privkey.verifying_key())
            .expect("SignedChallengeRequest should be valid");

        assert_eq!(request.sequence_number, 42);
    }

    #[tokio::test]
    async fn test_challenge_response() {
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = ChallengeResponse::sign(
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
            .expect("SignedChallengeResponse should be valid");

        assert_eq!(ToyMessage::default(), verified.payload);
    }
}
