use futures::try_join;
use futures::TryFutureExt;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::account::errors::Error;
use crate::account::errors::Result;
use crate::account::serialization::DerVerifyingKey;
use crate::account::signed::ChallengeResponse;
use crate::apple::AppleAttestedKey;
use crate::jwt::Jwt;
use crate::jwt::JwtSubject;
use crate::keys::EphemeralEcdsaKey;
use crate::keys::SecureEcdsaKey;

// Registration challenge response
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    #[serde_as(as = "Base64")]
    pub challenge: Vec<u8>,
}

// Registration request and response

#[derive(Debug, Serialize, Deserialize)]
pub struct Registration {
    pub attestation: RegistrationAttestation,
    pub pin_pubkey: DerVerifyingKey,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum RegistrationAttestation {
    Apple {
        #[serde_as(as = "Base64")]
        data: Vec<u8>,
    },
    // TODO: Replace this variant with Google attestation data
    None {
        hw_pubkey: DerVerifyingKey,
    },
}

impl ChallengeResponse<Registration> {
    pub async fn new_apple<AK, PK>(
        attested_key: &AK,
        attestation_data: Vec<u8>,
        pin_signing_key: &PK,
        challenge: Vec<u8>,
    ) -> Result<Self>
    where
        AK: AppleAttestedKey,
        PK: EphemeralEcdsaKey,
    {
        let pin_pubkey = pin_signing_key
            .verifying_key()
            .map_err(|e| Error::VerifyingKey(Box::new(e)))
            .await?;

        let registration = Registration {
            attestation: RegistrationAttestation::Apple { data: attestation_data },
            pin_pubkey: pin_pubkey.into(),
        };

        Self::sign_apple(registration, challenge, 0, attested_key, pin_signing_key).await
    }

    pub async fn new_unattested(
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: Vec<u8>,
    ) -> Result<Self> {
        let (pin_pubkey, hw_pubkey) = try_join!(
            pin_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
            hw_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
        )?;

        Self::sign_ecdsa(
            Registration {
                attestation: RegistrationAttestation::None {
                    hw_pubkey: hw_pubkey.into(),
                },
                pin_pubkey: pin_pubkey.into(),
            },
            challenge,
            0,
            hw_privkey,
            pin_privkey,
        )
        .await
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCertificateClaims {
    pub wallet_id: String,
    pub hw_pubkey: DerVerifyingKey,
    #[serde_as(as = "Base64")]
    pub pin_pubkey_hash: Vec<u8>,
    pub version: u32,

    pub iss: String,
    pub iat: u64,
}

impl JwtSubject for WalletCertificateClaims {
    const SUB: &'static str = "wallet_certificate";
}

pub type WalletCertificate = Jwt<WalletCertificateClaims>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub certificate: WalletCertificate,
}

#[cfg(test)]
mod tests {
    use apple_app_attest::AppIdentifier;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use crate::{
        account::{
            serialization::DerVerifyingKey,
            signed::{ChallengeResponse, SequenceNumberComparison},
        },
        apple::MockAppleAttestedKey,
        utils,
    };

    use super::{Registration, RegistrationAttestation};

    #[tokio::test]
    async fn test_apple_registration() {
        let app_identifier = AppIdentifier::new("1234567890", "com.example.app");
        let attested_key = MockAppleAttestedKey::new(app_identifier.clone());
        let attestation_data = utils::random_bytes(32);
        let pin_signing_key = SigningKey::random(&mut OsRng);

        // The Wallet Provider generates a challenge.
        let challenge = b"challenge";

        // The Wallet generates a registration message.
        let msg = ChallengeResponse::<Registration>::new_apple(
            &attested_key,
            attestation_data,
            &pin_signing_key,
            challenge.to_vec(),
        )
        .await
        .expect("challenge response with apple registration should be created successfully");

        let unverified = msg
            .dangerous_parse_unverified()
            .expect("registration should parse successfully");
        let RegistrationAttestation::Apple {
            data: _attestation_data,
        } = &unverified.payload.attestation
        else {
            panic!("apple registration message should contain attestation data");
        };

        // TODO: Get public key from attestation data.

        // The Wallet Provider takes the public keys from the message and verifies the signatures.
        msg.parse_and_verify_apple(
            challenge,
            SequenceNumberComparison::EqualTo(0),
            attested_key.signing_key.verifying_key(),
            &app_identifier,
            0,
            &unverified.payload.pin_pubkey.0,
        )
        .expect("apple registration should verify successfully");
    }

    #[tokio::test]
    async fn test_unattested_registration() {
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        // The Wallet Provider generates a challenge.
        let challenge = b"challenge";

        // The Wallet generates a registration message.
        let msg = ChallengeResponse::<Registration>::new_unattested(&hw_privkey, &pin_privkey, challenge.to_vec())
            .await
            .expect("challenge response with unattested registration should be created successfully");

        let unverified = msg
            .dangerous_parse_unverified()
            .expect("registration should parse successfully");
        let RegistrationAttestation::None {
            hw_pubkey: DerVerifyingKey(unverified_hw_pubkey),
        } = &unverified.payload.attestation
        else {
            panic!("unattested registration message should contain unattested public key");
        };

        // The Wallet Provider takes the public keys from the message and verifies the signatures.
        msg.parse_and_verify_ecdsa(
            challenge,
            SequenceNumberComparison::EqualTo(0),
            unverified_hw_pubkey,
            &unverified.payload.pin_pubkey.0,
        )
        .expect("unattested registration should verify successfully");
    }
}
