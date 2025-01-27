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
use crate::vec_at_least::VecAtLeastTwo;

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
    Google {
        // TODO: Consider using `BorrowingCertificate` here when it becomes available.
        #[serde_as(as = "Vec<Base64>")]
        certificate_chain: VecAtLeastTwo<Vec<u8>>,
        integrity_token: String,
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

    pub async fn new_google<SK, PK>(
        secure_key: &SK,
        certificate_chain: VecAtLeastTwo<Vec<u8>>,
        integrity_token: String,
        pin_signing_key: &PK,
        challenge: Vec<u8>,
    ) -> Result<Self>
    where
        SK: SecureEcdsaKey,
        PK: EphemeralEcdsaKey,
    {
        let pin_pubkey = pin_signing_key
            .verifying_key()
            .map_err(|e| Error::VerifyingKey(Box::new(e)))
            .await?;

        Self::sign_google(
            Registration {
                attestation: RegistrationAttestation::Google {
                    certificate_chain,
                    integrity_token,
                },
                pin_pubkey: pin_pubkey.into(),
            },
            challenge,
            0,
            secure_key,
            pin_signing_key,
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
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use p256::pkcs8::DecodePublicKey;
    use rand_core::OsRng;
    use rustls_pki_types::CertificateDer;

    use android_attest::android_crl::RevocationStatusList;
    use android_attest::attestation_extension::key_description::KeyDescription;
    use android_attest::certificate_chain::verify_google_key_attestation;
    use android_attest::mock_chain::MockCaChain;
    use android_attest::root_public_key::RootPublicKey;
    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AssertionCounter;
    use apple_app_attest::AttestationEnvironment;
    use apple_app_attest::MockAttestationCa;
    use apple_app_attest::VerifiedAttestation;

    use crate::account::signed::ChallengeResponse;
    use crate::account::signed::SequenceNumberComparison;
    use crate::apple::MockAppleAttestedKey;
    use crate::utils;

    use super::Registration;
    use super::RegistrationAttestation;

    #[tokio::test]
    async fn test_apple_registration() {
        // The Wallet Provider generates a challenge.
        let challenge = b"challenge";

        // Generate a mock assertion, a mock attested key and a mock PIN siging key.
        let environment = AttestationEnvironment::Development;
        let app_identifier = AppIdentifier::new_mock();
        let mock_ca = MockAttestationCa::generate();
        let (attested_key, attestation) =
            MockAppleAttestedKey::new_with_attestation(&mock_ca, challenge, environment, app_identifier.clone());
        let pin_signing_key = SigningKey::random(&mut OsRng);

        // The Wallet generates a registration message.
        let msg = ChallengeResponse::<Registration>::new_apple(
            &attested_key,
            attestation,
            &pin_signing_key,
            challenge.to_vec(),
        )
        .await
        .expect("challenge response with apple registration should be created successfully");

        let unverified = msg
            .dangerous_parse_unverified()
            .expect("registration should parse successfully");
        let RegistrationAttestation::Apple { data: attestation_data } = &unverified.payload.attestation else {
            panic!("apple registration message should contain attestation data");
        };

        let (_attestation, public_key) = VerifiedAttestation::parse_and_verify(
            attestation_data,
            &[mock_ca.trust_anchor()],
            challenge,
            &app_identifier,
            environment,
        )
        .expect("apple attestation should validate succesfully");

        // The Wallet Provider takes the public keys from the message and verifies the signatures.
        msg.parse_and_verify_apple(
            challenge,
            SequenceNumberComparison::EqualTo(0),
            &public_key,
            &app_identifier,
            AssertionCounter::default(),
            &unverified.payload.pin_pubkey.0,
        )
        .expect("apple registration should verify successfully");
    }

    #[tokio::test]
    async fn test_google_registration() {
        // The Wallet Provider generates a challenge.
        let challenge = b"challenge";

        // Generate a mock certificate chain, a random app attestation token and a mock PIN signing key.
        let attested_ca_chain = MockCaChain::generate(1);
        let (attested_certificate_chain, attested_private_key) =
            attested_ca_chain.generate_attested_leaf_certificate(&KeyDescription::new_valid_mock(challenge.to_vec()));
        let integrity_token = utils::random_string(32);
        let pin_signing_key = SigningKey::random(&mut OsRng);

        // The Wallet generates a registration message.
        let msg = ChallengeResponse::<Registration>::new_google(
            &attested_private_key,
            attested_certificate_chain.try_into().unwrap(),
            integrity_token,
            &pin_signing_key,
            challenge.to_vec(),
        )
        .await
        .expect("challenge response with google registration should be created successfully");

        let unverified = msg
            .dangerous_parse_unverified()
            .expect("registration should parse successfully");
        let RegistrationAttestation::Google { certificate_chain, .. } = &unverified.payload.attestation else {
            panic!("google registration message should contain certificate chain");
        };

        // Verify mock certificate chain and extract the leaf certificate public key.
        let der_certificate_chain = certificate_chain
            .as_ref()
            .iter()
            .map(|der| CertificateDer::from_slice(der))
            .collect::<Vec<_>>();
        let root_public_keys = vec![RootPublicKey::Rsa(attested_ca_chain.root_public_key.clone())];

        let certificate = verify_google_key_attestation(
            &der_certificate_chain,
            &root_public_keys,
            &RevocationStatusList::default(),
            challenge,
        )
        .unwrap();

        let attested_public_key = VerifyingKey::from_public_key_der(certificate.public_key().raw).unwrap();

        // The Wallet Provider takes the public keys from the message and verifies the signatures.
        msg.parse_and_verify_google(
            challenge,
            SequenceNumberComparison::EqualTo(0),
            &attested_public_key,
            &unverified.payload.pin_pubkey.0,
        )
        .expect("google registration should verify successfully");
    }
}
