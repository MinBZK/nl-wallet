use serde::Deserialize;
use serde::Serialize;

// Data structures implemening OAuth/OpenID(4VCI) protocol messages.
pub mod authorization;
pub mod credential;
pub mod credential_formats;
pub mod token;

// Cryptographic tools.
pub mod dpop;
pub mod jwt;
pub mod pkce;

// Issuance code for the server and client.
pub mod attributes;
pub mod issuance_session;
pub mod issuer;

// Verification code for the server and client.
pub mod disclosure_session;
pub mod openid4vp;
pub mod presentation_exchange;
pub mod return_url;
pub mod verifier;

// Errors used throughout the crate.
pub mod errors;
pub use errors::*;

pub mod metadata;
pub mod oidc;
pub mod server_state;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    MsoMdoc,
    Jwt,

    // Other formats we don't currently support; we include them here so we can give the appropriate error message
    // when they might be requested by the wallet (as opposed to a deserialization error).
    // The OpenID4VCI and OpenID4VP specs aim to be general and do not provide an exhaustive list; the formats below
    // are found as examples in the specs.
    LdpVc,
    JwtVc,
    JwtVcJson,
    AcVc, // Anonymous Credentials i.e. Idemix
}
