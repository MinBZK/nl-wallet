// Data structures defined in ISO 18013-5
pub mod iso;

// Functionality for the three main agents
pub mod holder;
pub mod issuer;
pub mod verifier;

pub mod issuer_shared;

pub mod cose;
pub mod serialization;

mod crypto;

#[cfg(test)]
mod examples;
#[cfg(test)]
mod tests;

pub use iso::*;

// TODO: check expiry of all certificates and credentials
