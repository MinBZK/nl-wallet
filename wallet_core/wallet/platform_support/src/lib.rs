#![allow(clippy::empty_line_after_doc_comments)] // the generated code included below has this issue
mod bridge;

pub mod attested_key;
pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware_integration_test")]
pub mod hardware_test;

// import generated Rust bindings
use crate::bridge::attested_key::AttestationData;
use crate::bridge::attested_key::AttestedKeyBridge;
use crate::bridge::attested_key::AttestedKeyError;
use crate::bridge::attested_key::AttestedKeyType;
use crate::bridge::hw_keystore::EncryptionKeyBridge;
use crate::bridge::hw_keystore::KeyStoreError;
use crate::bridge::hw_keystore::SigningKeyBridge;
use crate::bridge::init_platform_support;
use crate::bridge::utils::UtilitiesBridge;
use crate::bridge::utils::UtilitiesError;

uniffi::include_scaffolding!("platform_support");
