use std::borrow::Cow;

use p256::ecdsa::signature::Verifier;
use p256::ecdsa::Signature;
use p256::ecdsa::VerifyingKey;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use apple_app_attest::AppIdentifier;
use apple_app_attest::AssertionCounter;
use apple_app_attest::ClientData;
use apple_app_attest::VerifiedAssertion;
use wallet_common::apple::AppleAssertion;
use wallet_common::apple::AppleAttestedKey;
use wallet_common::keys::EcdsaKey;
use wallet_common::p256_der::DerSignature;

use super::super::errors::Error;
use super::super::errors::Result;
use super::raw_value::TypedRawValue;
use super::ContainsChallenge;
use super::EcdsaSignatureType;
use super::SignatureType;

/// Wraps both a type and a reference to the JSON data it was parsed from.
/// This is used internally in order to implement [`ClientData`] without
/// have to re-parse JSON multiple times.
struct ParsedValueWithSource<'a, T> {
    value: T,
    source: &'a [u8],
}

impl<T> ParsedValueWithSource<'_, T> {
    fn into_value(self) -> T {
        self.value
    }
}

impl<'a, T> TryFrom<&'a TypedRawValue<T>> for ParsedValueWithSource<'a, T>
where
    T: DeserializeOwned,
{
    type Error = Error;

    fn try_from(raw_value: &'a TypedRawValue<T>) -> Result<Self> {
        let value = raw_value.parse().map_err(Error::JsonParsing)?;

        let parsed = Self {
            value,
            source: raw_value.as_ref(),
        };

        Ok(parsed)
    }
}

impl<T> ClientData for ParsedValueWithSource<'_, T>
where
    T: ContainsChallenge,
{
    type Error = Error;

    fn hash_data(&self) -> Result<impl AsRef<[u8]>> {
        Ok(&self.source)
    }

    fn challenge(&self) -> Result<impl AsRef<[u8]>> {
        self.value.challenge()
    }
}

/// Wraps an arbitrary payload that can be represented as a byte slice and includes a signature and signature type. Its
/// data can be serialized and deserialized, while maintaining a stable string representation. This is necessary, as
/// JSON representation is not deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SignedMessage<T> {
    signed: TypedRawValue<T>,
    #[serde(flatten)]
    signature: MessageSignature,
}

/// Part of [`SignedMessage`] and represent the type and contents of the signature.
/// Contains several methods for converting to and from [`SignatureType`].
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum MessageSignature {
    Pin {
        #[serde_as(as = "Base64")]
        signature: DerSignature,
    },
    Google {
        #[serde_as(as = "Base64")]
        signature: DerSignature,
    },
    AppleAssertion {
        assertion: AppleAssertion,
    },
}

impl MessageSignature {
    fn new_ecdsa(r#type: EcdsaSignatureType, signature: Signature) -> Self {
        let signature = signature.into();

        match r#type {
            EcdsaSignatureType::Pin => Self::Pin { signature },
            EcdsaSignatureType::Google => Self::Google { signature },
        }
    }

    fn signature_type(&self) -> SignatureType {
        match self {
            Self::Pin { .. } => SignatureType::Ecdsa(EcdsaSignatureType::Pin),
            Self::Google { .. } => SignatureType::Ecdsa(EcdsaSignatureType::Google),
            Self::AppleAssertion { .. } => SignatureType::AppleAssertion,
        }
    }

    fn ecdsa_signature(&self, r#type: EcdsaSignatureType) -> Option<&Signature> {
        match (self, r#type) {
            (Self::Pin { signature }, EcdsaSignatureType::Pin)
            | (Self::Google { signature }, EcdsaSignatureType::Google) => Some(signature.as_inner()),
            _ => None,
        }
    }
}

