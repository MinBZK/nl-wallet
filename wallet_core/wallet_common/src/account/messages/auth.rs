use futures::{try_join, TryFutureExt};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use crate::{
    account::{
        errors::{Error, Result},
        serialization::DerVerifyingKey,
        signed::ChallengeResponse,
    },
    jwt::{Jwt, JwtSubject},
    keys::{EphemeralEcdsaKey, SecureEcdsaKey},
};

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
    pub pin_pubkey: DerVerifyingKey,
    pub hw_pubkey: DerVerifyingKey,
}

impl ChallengeResponse<Registration> {
    pub async fn new_signed(
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: Vec<u8>,
    ) -> Result<Self> {
        let (pin_pubkey, hw_pubkey) = try_join!(
            pin_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
            hw_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
        )?;

        Self::sign(
            Registration {
                pin_pubkey: pin_pubkey.into(),
                hw_pubkey: hw_pubkey.into(),
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
    use crate::account::signed::SequenceNumberComparison;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::*;

    #[tokio::test]
    async fn registration() -> Result<()> {
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        // wallet provider generates a challenge
        let challenge = b"challenge";

        // wallet calculates wallet provider registration message
        let msg = ChallengeResponse::<Registration>::new_signed(&hw_privkey, &pin_privkey, challenge.to_vec()).await?;

        let unverified = msg.dangerous_parse_unverified()?;

        // wallet provider takes the public keys from the message, and verifies the signatures
        msg.parse_and_verify(
            challenge,
            SequenceNumberComparison::EqualTo(0),
            &unverified.payload.hw_pubkey.0,
            &unverified.payload.pin_pubkey.0,
        )?;

        Ok(())
    }
}
