use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crypto::p256_der::DerVerifyingKey;
use jwt::JwtSubject;
use jwt::UnverifiedJwt;
use utils::vec_at_least::VecAtLeastTwo;

/// Registration challenge, sent by account server to wallet after the latter requests enrollment.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    #[serde_as(as = "Base64")]
    pub challenge: Vec<u8>,
}

/// Registration request, sent by wallet to account server after receiving challenge.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Registration {
    pub attestation: RegistrationAttestation,
    #[serde_as(as = "Base64")]
    pub pin_pubkey: DerVerifyingKey,
}

/// App and key attestation data for both platforms.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "platform", rename_all = "snake_case")]
pub enum RegistrationAttestation {
    Apple {
        #[serde_as(as = "Base64")]
        data: Vec<u8>,
    },
    Google {
        #[serde_as(as = "Vec<Base64>")]
        certificate_chain: VecAtLeastTwo<Vec<u8>>,
        integrity_token: String,
    },
}

/// Wallet certificate provisioning message, sent by account server to wallet after successful registration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub certificate: WalletCertificate,
}

pub type WalletCertificate = UnverifiedJwt<WalletCertificateClaims>;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletCertificateClaims {
    pub wallet_id: String,
    #[serde_as(as = "Base64")]
    pub hw_pubkey: DerVerifyingKey,
    #[serde_as(as = "Base64")]
    pub pin_pubkey_hash: Vec<u8>,
    pub version: u32,

    pub iss: String,

    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,
}

impl JwtSubject for WalletCertificateClaims {
    const SUB: &'static str = "wallet_certificate";
}

#[cfg(feature = "client")]
mod client {
    use futures::TryFutureExt;

    use crypto::keys::EphemeralEcdsaKey;
    use crypto::keys::SecureEcdsaKey;
    use crypto::p256_der::DerVerifyingKey;
    use platform_support::attested_key::AppleAttestedKey;
    use utils::vec_at_least::VecAtLeastTwo;

    use crate::error::EncodeError;
    use crate::signed::ChallengeResponse;

    use super::Registration;
    use super::RegistrationAttestation;

    // Constructors for ChallengeResponse<Registration>.
    impl ChallengeResponse<Registration> {
        pub async fn new_apple<AK, PK>(
            attested_key: &AK,
            attestation_data: Vec<u8>,
            pin_signing_key: &PK,
            challenge: Vec<u8>,
        ) -> Result<Self, EncodeError>
        where
            AK: AppleAttestedKey,
            PK: EphemeralEcdsaKey,
        {
            let pin_pubkey = pin_signing_key
                .verifying_key()
                .map_err(|e| EncodeError::VerifyingKey(Box::new(e)))
                .await?;

            let registration = Registration {
                attestation: RegistrationAttestation::Apple { data: attestation_data },
                pin_pubkey: DerVerifyingKey::from(pin_pubkey),
            };

            Self::sign_apple(registration, challenge, 0, attested_key, pin_signing_key).await
        }

        pub async fn new_google<SK, PK>(
            secure_key: &SK,
            certificate_chain: VecAtLeastTwo<Vec<u8>>,
            integrity_token: String,
            pin_signing_key: &PK,
            challenge: Vec<u8>,
        ) -> Result<Self, EncodeError>
        where
            SK: SecureEcdsaKey,
            PK: EphemeralEcdsaKey,
        {
            let pin_pubkey = pin_signing_key
                .verifying_key()
                .map_err(|e| EncodeError::VerifyingKey(Box::new(e)))
                .await?;

            Self::sign_google(
                Registration {
                    attestation: RegistrationAttestation::Google {
                        certificate_chain,
                        integrity_token,
                    },
                    pin_pubkey: DerVerifyingKey::from(pin_pubkey),
                },
                challenge,
                0,
                secure_key,
                pin_signing_key,
            )
            .await
        }
    }
}
