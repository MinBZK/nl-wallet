pub mod instructions;
mod jwt;
mod serialization;
pub mod server;
mod signed;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::account::client::{
    instructions::Registration,
    jwt::{Jwt, JwtClaims},
    serialization::{Base64Bytes, DerVerifyingKey},
    signed::SignedDouble,
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

pub trait AccountServerClient {
    fn registration_challenge(&self) -> Result<Vec<u8>>;
    fn register(&self, registration_message: SignedDouble<Registration>) -> Result<WalletCertificate>;
}
