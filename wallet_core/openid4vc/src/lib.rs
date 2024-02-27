// Data structures implemening OAuth/OpenID(4VCI) protocol messages.
pub mod authorization;
pub mod credential;
pub mod token;

// Cryptographic tools.
pub mod dpop;
pub mod jwt;
pub mod pkce;

// Issuance code for the server and client.
pub mod issuance_session;
pub mod issuer;

// Errors used throughout the crate.
pub mod errors;
pub use errors::*;

#[cfg(feature = "mock")]
pub mod mock;

pub const NL_WALLET_CLIENT_ID: &str = "https://example.com";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    MsoMdoc,

    // Other formats we don't currently support; we include them here so we can give the appropriate error message
    // when they might be requested by the wallet (as opposed to a deserialization error).
    // The OpenID4VCI and OpenID4VP specs aim to be general and do not provide an exhaustive list; the formats below
    // are found as examples in the specs.
    LdpVc,
    JwtVc,
    JwtVcJson,
    AcVc, // Anonymous Credentials i.e. Idemix
}
