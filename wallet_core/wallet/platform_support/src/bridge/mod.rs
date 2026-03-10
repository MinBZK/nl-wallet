pub mod attested_key;
pub mod close_proximity_disclosure;
pub mod hw_keystore;
pub mod utils;

use std::sync::Arc;
use std::sync::OnceLock;

use crate::bridge::attested_key::AttestedKeyBridge;
use crate::bridge::close_proximity_disclosure::CloseProximityDisclosureBridge;
use crate::bridge::hw_keystore::EncryptionKeyBridge;
use crate::bridge::hw_keystore::SigningKeyBridge;
use crate::bridge::utils::UtilitiesBridge;

static PLATFORM_SUPPORT: OnceLock<PlatformSupport> = OnceLock::new();

#[derive(Debug)]
struct PlatformSupport {
    signing_key: Arc<dyn SigningKeyBridge>,
    encryption_key: Arc<dyn EncryptionKeyBridge>,
    attested_key: Arc<dyn AttestedKeyBridge>,
    utils: Arc<dyn UtilitiesBridge>,
    close_proximity_disclosure: Arc<dyn CloseProximityDisclosureBridge>,
}

pub fn init_platform_support(
    signing_key: Arc<dyn SigningKeyBridge>,
    encryption_key: Arc<dyn EncryptionKeyBridge>,
    attested_key: Arc<dyn AttestedKeyBridge>,
    utils: Arc<dyn UtilitiesBridge>,
    close_proximity_disclosure: Arc<dyn CloseProximityDisclosureBridge>,
) {
    let platform_support = PlatformSupport {
        signing_key,
        encryption_key,
        attested_key,
        utils,
        close_proximity_disclosure,
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
