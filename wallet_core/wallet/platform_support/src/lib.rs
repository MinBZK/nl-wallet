mod bridge;

pub mod attested_key;
pub mod close_proximity_disclosure;
pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware_integration_test")]
pub mod hardware_test;

// import generated Rust bindings
use crate::bridge::attested_key::AttestationData;
use crate::bridge::attested_key::AttestedKeyBridge;
use crate::bridge::attested_key::AttestedKeyError;
use crate::bridge::attested_key::AttestedKeyType;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureBridge;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureChannel;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureError;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureUpdate;
use crate::bridge::hw_keystore::EncryptionKeyBridge;
use crate::bridge::hw_keystore::KeyStoreError;
use crate::bridge::hw_keystore::SigningKeyBridge;
use crate::bridge::init_platform_support;
use crate::bridge::utils::UtilitiesBridge;
use crate::bridge::utils::UtilitiesError;

uniffi::include_scaffolding!("platform_support");
