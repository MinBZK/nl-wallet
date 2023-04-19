use anyhow::{anyhow, Result};
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use wallet_shared::{
    account::{
        instructions::Registration,
        jwt::{EcdsaDecodingKey, Jwt, JwtClaims},
        serialization::Base64Bytes,
        signed::SignedDouble,
        AccountServerClient, WalletCertificate, WalletCertificateClaims,
    },
    utils::{random_bytes, random_string},
};

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

impl AccountServerClient for AccountServer {
    fn registration_challenge(&self) -> Result<Vec<u8>> {
        AccountServer::registration_challenge(self)
    }
    fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
        AccountServer::register(self, registration_message)
    }
}

impl AccountServer {
    pub fn new(privkey: Vec<u8>, pin_hash_salt: Vec<u8>, name: String) -> Result<AccountServer> {
        let pubkey = EcdsaDecodingKey::from_sec1(
            SigningKey::from_pkcs8_der(&privkey)
                .map_err(anyhow::Error::msg)?
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

    pub fn new_stub() -> AccountServer {
        let account_server_privkey = SigningKey::random(&mut OsRng);
        AccountServer::new(
            account_server_privkey.to_pkcs8_der().unwrap().as_bytes().to_vec(),
            random_bytes(32),
            "stub_account_server".into(),
        )
        .unwrap()
    }

    // Only used for registration. When a registered user sends an instruction, we should store
    // the challenge per user, instead globally.
    pub fn registration_challenge(&self) -> Result<Vec<u8>> {
        let challenge = Jwt::sign(
            &RegistrationChallengeClaims {
                wallet_id: random_string(32),
                random: random_bytes(32).into(),
                exp: jsonwebtoken::get_current_timestamp() + 60,
            },
            &self.privkey,
        )?
        .0
        .as_bytes()
        .to_vec();
        Ok(challenge)
    }

    fn verify_registration_challenge(&self, challenge: &[u8]) -> Result<RegistrationChallengeClaims> {
        Jwt::parse_and_verify(&String::from_utf8(challenge.to_owned())?.into(), &self.pubkey)
    }

    pub fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate> {
        // We don't have the public keys yet against which to verify the message, as those are contained within the
        // message (like in X509 certificate requests). So first parse it to grab the public keys from it.
        let unverified = registration_message.dangerous_parse_unverified()?;

        let challenge = &unverified.challenge.0;
        let wallet_id = self.verify_registration_challenge(challenge)?.wallet_id;

        let hw_pubkey = unverified.payload.hw_pubkey.0;
        let pin_pubkey = unverified.payload.pin_pubkey.0;
        let signed = registration_message.parse_and_verify(challenge, &hw_pubkey, &pin_pubkey)?;

        if signed.serial_number != 0 {
            return Err(anyhow!("serial_number was {}, expected 0", signed.serial_number));
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
