pub mod hw_keystore;
pub mod utils;

use self::{
    hw_keystore::{init_hw_keystore, EncryptionKeyBridge, KeyStoreBridge, SigningKeyBridge},
    utils::{init_utilities, UtilitiesBridge, UtilitiesError},
};
use crate::hw_keystore::error::KeyStoreError;

// import generated Rust bindings
uniffi::include_scaffolding!("platform_support");

// This prevents a compilation warning that "uniffi_reexport_hack" is unused.
// It needs a top level use statement, see lib.rs
uniffi_reexport_scaffolding!();
