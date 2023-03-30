pub mod hw_keystore;

#[cfg(feature = "hardware")]
mod bridge;

// this prevents a compilation warning, see bridge/mod.rs
#[cfg(feature = "hardware")]
use bridge::uniffi_reexport_hack;
