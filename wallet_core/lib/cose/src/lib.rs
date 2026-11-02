//! Shared COSE signing, verification, and typed-payload support.
//!
//! This crate owns protocol-independent COSE primitives and the WRPRC CWT profile. Other protocol-specific behavior,
//! such as mdoc WSCD batch signing and ISO data structures, belongs in the corresponding protocol crate.

mod algorithm;
mod cbor_int_map;
mod error;
mod key;
mod message;
mod serialization;
mod sign1;

pub mod wrprc_cwt;

pub use algorithm::*;
pub use cbor_int_map::*;
pub use error::*;
pub use key::*;
pub use message::*;
pub use serialization::*;
pub use sign1::*;
