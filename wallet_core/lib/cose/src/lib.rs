//! Shared COSE signing, verification, and typed-payload support.
//!
//! This crate owns protocol-independent COSE primitives and CWT handling. Protocol-specific behavior, such as mdoc
//! WSCD batch signing and ISO data structures, belongs in the corresponding protocol crate.

mod algorithm;
mod error;
mod key;
mod message;
mod serialization;
mod sign1;

pub mod cwt;

pub use algorithm::*;
pub use error::*;
pub use key::*;
pub use message::*;
pub use serialization::CborError;
pub use sign1::*;
