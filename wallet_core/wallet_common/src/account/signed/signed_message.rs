use std::borrow::Cow;

use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::keys::EcdsaKey;

use super::{
    super::{
        errors::{Error, Result},
        serialization::DerSignature,
    },
    raw_value::TypedRawValue,
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
pub(super) struct SignedMessage<T> {
    signed: TypedRawValue<T>,
    r#type: SignedType,
    signature: DerSignature,
}

impl<T> SignedMessage<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn sign<K>(payload: &T, r#type: SignedType, signing_key: &K) -> Result<Self>
    where
        K: EcdsaKey,
    {
        let signed = TypedRawValue::try_new(payload).map_err(Error::JsonParsing)?;
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

    pub fn dangerous_parse_unverified(&self) -> Result<T> {
        let value = self.signed.parse().map_err(Error::JsonParsing)?;

        Ok(value)
    }

    pub fn parse_and_verify(&self, r#type: SignedType, verifying_key: &VerifyingKey) -> Result<T> {
        verifying_key
            .verify(self.signed.as_ref(), &self.signature.0)
            .map_err(Error::Verification)?;

        if self.r#type != r#type {
            return Err(Error::TypeMismatch {
                expected: r#type,
                received: self.r#type,
            });
        }

        self.dangerous_parse_unverified()
    }
}

pub(super) trait SubjectPayload {
    const SUBJECT: &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct PayloadWithSubject<T> {
    subject: Cow<'static, str>,
    #[serde(flatten)]
    payload: T,
}

impl<T> PayloadWithSubject<T>
where
    T: SubjectPayload,
{
    fn new(payload: T) -> Self {
        Self {
            subject: Cow::Borrowed(T::SUBJECT),
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SignedSubjectMessage<T>(SignedMessage<PayloadWithSubject<T>>);

/// Same as [`SignedMessage`], but adds a subject string to the signed JSON object, the contents of which is verified.
impl<T> SignedSubjectMessage<T>
where
    T: SubjectPayload + Serialize + DeserializeOwned,
{
    pub async fn sign<K>(payload: T, r#type: SignedType, signing_key: &K) -> Result<Self>
    where
        K: EcdsaKey,
    {
        let signed_message = SignedMessage::sign(&PayloadWithSubject::new(payload), r#type, signing_key).await?;

        Ok(Self(signed_message))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<T> {
        let payload = self.0.dangerous_parse_unverified()?.payload;

        Ok(payload)
    }

    pub fn parse_and_verify(&self, r#type: SignedType, verifying_key: &VerifyingKey) -> Result<T> {
        let payload = self.0.parse_and_verify(r#type, verifying_key)?;

        if payload.subject.as_ref() != T::SUBJECT {
            return Err(Error::SubjectMismatch {
                expected: T::SUBJECT.to_string(),
                received: payload.subject.into_owned(),
            });
        }

        Ok(payload.payload)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct ToyPayload {
        string: String,
    }
    impl Default for ToyPayload {
        fn default() -> Self {
            Self {
                string: "Some payload.".to_string(),
            }
        }
    }
    impl SubjectPayload for ToyPayload {
        const SUBJECT: &'static str = "toy_subject";
    }

    #[tokio::test]
    async fn test_signed_message() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedMessage::sign(&ToyPayload::default(), SignedType::HW, &key)
            .await
            .unwrap();

        let payload = signed_message
            .parse_and_verify(SignedType::HW, key.verifying_key())
            .expect("should parse and verify SignedMessage successfully");

        assert_eq!(payload.string, "Some payload.");
    }

    #[tokio::test]
    async fn test_signed_message_error_type_mismatch() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedMessage::sign(&ToyPayload::default(), SignedType::HW, &key)
            .await
            .unwrap();

        // Verifying with the wrong signature type should return a `Error::TypeMismatch`.
        let error = signed_message
            .parse_and_verify(SignedType::Pin, key.verifying_key())
            .expect_err("verifying SignedMessage should return an error");

        assert_matches!(
            error,
            Error::TypeMismatch {
                expected: SignedType::Pin,
                received: SignedType::HW
            }
        );
    }

    #[tokio::test]
    async fn test_subject_signed_message() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedSubjectMessage::sign(ToyPayload::default(), SignedType::HW, &key)
            .await
            .unwrap();

        let payload = signed_message
            .parse_and_verify(SignedType::HW, key.verifying_key())
            .expect("should parse and verify SignedSubjectMessage successfully");

        assert_eq!(payload.string, "Some payload.");
    }

    #[tokio::test]
    async fn test_subject_signed_message_error_subject_mismatch() {
        #[derive(Debug, Serialize, Deserialize)]
        struct WrongToyPayload {
            string: String,
        }
        impl SubjectPayload for WrongToyPayload {
            const SUBJECT: &'static str = "wrong_subject";
        }

        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedSubjectMessage::sign(
            WrongToyPayload {
                string: "WRONG!".to_string(),
            },
            SignedType::HW,
            &key,
        )
        .await
        .unwrap();

        let decoded_message =
            serde_json::from_str::<SignedSubjectMessage<ToyPayload>>(&serde_json::to_string(&signed_message).unwrap())
                .unwrap();

        // Verifying with an incorrect subject field should return a `Error::SubjectMismatch`.
        let error = decoded_message
            .parse_and_verify(SignedType::HW, key.verifying_key())
            .expect_err("verifying SignedMessage should return an error");

        assert_matches!(
            error,
            Error::SubjectMismatch {
                expected,
                received,
            } if expected == "toy_subject" && received == "wrong_subject"
        );
    }
}
