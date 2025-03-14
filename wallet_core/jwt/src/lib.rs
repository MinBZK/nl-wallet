pub mod credential;
pub mod error;
pub mod jwt;

pub use jwt::*;

/// Used as the `iss` field in various JWTs, identifying this wallet solution as the issuer of the JWT.
pub const NL_WALLET_CLIENT_ID: &str = "https://wallet.edi.rijksoverheid.nl";
