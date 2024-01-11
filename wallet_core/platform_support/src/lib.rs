mod bridge;

pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware-integration-test")]
pub mod hardware_test;

// import generated Rust bindings
use crate::bridge::{
    hw_keystore::{EncryptionKeyBridge, KeyStoreError, SigningKeyBridge},
    init_platform_support,
    utils::{UtilitiesBridge, UtilitiesError},
};

uniffi::include_scaffolding!("platform_support");
