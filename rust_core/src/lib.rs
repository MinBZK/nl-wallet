// Prevent dead code warnings since the lower 4 modules are not exposed in the `api` module yet.
// TODO: remove this when these modules are used.
#![allow(dead_code)]

/// Functions callable by Flutter.
mod api;

/// Generated code for the Flutter bridge using `flutter_rust_bridge_codegen`.
mod bridge_generated;

mod jwt;
mod serialization;
pub mod utils;

pub mod wallet;
pub mod wp;
