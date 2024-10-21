pub mod attested_key;
pub mod hw_keystore;
pub mod utils;

use std::sync::OnceLock;

use self::{
    attested_key::AttestedKeyBridge,
    hw_keystore::{EncryptionKeyBridge, SigningKeyBridge},
    utils::UtilitiesBridge,
};

static PLATFORM_SUPPORT: OnceLock<PlatformSupport> = OnceLock::new();

#[derive(Debug)]
struct PlatformSupport {
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    attested_key: Box<dyn AttestedKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
}

pub fn init_platform_support(
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    attested_key: Box<dyn AttestedKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
) {
    let platform_support = PlatformSupport {
        signing_key,
        encryption_key,
        attested_key,
        utils,
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
