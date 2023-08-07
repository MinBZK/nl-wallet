pub mod hw_keystore;
pub mod utils;

mod bridge;

#[cfg(feature = "integration-test")]
pub mod integration_test;

// import generated Rust bindings
use crate::bridge::{
    hw_keystore::{EncryptionKeyBridge, KeyStoreError, SigningKeyBridge},
    init_platform_support,
    utils::{UtilitiesBridge, UtilitiesError},
};

uniffi::include_scaffolding!("platform_support");
