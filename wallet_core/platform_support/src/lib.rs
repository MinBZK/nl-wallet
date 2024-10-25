mod bridge;

pub mod attested_key;
pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware_integration_test")]
pub mod hardware_test;

// import generated Rust bindings
use crate::bridge::{
    attested_key::{AttestationData, AttestedKeyBridge, AttestedKeyError, AttestedKeyType},
    hw_keystore::{EncryptionKeyBridge, KeyStoreError, SigningKeyBridge},
    init_platform_support,
    utils::{UtilitiesBridge, UtilitiesError},
};

uniffi::include_scaffolding!("platform_support");
