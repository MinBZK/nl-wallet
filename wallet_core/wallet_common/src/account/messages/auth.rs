use futures::{try_join, TryFutureExt};
use serde::{Deserialize, Serialize};

use crate::{
    account::{
        serialization::{Base64Bytes, DerVerifyingKey},
        signed::SignedDouble,
    },
    errors::{Error, Result},
    jwt::{Jwt, JwtSubject},
    keys::{EphemeralEcdsaKey, SecureEcdsaKey},
};

// Registration challenge response

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub challenge: Base64Bytes,
}

// Registration request and response

#[derive(Debug, Serialize, Deserialize)]
pub struct Registration {
    pub pin_pubkey: DerVerifyingKey,
    pub hw_pubkey: DerVerifyingKey,
}

impl Registration {
    pub async fn new_signed(
        hw_privkey: &impl SecureEcdsaKey,
        pin_privkey: &impl EphemeralEcdsaKey,
        challenge: &[u8],
    ) -> Result<SignedDouble<Registration>> {
        let (pin_pubkey, hw_pubkey) = try_join!(
            pin_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
            hw_privkey.verifying_key().map_err(|e| Error::VerifyingKey(e.into())),
        )?;

        SignedDouble::sign(
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletCertificateClaims {
    pub wallet_id: String,
    pub hw_pubkey: DerVerifyingKey,
    pub pin_pubkey_hash: Base64Bytes,
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
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};

    use super::*;

    #[tokio::test]
    async fn registration() -> Result<()> {
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        // wallet provider generates a challenge
        let challenge = b"challenge";

        // wallet calculates wallet provider registration message
        let msg = Registration::new_signed(&hw_privkey, &pin_privkey, challenge).await?;
        println!("{}", &msg.0);

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
