//! A temporary rust implementation of the account server, for use in tests and stubs before the Kotlin implementation
//! is fully functional.

use anyhow::{anyhow, Result};
// use base64::{engine::general_purpose::STANDARD, Engine};
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    jwt::{Jwt, JwtClaims},
    serialization::Base64Bytes,
    utils::{random_bytes, random_string},
    wallet::signed::SignedDouble,
    wp::{
        instructions::Registration, AccountServerClient, WalletCertificate, WalletCertificateClaims,
    },
};

const WALLET_CERTIFICATE_VERSION: u32 = 0;

pub struct AccountServer {
    privkey: Vec<u8>,
    pin_hash_salt: Vec<u8>,
    name: String,

    pub pubkey: Vec<u8>,
}

pub struct User {
    cert: String,
    wallet_id: String,
    // TODO logs
    // TODO user key material
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

pub struct RemoteAccountServer {
    url: String,
    client: reqwest::blocking::Client,
}

impl RemoteAccountServer {
    pub fn new(url: String) -> RemoteAccountServer {
        RemoteAccountServer {
            url,
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl AccountServerClient for RemoteAccountServer {
    fn registration_challenge(&self) -> Result<Vec<u8>> {
        #[derive(Deserialize)]
        struct Challenge {
            challenge: Base64Bytes,
        }

        let challenge = self
            .client
            .post(format!("{}/api/v1/enroll", self.url))
            .body("")
            .send()?
            .json::<Challenge>()?
            .challenge
            .0;
        Ok(challenge)
    }

    fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate> {
        #[derive(Deserialize)]
        struct Certificate {
            certificate: WalletCertificate,
        }

        let cert = self
            .client
            .post(format!("{}/api/v1/createwallet", self.url))
            .json(&registration_message)
            .send()?
            .json::<Certificate>()?
            .certificate;
        Ok(cert)
    }
}

impl AccountServerClient for AccountServer {
    fn registration_challenge(&self) -> Result<Vec<u8>> {
        AccountServer::registration_challenge(self)
    }
    fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate> {
        AccountServer::register(self, registration_message)
    }
}

impl AccountServer {
    pub fn new(privkey: Vec<u8>, pin_hash_salt: Vec<u8>, name: String) -> Result<AccountServer> {
        let pubkey = SigningKey::from_pkcs8_der(&privkey)
            .map_err(anyhow::Error::msg)?
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        Ok(AccountServer {
            privkey,
            pin_hash_salt,
            name,
            pubkey,
        })
    }

    pub fn new_stub() -> AccountServer {
        let account_server_privkey = SigningKey::random(&mut OsRng);
        AccountServer::new(
            account_server_privkey
                .to_pkcs8_der()
                .unwrap()
                .as_bytes()
                .to_vec(),
            random_bytes(32),
            "stub_account_server".into(),
        )
        .unwrap()
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub fn registration_challenge(&self) -> Result<Vec<u8>> {
        Ok(Jwt::sign(
            &RegistrationChallengeClaims {
                wallet_id: random_string(32),
                random: random_bytes(32).into(),
                exp: jsonwebtoken::get_current_timestamp() + 60,
            },
            &self.privkey,
        )?
        .0
        .as_bytes()
        .to_vec())
    }

    fn verify_registration_challenge(
        &self,
        challenge: &[u8],
    ) -> Result<RegistrationChallengeClaims> {
        Jwt::parse_and_verify(
            &String::from_utf8(challenge.to_owned())?.into(),
            &self.pubkey,
        )
    }

    pub fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate> {
        // We don't have the public keys yet against which to verify the message, as those are contained within the
        // message (like in X509 certificate requests). So first parse it to grab the public keys from it.
        let unverified = registration_message.dangerous_parse_unverified()?;

        let challenge = &unverified.challenge.0;
        let wallet_id = self.verify_registration_challenge(challenge)?.wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;
        let signed = registration_message.parse_and_verify(challenge, &hw_pubkey, &pin_pubkey)?;

        if signed.serial_number != 0 {
            return Err(anyhow!(
                "serial_number was {}, expected 0",
                signed.serial_number
            ));
        }

        self.new_wallet_certificate(wallet_id, hw_pubkey, pin_pubkey)

        // TODO insert into users table
    }

    fn new_wallet_certificate(
        &self,
        wallet_id: String,
        wallet_hw_pubkey: VerifyingKey,
        wallet_pin_pubkey: VerifyingKey,
    ) -> Result<WalletCertificate> {
        let pin_pubkey_bts = wallet_pin_pubkey
            .to_public_key_der()
            .map_err(|e| anyhow!("failed to convert pin pubkey to DER bytes: {e}"))?
            .to_vec();

        let pin_pubkey_tohash = der_encode(vec![self.pin_hash_salt.clone(), pin_pubkey_bts])
            .map_err(|e| anyhow!("failed to DER-encode pin pubkey: {e}"))?;

        let cert = WalletCertificateClaims {
            wallet_id,
            hw_pubkey: wallet_hw_pubkey.into(),
            pin_pubkey_hash: sha2::Sha256::digest(pin_pubkey_tohash).to_vec().into(),
            version: WALLET_CERTIFICATE_VERSION,

            iss: self.name.clone(),
            iat: jsonwebtoken::get_current_timestamp(),
        };

        Jwt::sign(&cert, &self.privkey)
    }
}

fn der_encode(payload: impl der::Encode) -> Result<Vec<u8>, der::Error> {
    let mut buf = Vec::<u8>::with_capacity(payload.encoded_len()?.try_into()?);
    payload.encode_to_vec(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
pub mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng, pkcs8::EncodePrivateKey};

    use crate::{
        utils::random_bytes,
        wp::{instructions, AccountServer, RemoteAccountServer},
    };

    use super::AccountServerClient;

    pub fn new_account_server() -> (AccountServer, Vec<u8>) {
        let as_privkey = SigningKey::random(&mut OsRng);
        (
            AccountServer::new(
                as_privkey.to_pkcs8_der().unwrap().as_bytes().to_vec(),
                random_bytes(32),
                "test_account_server".to_owned(),
            )
            .unwrap(),
            as_privkey
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes()
                .to_vec(),
        )
    }

    #[test]
    fn it_works() {
        // Setup wallet provider
        let (account_server, account_server_pubkey) = new_account_server();

        // Setup wallet
        let salt = crate::wallet::pin_key::new_pin_salt();
        let pin = "123456";
        let hw_privkey = SigningKey::random(&mut OsRng);

        // Register
        let challenge = account_server.registration_challenge().unwrap();
        let registration_message =
            instructions::Registration::new_signed(&hw_privkey, &salt, pin, &challenge).unwrap();
        let cert = account_server.register(registration_message).unwrap();

        // Verify the certificate
        let cert_data = cert.parse_and_verify(&account_server_pubkey).unwrap();
        dbg!(&cert, &cert_data);
        assert_eq!(cert_data.iss, account_server.name);
        assert_eq!(cert_data.hw_pubkey.0, *hw_privkey.verifying_key());
    }

    #[test]
    fn reqwest() {
        dbg!(String::from_utf8(
            RemoteAccountServer::new(
                "https://SSSS".to_owned(),
            )
            .registration_challenge()
            .unwrap()
        )
        .unwrap());
    }
}
