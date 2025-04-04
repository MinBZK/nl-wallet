pub mod error;
pub mod messages;
pub mod signed;

/// Used as the `iss` field in various JWTs, identifying this wallet solution as the issuer of the JWTs.
pub const NL_WALLET_CLIENT_ID: &str = "https://wallet.edi.rijksoverheid.nl";
