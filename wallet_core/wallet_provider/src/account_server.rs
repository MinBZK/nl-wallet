use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, EncodePublicKey},
};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use wallet_common::{
    account::{
        auth::{Registration, WalletCertificate, WalletCertificateClaims},
        jwt::{EcdsaDecodingKey, Jwt, JwtClaims},
        serialization::Base64Bytes,
        signed::SignedDouble,
    },
    utils::{random_bytes, random_string},
};

#[derive(Debug, thiserror::Error)]
pub enum AccountServerError {
    // Do not format original error to prevent potentially leaking key material
    #[error("Server private key decoding error")]
    PrivateKeyDecoding(#[from] p256::pkcs8::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ChallengeError {
    #[error("Challenge signing error: {0}")]
    ChallengeSigning(#[source] wallet_common::errors::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("Registration challenge UTF-8 decoding error: {0}")]
    ChallengeDecoding(#[from] std::string::FromUtf8Error),
    #[error("Registration challenge validation error: {0}")]
    ChallengeValidation(#[source] wallet_common::errors::Error),
    #[error("Registration message parsing error: {0}")]
    MessageParsing(#[source] wallet_common::errors::Error),
    #[error("Registration message validation error: {0}")]
    MessageValidation(#[source] wallet_common::errors::Error),
    #[error("Incorrect registration serial number (expected: {expected:?}, received: {received:?})")]
    SerialNumberMismatch { expected: u64, received: u64 },
    #[error("Registration PIN public key decoding error: {0}")]
    PinPubKeyDecoding(#[from] p256::pkcs8::spki::Error),
    #[error("Registration PIN public key DER encoding error: {0}")]
    PinPubKeyEncoding(#[from] der::Error),
    #[error("Registration JWT signing error: {0}")]
    JwtSigning(#[source] wallet_common::errors::Error),
}

const WALLET_CERTIFICATE_VERSION: u32 = 0;

pub struct AccountServer {
    privkey: Vec<u8>,
    pin_hash_salt: Vec<u8>,

    pub name: String,
    pub pubkey: EcdsaDecodingKey,
}

/// Used as the challenge in the challenge-response protocol during wallet registration.
#[derive(Serialize, Deserialize, Debug)]
struct RegistrationChallengeClaims {
    wallet_id: String,
    exp: u64,

    /// Random bytes to serve as the actual challenge for the wallet to sign.
    random: Base64Bytes,
}

impl JwtClaims for RegistrationChallengeClaims {
    const SUB: &'static str = "registration_challenge";
}

impl AccountServer {
    pub fn new(privkey: Vec<u8>, pin_hash_salt: Vec<u8>, name: String) -> Result<AccountServer, AccountServerError> {
        let pubkey = EcdsaDecodingKey::from_sec1(
            SigningKey::from_pkcs8_der(&privkey)?
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes(),
        );
        Ok(AccountServer {
            privkey,
            pin_hash_salt,
            name,
            pubkey,
        })
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub fn registration_challenge(&self) -> Result<Vec<u8>, ChallengeError> {
        let challenge = Jwt::sign(
            &RegistrationChallengeClaims {
                wallet_id: random_string(32),
                random: random_bytes(32).into(),
                exp: jsonwebtoken::get_current_timestamp() + 60,
            },
            &self.privkey,
        )
        .map_err(ChallengeError::ChallengeSigning)?
        .0
        .as_bytes()
        .to_vec();
        Ok(challenge)
    }

    fn verify_registration_challenge(
        &self,
        challenge: &[u8],
    ) -> Result<RegistrationChallengeClaims, RegistrationError> {
        Jwt::parse_and_verify(&String::from_utf8(challenge.to_owned())?.into(), &self.pubkey)
            .map_err(RegistrationError::ChallengeValidation)
    }

    pub fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, RegistrationError> {
        // We don't have the public keys yet against which to verify the message, as those are contained within the
        // message (like in X509 certificate requests). So first parse it to grab the public keys from it.
        let unverified = registration_message
            .dangerous_parse_unverified()
            .map_err(RegistrationError::MessageParsing)?;

        let challenge = &unverified.challenge.0;
        let wallet_id = self.verify_registration_challenge(challenge)?.wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;
        let signed = registration_message
            .parse_and_verify(challenge, &hw_pubkey, &pin_pubkey)
            .map_err(RegistrationError::MessageValidation)?;

        if signed.serial_number != 0 {
            return Err(RegistrationError::SerialNumberMismatch {
                expected: 0,
                received: signed.serial_number,
            });
        }

        self.new_wallet_certificate(wallet_id, hw_pubkey, pin_pubkey)

        // TODO insert into users table
    }

    fn new_wallet_certificate(
        &self,
        wallet_id: String,
        wallet_hw_pubkey: VerifyingKey,
        wallet_pin_pubkey: VerifyingKey,
    ) -> Result<WalletCertificate, RegistrationError> {
        let pin_pubkey_bts = wallet_pin_pubkey.to_public_key_der()?.to_vec();

        let pin_pubkey_tohash = der_encode(vec![self.pin_hash_salt.clone(), pin_pubkey_bts])?;

        let cert = WalletCertificateClaims {
            wallet_id,
            hw_pubkey: wallet_hw_pubkey.into(),
            pin_pubkey_hash: sha2::Sha256::digest(pin_pubkey_tohash).to_vec().into(),
            version: WALLET_CERTIFICATE_VERSION,

            iss: self.name.clone(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign(&cert, &self.privkey).map_err(RegistrationError::JwtSigning)
    }
}

fn der_encode(payload: impl der::Encode) -> Result<Vec<u8>, der::Error> {
    let mut buf = Vec::<u8>::with_capacity(payload.encoded_len()?.try_into()?);
    payload.encode_to_vec(&mut buf)?;
    Ok(buf)
}

#[cfg(any(test, feature = "stub"))]
pub mod stub {
    use p256::{ecdsa::SigningKey, pkcs8::EncodePrivateKey};
    use rand::rngs::OsRng;

    use wallet_common::utils::random_bytes;

    use super::AccountServer;

    #[allow(dead_code)] // Clippy does not seem to understand that this is used during testing
    pub fn account_server() -> AccountServer {
        let account_server_privkey = SigningKey::random(&mut OsRng);

        AccountServer::new(
            account_server_privkey.to_pkcs8_der().unwrap().as_bytes().to_vec(),
            random_bytes(32),
            "stub_account_server".into(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;

    use super::*;

    #[test]
    fn test_account_server() {
        // Setup account server
        let account_server = stub::account_server();

        // Set up keys
        let hw_privkey = SigningKey::random(&mut OsRng);
        let pin_privkey = SigningKey::random(&mut OsRng);

        // Register
        let challenge = account_server
            .registration_challenge()
            .expect("Could not get registration challenge");
        let registration_message =
            Registration::new_signed(&hw_privkey, &pin_privkey, &challenge).expect("Could not sign new registration");
        let cert = account_server
            .register(registration_message)
            .expect("Could not process registration message at account server");

        // Verify the certificate
        let cert_data = cert
            .parse_and_verify(&account_server.pubkey)
            .expect("Could not parse and verify wallet certificate");
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(cert_data.hw_pubkey.0, *hw_privkey.verifying_key());
    }
}
