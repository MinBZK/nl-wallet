pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware")]
mod bridge;

#[cfg(feature = "integration-test")]
pub mod integration_test;

// this prevents a compilation warning, see bridge/mod.rs
#[cfg(feature = "hardware")]
use bridge::uniffi_reexport_hack;
