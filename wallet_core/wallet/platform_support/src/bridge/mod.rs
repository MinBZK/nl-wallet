pub mod attested_key;
pub mod hw_keystore;
pub mod iso18013_5;
pub mod utils;

use std::sync::Arc;
use std::sync::OnceLock;

use crate::bridge::attested_key::AttestedKeyBridge;
use crate::bridge::hw_keystore::EncryptionKeyBridge;
use crate::bridge::hw_keystore::SigningKeyBridge;
use crate::bridge::iso18013_5::Iso18013_5Bridge;
use crate::bridge::utils::UtilitiesBridge;

static PLATFORM_SUPPORT: OnceLock<PlatformSupport> = OnceLock::new();

#[derive(Debug)]
struct PlatformSupport {
    signing_key: Arc<dyn SigningKeyBridge>,
    encryption_key: Arc<dyn EncryptionKeyBridge>,
    attested_key: Arc<dyn AttestedKeyBridge>,
    utils: Arc<dyn UtilitiesBridge>,
    iso18013_5: Arc<dyn Iso18013_5Bridge>,
}

pub fn init_platform_support(
    signing_key: Arc<dyn SigningKeyBridge>,
    encryption_key: Arc<dyn EncryptionKeyBridge>,
    attested_key: Arc<dyn AttestedKeyBridge>,
    utils: Arc<dyn UtilitiesBridge>,
    iso18013_5: Arc<dyn Iso18013_5Bridge>,
) {
    let platform_support = PlatformSupport {
        signing_key,
        encryption_key,
        attested_key,
        utils,
        iso18013_5,
    };

    PLATFORM_SUPPORT
        .set(platform_support)
        .expect("Cannot call init_platform_support() more than once");
}

fn get_platform_support() -> &'static PlatformSupport {
    // crash if PLATFORM_SUPPORT is not yet set
    PLATFORM_SUPPORT
        .get()
        .expect("PLATFORM_SUPPORT used before init_platform_support() was called")
}