impl<T> SignedMessage<T> {
    /// Create a [`SignedMessage`] containing an ECDSA signature, one of two subtypes.
    pub async fn sign_ecdsa<K>(payload: &T, r#type: EcdsaSignatureType, signing_key: &K) -> Result<Self>
    where
        T: Serialize,
        K: EcdsaKey,
    {
        let signed = TypedRawValue::try_new(payload).map_err(Error::JsonParsing)?;
        let ecdsa_signature = signing_key
            .try_sign(signed.as_ref())
            .await
            .map_err(|err| Error::Signing(Box::new(err)))?;
        let signature = MessageSignature::new_ecdsa(r#type, ecdsa_signature);

        let signed_message = SignedMessage { signed, signature };

        Ok(signed_message)
    }

    /// Create a [`SignedMessage`] containing an Apple assertion, using an attested key.
    pub async fn sign_apple<K>(payload: &T, attested_key: &K) -> Result<Self>
    where
        T: Serialize,
        K: AppleAttestedKey,
    {
        let signed = TypedRawValue::try_new(payload).map_err(Error::JsonParsing)?;
        let assertion = attested_key
            .sign(signed.as_ref().to_vec())
            .await
            .map_err(|err| Error::Signing(Box::new(err)))?;

        let signed_message = SignedMessage {
            signed,
            signature: MessageSignature::AppleAssertion { assertion },
        };

        Ok(signed_message)
    }

    /// Parse the payload of this message without verifying the signature.
    pub fn dangerous_parse_unverified(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let value = self.signed.parse().map_err(Error::JsonParsing)?;

        Ok(value)
    }

    /// Parse the payload of this message and verify its ECDSA signature.
    pub fn parse_and_verify_ecdsa(&self, r#type: EcdsaSignatureType, verifying_key: &VerifyingKey) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let signature = self
            .signature
            .ecdsa_signature(r#type)
            .ok_or_else(|| Error::SignatureTypeMismatch {
                expected: SignatureType::Ecdsa(r#type),
                received: self.signature.signature_type(),
            })?;

        verifying_key
            .verify(self.signed.as_ref(), signature)
            .map_err(Error::SignatureVerification)?;

        self.dangerous_parse_unverified()
    }

    /// Parse the payload of this message and verify its Apple assertion.
    pub fn parse_and_verify_apple(
        &self,
        verifying_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: AssertionCounter,
        expected_challenge: &[u8],
    ) -> Result<(T, AssertionCounter)>
    where
        T: DeserializeOwned + ContainsChallenge,
    {
        let parsed = ParsedValueWithSource::try_from(&self.signed)?;

        let MessageSignature::AppleAssertion { assertion } = &self.signature else {
            return Err(Error::SignatureTypeMismatch {
                expected: SignatureType::AppleAssertion,
                received: self.signature.signature_type(),
            });
        };

        let (_, counter) = VerifiedAssertion::parse_and_verify(
            assertion.as_ref(),
            &parsed,
            verifying_key,
            app_identifier,
            previous_counter,
            expected_challenge,
        )
        .map_err(Error::AssertionVerification)?;

        Ok((parsed.into_value(), counter))
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

impl<T> ContainsChallenge for PayloadWithSubject<T>
where
    T: ContainsChallenge,
{
    fn challenge(&self) -> Result<impl AsRef<[u8]>> {
        self.payload.challenge()
    }
}

/// Wraps a [`SignedMessage`] and adds a static subject field per concrete type.
/// All of the methods on the wrapped type are reproduced and forwarded, with
/// additional checking when appropriate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SignedSubjectMessage<T>(SignedMessage<PayloadWithSubject<T>>);

/// Same as [`SignedMessage`], but adds a subject string to the signed JSON object, the contents of which is verified.
impl<T> SignedSubjectMessage<T> {
    fn check_subject(payload: &PayloadWithSubject<T>) -> Result<()>
    where
        T: SubjectPayload,
    {
        if payload.subject.as_ref() != T::SUBJECT {
            return Err(Error::SubjectMismatch {
                expected: T::SUBJECT.to_string(),
                received: payload.subject.as_ref().to_string(),
            });
        }

        Ok(())
    }

    pub async fn sign_ecdsa<K>(payload: T, r#type: EcdsaSignatureType, signing_key: &K) -> Result<Self>
    where
        T: Serialize + SubjectPayload,
        K: EcdsaKey,
    {
        let signed_message = SignedMessage::sign_ecdsa(&PayloadWithSubject::new(payload), r#type, signing_key).await?;

        Ok(Self(signed_message))
    }

    pub async fn sign_apple<K>(payload: T, attested_key: &K) -> Result<Self>
    where
        T: Serialize + SubjectPayload,
        K: AppleAttestedKey,
    {
        let signed_message = SignedMessage::sign_apple(&PayloadWithSubject::new(payload), attested_key).await?;

        Ok(Self(signed_message))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let payload = self.0.dangerous_parse_unverified()?.payload;

        Ok(payload)
    }

    pub fn parse_and_verify_ecdsa(&self, r#type: EcdsaSignatureType, verifying_key: &VerifyingKey) -> Result<T>
    where
        T: DeserializeOwned + SubjectPayload,
    {
        let payload = self.0.parse_and_verify_ecdsa(r#type, verifying_key)?;

        Self::check_subject(&payload)?;

        Ok(payload.payload)
    }

    pub fn parse_and_verify_apple(
        &self,
        verifying_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: AssertionCounter,
        expected_challenge: &[u8],
    ) -> Result<(T, AssertionCounter)>
    where
        T: DeserializeOwned + ContainsChallenge + SubjectPayload,
    {
        let (payload, counter) =
            self.0
                .parse_and_verify_apple(verifying_key, app_identifier, previous_counter, expected_challenge)?;

        Self::check_subject(&payload)?;

        Ok((payload.payload, counter))
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;

    use wallet_common::apple::MockAppleAttestedKey;
    use wallet_common::utils;

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ToyPayload {
        string: String,
        challenge: Vec<u8>,
    }
    impl Default for ToyPayload {
        fn default() -> Self {
            Self {
                string: "Some payload.".to_string(),
                challenge: utils::random_bytes(32),
            }
        }
    }
    impl SubjectPayload for ToyPayload {
        const SUBJECT: &'static str = "toy_subject";
    }
    impl ContainsChallenge for ToyPayload {
        fn challenge(&self) -> Result<impl AsRef<[u8]>> {
            Ok(&self.challenge)
        }
    }

    fn create_mock_apple_attested_key() -> MockAppleAttestedKey {
        let app_identifier = AppIdentifier::new_mock();

        MockAppleAttestedKey::new_random(app_identifier)
    }

    #[tokio::test]
    async fn test_ecdsa_signed_message() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedMessage::sign_ecdsa(&ToyPayload::default(), EcdsaSignatureType::Google, &key)
            .await
            .expect("should sign message with ECDSA key");

        let payload = signed_message
            .parse_and_verify_ecdsa(EcdsaSignatureType::Google, key.verifying_key())
            .expect("should parse and verify SignedMessage successfully using its ECDSA signature");

        assert_eq!(payload.string, "Some payload.");
    }

    #[tokio::test]
    async fn test_apple_signed_message() {
        let key = create_mock_apple_attested_key();
        let input_payload = ToyPayload::default();
        let signed_message = SignedMessage::sign_apple(&input_payload, &key)
            .await
            .expect("should sign message with Apple attested key");

        let (output_payload, counter) = signed_message
            .parse_and_verify_apple(
                key.verifying_key(),
                &key.app_identifier,
                AssertionCounter::default(),
                &input_payload.challenge,
            )
            .expect("should parse and verify SignedMessage successfully using its Apple assertion");

        assert_eq!(output_payload.string, "Some payload.");
        assert_eq!(*counter, 1);
    }

    #[tokio::test]
    async fn test_signed_message_signature_verification_error() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedMessage::sign_ecdsa(&ToyPayload::default(), EcdsaSignatureType::Google, &key)
            .await
            .expect("should sign message with ECDSA key");

        // Verifying with a wrong public key should return a `Error::SignatureVerification`.
        let other_key = SigningKey::random(&mut OsRng);
        let error = signed_message
            .parse_and_verify_ecdsa(EcdsaSignatureType::Google, other_key.verifying_key())
            .expect_err("verifying SignedMessage should return an error");

        assert_matches!(error, Error::SignatureVerification(_));
    }

    #[tokio::test]
    async fn test_apple_signed_message_assertion_verification_error() {
        let key = create_mock_apple_attested_key();
        let input_payload = ToyPayload::default();
        let signed_message = SignedMessage::sign_apple(&input_payload, &key)
            .await
            .expect("should sign message with Apple attested key");

        // Verifying with a wrong challenge should return a `Error::AssertionVerification`.
        let error = signed_message
            .parse_and_verify_apple(
                key.verifying_key(),
                &key.app_identifier,
                AssertionCounter::default(),
                b"wrong_challenge",
            )
            .expect_err("verifying SignedMessage should return an error");

        assert_matches!(error, Error::AssertionVerification(_));
    }

    #[rstest]
    #[tokio::test]
    async fn test_signed_message_error_type_mismatch(
        #[values(
            SignatureType::Ecdsa(EcdsaSignatureType::Pin),
            SignatureType::Ecdsa(EcdsaSignatureType::Google),
            SignatureType::AppleAssertion
        )]
        signature_type: SignatureType,
    ) {
        // Pick a wrong signature type to verify for every input signature type.
        let verify_signature_type = match signature_type {
            SignatureType::Ecdsa(EcdsaSignatureType::Pin) => SignatureType::Ecdsa(EcdsaSignatureType::Google),
            SignatureType::Ecdsa(EcdsaSignatureType::Google) => SignatureType::AppleAssertion,
            SignatureType::AppleAssertion => SignatureType::Ecdsa(EcdsaSignatureType::Pin),
        };

        let ecdsa_key = SigningKey::random(&mut OsRng);
        let attested_key = create_mock_apple_attested_key();
        let payload = ToyPayload::default();

        let signed_message = match signature_type {
            SignatureType::Ecdsa(r#type) => SignedMessage::sign_ecdsa(&payload, r#type, &ecdsa_key).await,
            SignatureType::AppleAssertion => SignedMessage::sign_apple(&payload, &attested_key).await,
        }
        .expect("should sign message successfully");

        // Verifying with the wrong signature type should return a `Error::TypeMismatch`.
        let error = match verify_signature_type {
            SignatureType::Ecdsa(r#type) => signed_message.parse_and_verify_ecdsa(r#type, ecdsa_key.verifying_key()),
            SignatureType::AppleAssertion => signed_message
                .parse_and_verify_apple(
                    attested_key.verifying_key(),
                    &attested_key.app_identifier,
                    AssertionCounter::default(),
                    &payload.challenge,
                )
                .map(|(payload, _)| payload),
        }
        .expect_err("verifying SignedMessage should return an error");

        assert_matches!(
            error,
            Error::SignatureTypeMismatch {
                expected,
                received
            } if expected == verify_signature_type && received == signature_type
        );
    }

    #[tokio::test]
    async fn test_subject_ecdsa_signed_message() {
        let key = SigningKey::random(&mut OsRng);
        let signed_message = SignedSubjectMessage::sign_ecdsa(ToyPayload::default(), EcdsaSignatureType::Google, &key)
            .await
            .expect("should sign message successfully");

        let payload = signed_message
            .parse_and_verify_ecdsa(EcdsaSignatureType::Google, key.verifying_key())
            .expect("should parse and verify SignedSubjectMessage successfully using its ECDSA signature");

        assert_eq!(payload.string, "Some payload.");
    }

    #[tokio::test]
    async fn test_subject_apple_signed_message() {
        let key = create_mock_apple_attested_key();
        let input_payload = ToyPayload::default();
        let signed_message = SignedSubjectMessage::sign_apple(input_payload.clone(), &key)
            .await
            .expect("should sign message successfully");

        let (output_payload, counter) = signed_message
            .parse_and_verify_apple(
                key.verifying_key(),
                &key.app_identifier,
                AssertionCounter::default(),
                &input_payload.challenge,
            )
            .expect("should parse and verify SignedSubjectMessage successfully using its Apple assertion");

        assert_eq!(output_payload.string, "Some payload.");
        assert_eq!(*counter, 1);
    }

    #[rstest]
    #[tokio::test]
    async fn test_subject_signed_message_error_subject_mismatch(
        #[values(
            SignatureType::Ecdsa(EcdsaSignatureType::Pin),
            SignatureType::Ecdsa(EcdsaSignatureType::Google),
            SignatureType::AppleAssertion
        )]
        signature_type: SignatureType,
    ) {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct WrongToyPayload {
            string: String,
            challenge: Vec<u8>,
        }
        impl SubjectPayload for WrongToyPayload {
            const SUBJECT: &'static str = "wrong_subject";
        }
        impl ContainsChallenge for WrongToyPayload {
            fn challenge(&self) -> Result<impl AsRef<[u8]>> {
                Ok(&self.challenge)
            }
        }

        let ecdsa_key = SigningKey::random(&mut OsRng);
        let attested_key = create_mock_apple_attested_key();
        let payload = WrongToyPayload {
            string: "WRONG!".to_string(),
            challenge: b"challenge".to_vec(),
        };

        let signed_message = match signature_type {
            SignatureType::Ecdsa(r#type) => SignedSubjectMessage::sign_ecdsa(payload.clone(), r#type, &ecdsa_key).await,
            SignatureType::AppleAssertion => SignedSubjectMessage::sign_apple(payload.clone(), &attested_key).await,
        }
        .expect("should sign message successfully");

        let decoded_message =
            serde_json::from_str::<SignedSubjectMessage<ToyPayload>>(&serde_json::to_string(&signed_message).unwrap())
                .unwrap();

        // Verifying with an incorrect subject field should return a `Error::SubjectMismatch`.
        let error = match signature_type {
            SignatureType::Ecdsa(r#type) => decoded_message.parse_and_verify_ecdsa(r#type, ecdsa_key.verifying_key()),
            SignatureType::AppleAssertion => decoded_message
                .parse_and_verify_apple(
                    attested_key.verifying_key(),
                    &attested_key.app_identifier,
                    AssertionCounter::default(),
                    &payload.challenge,
                )
                .map(|(payload, _)| payload),
        }
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
