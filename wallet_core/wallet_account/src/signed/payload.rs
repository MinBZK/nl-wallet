use p256::ecdsa::VerifyingKey;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use apple_app_attest::AppIdentifier;
use apple_app_attest::AssertionCounter;
use wallet_common::apple::AppleAttestedKey;
use wallet_common::keys::EphemeralEcdsaKey;
use wallet_common::keys::SecureEcdsaKey;

use super::super::errors::Error;
use super::super::errors::Result;
use super::signed_message::SignedMessage;
use super::signed_message::SignedSubjectMessage;
use super::signed_message::SubjectPayload;
use super::ContainsChallenge;
use super::EcdsaSignatureType;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRequestPayload {
    pub wallet_id: String,
    pub sequence_number: u64,
    pub instruction_name: String,
}

impl ChallengeRequestPayload {
    pub fn new(wallet_id: String, sequence_number: u64, instruction_name: String) -> Self {
        ChallengeRequestPayload {
            wallet_id,
            sequence_number,
            instruction_name,
        }
    }

    pub fn verify(&self, wallet_id: &str, sequence_number_comparison: SequenceNumberComparison) -> Result<()> {
        if wallet_id != self.wallet_id {
            return Err(Error::WalletIdMismatch);
        }

        if !sequence_number_comparison.verify(self.sequence_number) {
            return Err(Error::SequenceNumberMismatch);
        }

        Ok(())
    }
}

impl SubjectPayload for ChallengeRequestPayload {
    const SUBJECT: &'static str = "instruction_challenge_request";
}

// When signing and validating a `ChallengeRequest` using Apple assertions,
// we need this type to contain a challenge. However, as this is the actual
// message that requests a challenge from the Wallet Provider, we have a
// bootstrap problem and cannot use a server generated random challenge. In
// its place we use the `wallet_id` field to act as a predictable byte string.
//
// As the `wallet_id` is sent in `InstructionChallengeRequest` along with the
// `ChallengeRequest`, this in itself does not provide any sort of replay
// protection. This is not an issue, as `ChallengeResponse` does include a
// server generated challenge and this is the message that includes the
// actual instruction, to be performed by the Wallet Provider.
impl ContainsChallenge for ChallengeRequestPayload {
    fn challenge(&self) -> Result<impl AsRef<[u8]>> {
        Ok(self.wallet_id.as_bytes())
    }
}

/// Sent to the Wallet Provider to request an instruction challenge. This
/// is signed with either the device's hardware key or Apple attested key.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeRequest(SignedSubjectMessage<ChallengeRequestPayload>);

impl ChallengeRequest {
    pub async fn sign_apple<K>(
        wallet_id: String,
        sequence_number: u64,
        instruction_name: String,
        attested_key: &K,
    ) -> Result<Self>
    where
        K: AppleAttestedKey,
    {
        let challenge_request = ChallengeRequestPayload::new(wallet_id, sequence_number, instruction_name);
        let signed = SignedSubjectMessage::sign_apple(challenge_request, attested_key).await?;

        Ok(Self(signed))
    }

    pub async fn sign_google<K>(
        wallet_id: String,
        sequence_number: u64,
        instruction_name: String,
        hardware_signing_key: &K,
    ) -> Result<Self>
    where
        K: SecureEcdsaKey,
    {
        let challenge_request = ChallengeRequestPayload::new(wallet_id, sequence_number, instruction_name);
        let signed =
            SignedSubjectMessage::sign_ecdsa(challenge_request, EcdsaSignatureType::Google, hardware_signing_key)
                .await?;

        Ok(Self(signed))
    }

    pub fn parse_and_verify_apple(
        &self,
        wallet_id: &str,
        sequence_number_comparison: SequenceNumberComparison,
        verifying_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: AssertionCounter,
    ) -> Result<(ChallengeRequestPayload, AssertionCounter)> {
        let (request, counter) =
            self.0
                .parse_and_verify_apple(verifying_key, app_identifier, previous_counter, wallet_id.as_bytes())?;
        request.verify(wallet_id, sequence_number_comparison)?;

        Ok((request, counter))
    }

