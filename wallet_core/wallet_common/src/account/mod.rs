pub mod instructions;
pub mod jwt;
pub mod serialization;
pub mod signed;
pub mod signing_key;

use serde::{Deserialize, Serialize};

use crate::account::{
    jwt::{Jwt, JwtClaims},
    serialization::{Base64Bytes, DerVerifyingKey},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletCertificateClaims {
    pub wallet_id: String,
    pub hw_pubkey: DerVerifyingKey,
    pub pin_pubkey_hash: Base64Bytes,
    pub version: u32,

    pub iss: String,
    pub iat: u64,
}

impl JwtClaims for WalletCertificateClaims {
    const SUB: &'static str = "wallet_certificate";
}

pub type WalletCertificate = Jwt<WalletCertificateClaims>;

#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub challenge: Base64Bytes,
}

#[derive(Serialize, Deserialize)]
pub struct Certificate {
    pub certificate: WalletCertificate,
}
