//! This module contains models that represent data coming from `wallet-core` that should be represented in Flutter.
//!
//! Flutter-rust-bridge (frb) offers functionality to scan external crates and generate bridging code for it. However,
//! there are quite some limitions to this approach:
//! - unit structs cannot be parsed;
//! - frb needs to parse and generate code for every type that is used in Flutter. Examples of external types that cause
//!   problems are IndexMap and Uri;
//!
//! This leaves us to either use a subset of Rust for the models that need to be translated or create separate mapping
//! models. The former option places severe limitions on how Rust can be used in `wallet-core`. The latter option is
//! chosen here, even though it causes a lot duplication.

pub mod attestation;
pub mod config;
pub mod disclosure;
pub mod instruction;
pub mod localize;
pub mod pin;
pub mod uri;
pub mod version_state;
pub mod wallet_event;