    pub fn parse_and_verify_google(
        &self,
        wallet_id: &str,
        sequence_number_comparison: SequenceNumberComparison,
        verifying_key: &VerifyingKey,
    ) -> Result<ChallengeRequestPayload> {
        let request = self
            .0
            .parse_and_verify_ecdsa(EcdsaSignatureType::Google, verifying_key)?;
        request.verify(wallet_id, sequence_number_comparison)?;

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

impl<T> SubjectPayload for ChallengeResponsePayload<T> {
    const SUBJECT: &'static str = "instruction_challenge_response";
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

impl<T> SignedSubjectMessage<ChallengeResponsePayload<T>> {
    async fn sign_pin<K>(payload: T, challenge: Vec<u8>, sequence_number: u64, pin_signing_key: &K) -> Result<Self>
    where
        T: Serialize,
        K: EphemeralEcdsaKey,
    {
        let challenge_response = ChallengeResponsePayload {
            payload,
            challenge,
            sequence_number,
        };
        let signed = Self::sign_ecdsa(challenge_response, EcdsaSignatureType::Pin, pin_signing_key).await?;

        Ok(signed)
    }

    fn parse_and_verify_pin(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        pin_verifying_key: &VerifyingKey,
    ) -> Result<ChallengeResponsePayload<T>>
    where
        T: DeserializeOwned,
    {
        let challenge_response = self.parse_and_verify_ecdsa(EcdsaSignatureType::Pin, pin_verifying_key)?;

        challenge_response.verify(challenge, sequence_number_comparison)?;

        Ok(challenge_response)
    }
}

impl<T> ContainsChallenge for SignedSubjectMessage<ChallengeResponsePayload<T>>
where
    T: DeserializeOwned,
{
    fn challenge(&self) -> Result<impl AsRef<[u8]>> {
        // We need to parse the inner message to get to the challenge, which unfortunately leads to double parsing.
        let challenge_response = self.dangerous_parse_unverified()?;

        Ok(challenge_response.challenge)
    }
}

/// Wraps a [`ChallengeResponsePayload`], which contains an arbitrary payload and the challenge received in response
/// to [`ChallengeRequest`]. The Wallet signs it with two keys. For the inner signing the PIN key is used, while the
/// outer signing is done with either the device's hardware key or Apple attested key.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse<T>(SignedMessage<SignedSubjectMessage<ChallengeResponsePayload<T>>>);

impl<T> ChallengeResponse<T> {
    pub async fn sign_apple<AK, PK>(
        payload: T,
        challenge: Vec<u8>,
        sequence_number: u64,
        attested_key: &AK,
        pin_signing_key: &PK,
    ) -> Result<Self>
    where
        T: Serialize,
        AK: AppleAttestedKey,
        PK: EphemeralEcdsaKey,
    {
        let inner_signed = SignedSubjectMessage::sign_pin(payload, challenge, sequence_number, pin_signing_key).await?;
        let outer_signed = SignedMessage::sign_apple(&inner_signed, attested_key).await?;

        Ok(Self(outer_signed))
    }

    pub async fn sign_google<HK, PK>(
        payload: T,
        challenge: Vec<u8>,
        sequence_number: u64,
        hardware_signing_key: &HK,
        pin_signing_key: &PK,
    ) -> Result<Self>
    where
        T: Serialize,
        HK: SecureEcdsaKey,
        PK: EphemeralEcdsaKey,
    {
        let inner_signed = SignedSubjectMessage::sign_pin(payload, challenge, sequence_number, pin_signing_key).await?;
        let outer_signed =
            SignedMessage::sign_ecdsa(&inner_signed, EcdsaSignatureType::Google, hardware_signing_key).await?;

        Ok(Self(outer_signed))
    }

    pub fn dangerous_parse_unverified(&self) -> Result<ChallengeResponsePayload<T>>
    where
        T: DeserializeOwned,
    {
        let challenge_response = self.0.dangerous_parse_unverified()?.dangerous_parse_unverified()?;

        Ok(challenge_response)
    }

    pub fn parse_and_verify_apple(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        apple_verifying_key: &VerifyingKey,
        app_identifier: &AppIdentifier,
        previous_counter: AssertionCounter,
        pin_verifying_key: &VerifyingKey,
    ) -> Result<(ChallengeResponsePayload<T>, AssertionCounter)>
    where
        T: DeserializeOwned,
    {
        let (inner_signed, counter) =
            self.0
                .parse_and_verify_apple(apple_verifying_key, app_identifier, previous_counter, challenge)?;
        let challenge_response =
            inner_signed.parse_and_verify_pin(challenge, sequence_number_comparison, pin_verifying_key)?;

        Ok((challenge_response, counter))
    }

    pub fn parse_and_verify_google(
        &self,
        challenge: &[u8],
        sequence_number_comparison: SequenceNumberComparison,
        hardware_verifying_key: &VerifyingKey,
        pin_verifying_key: &VerifyingKey,
    ) -> Result<ChallengeResponsePayload<T>>
    where
        T: DeserializeOwned,
    {
        let inner_signed = self
            .0
            .parse_and_verify_ecdsa(EcdsaSignatureType::Google, hardware_verifying_key)?;
        let challenge_response =
            inner_signed.parse_and_verify_pin(challenge, sequence_number_comparison, pin_verifying_key)?;

        Ok(challenge_response)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use wallet_common::apple::MockAppleAttestedKey;
    use wallet_common::utils;

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

    fn create_mock_apple_attested_key() -> MockAppleAttestedKey {
        let app_identifier = AppIdentifier::new_mock();

        MockAppleAttestedKey::new_random(app_identifier)
    }

    #[tokio::test]
    async fn test_apple_challenge_request() {
        let wallet_id = utils::random_string(32);
        let sequence_number = 42;
        let instruction_name = "jump";
        let attested_key = create_mock_apple_attested_key();

        let signed = ChallengeRequest::sign_apple(
            wallet_id.clone(),
            sequence_number,
            instruction_name.to_string(),
            &attested_key,
        )
        .await
        .expect("should sign SignedChallengeRequest successfully");

        // Verifying against an incorrect wallet_id should return a `Error::AssertionVerification`.
        // Note that an `Error::WalletIdMismatch` is not returned, as the wallet id is first checked
        // when validating the Apple assertion.
        let error = signed
            .parse_and_verify_apple(
                "incorrect",
                SequenceNumberComparison::EqualTo(sequence_number),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
            )
            .expect_err("verifying SignedChallengeRequest should return an error");

        assert_matches!(error, Error::AssertionVerification(_));

        // Verifying against an sequence number that is too low should return a `Error::SequenceNumberMismatch`.
        let error = signed
            .parse_and_verify_apple(
                &wallet_id,
                SequenceNumberComparison::LargerThan(sequence_number),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
            )
            .expect_err("verifying SignedChallengeRequest should return an error");

        assert_matches!(error, Error::SequenceNumberMismatch);

        // Verifying against the correct values should succeed.
        let (request, counter) = signed
            .parse_and_verify_apple(
                &wallet_id,
                SequenceNumberComparison::EqualTo(sequence_number),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
            )
            .expect("SignedChallengeRequest should be valid");

        assert_eq!(request.sequence_number, sequence_number);
        assert_eq!(request.instruction_name, instruction_name);
        assert_eq!(*counter, 1);
    }

    #[tokio::test]
    async fn test_google_challenge_request() {
        let wallet_id = utils::random_string(32);
        let sequence_number = 42;
        let instruction_name = "jump";
        let hw_privkey = SigningKey::random(&mut OsRng);

        let signed = ChallengeRequest::sign_google(
            wallet_id.clone(),
            sequence_number,
            instruction_name.to_string(),
            &hw_privkey,
        )
        .await
        .expect("should sign SignedChallengeRequest successfully");

        // Verifying against an incorrect wallet_id should return a `Error::WalletIdMismatch`.
        let error = signed
            .parse_and_verify_google(
                "incorrect",
                SequenceNumberComparison::EqualTo(sequence_number),
                hw_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeRequest should return an error");

        assert_matches!(error, Error::WalletIdMismatch);

        // Verifying against an sequence number that is too low should return a `Error::SequenceNumberMismatch`.
        let error = signed
            .parse_and_verify_google(
                &wallet_id,
                SequenceNumberComparison::LargerThan(sequence_number),
                hw_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeRequest should return an error");

        assert_matches!(error, Error::SequenceNumberMismatch);

        // Verifying against the correct values should succeed.
        let request = signed
            .parse_and_verify_google(
                &wallet_id,
                SequenceNumberComparison::EqualTo(sequence_number),
                hw_privkey.verifying_key(),
            )
            .expect("SignedChallengeRequest should be valid");

        assert_eq!(request.sequence_number, sequence_number);
        assert_eq!(request.instruction_name, instruction_name);
    }

    #[tokio::test]
    async fn test_apple_challenge_response() {
        let sequence_number = 1337;
        let challenge = b"challenge";
        let attested_key = create_mock_apple_attested_key();
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = ChallengeResponse::sign_apple(
            ToyMessage::default(),
            challenge.to_vec(),
            sequence_number,
            &attested_key,
            &pin_privkey,
        )
        .await
        .expect("should sign ChallengeResponse successfully");

        // Verifying against an incorrect challenge should return a `Error::AssertionVerification`.
        // Note that an `Error::ChallengeMismatch` is not returned, as the challenge is first checked
        // when validating the Apple assertion.
        let error = signed
            .parse_and_verify_apple(
                b"wrong",
                SequenceNumberComparison::LargerThan(sequence_number - 1),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
                pin_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeResponse should return an error");

        assert_matches!(error, Error::AssertionVerification(_));

        // Verifying against an sequence number that is too low should return a `Error::SequenceNumberMismatch`.
        let error = signed
            .parse_and_verify_apple(
                challenge,
                SequenceNumberComparison::EqualTo(42),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
                pin_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeResponse should return an error");

        assert_matches!(error, Error::SequenceNumberMismatch);

        // Verifying against the correct values should succeed.
        let (verified, counter) = signed
            .parse_and_verify_apple(
                challenge,
                SequenceNumberComparison::LargerThan(sequence_number - 1),
                attested_key.verifying_key(),
                &attested_key.app_identifier,
                AssertionCounter::default(),
                pin_privkey.verifying_key(),
            )
            .expect("SignedChallengeResponse should be valid");

        assert_eq!(ToyMessage::default(), verified.payload);
        assert_eq!(*counter, 1)
    }

    #[tokio::test]
    async fn test_google_challenge_response() {
        let sequence_number = 1337;
        let challenge = b"challenge";
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        let signed = ChallengeResponse::sign_google(
            ToyMessage::default(),
            challenge.to_vec(),
            sequence_number,
            &hw_privkey,
            &pin_privkey,
        )
        .await
        .expect("should sign ChallengeResponse successfully");

        // Verifying against an incorrect challenge should return a `Error::ChallengeMismatch`.
        let error = signed
            .parse_and_verify_google(
                b"wrong",
                SequenceNumberComparison::LargerThan(sequence_number - 1),
                hw_privkey.verifying_key(),
                pin_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeResponse should return an error");

        assert_matches!(error, Error::ChallengeMismatch);

        // Verifying against an sequence number that is too low should return a `Error::SequenceNumberMismatch`.
        let error = signed
            .parse_and_verify_google(
                challenge,
                SequenceNumberComparison::EqualTo(42),
                hw_privkey.verifying_key(),
                pin_privkey.verifying_key(),
            )
            .expect_err("verifying SignedChallengeResponse should return an error");

        assert_matches!(error, Error::SequenceNumberMismatch);

        // Verifying against the correct values should succeed.
        let verified = signed
            .parse_and_verify_google(
                challenge,
                SequenceNumberComparison::LargerThan(sequence_number - 1),
                hw_privkey.verifying_key(),
                pin_privkey.verifying_key(),
            )
            .expect("SignedChallengeResponse should be valid");

        assert_eq!(ToyMessage::default(), verified.payload);
    }
}
