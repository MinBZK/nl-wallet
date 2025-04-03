pub mod credential;
pub mod error;
pub mod jwk;
pub mod jwt;
pub mod pop;
pub mod wte;

pub use jwt::*;

// TODO: remove this nl-wallet specific constant: PVW-4192
/// Used as the `iss` field in various JWTs, identifying this wallet solution as the issuer of the JWT.
pub const NL_WALLET_CLIENT_ID: &str = "https://wallet.edi.rijksoverheid.nl";
