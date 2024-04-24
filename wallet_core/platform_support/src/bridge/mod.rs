pub mod hw_keystore;
pub mod sentry;
pub mod utils;

use once_cell::sync::OnceCell;
use sentry::ClientInitGuard;

use self::{
    hw_keystore::{EncryptionKeyBridge, SigningKeyBridge},
    sentry::init_sentry,
    utils::UtilitiesBridge,
};

static BRIDGE_COLLECTION: OnceCell<BridgeCollection> = OnceCell::new();

struct BridgeCollection {
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
    _sentry_guard: Option<ClientInitGuard>,
}

// Debug cannot be derived, because ClientInitGuard doesn't implement it.
impl std::fmt::Debug for BridgeCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeCollection")
            .field("signing_key", &self.signing_key)
            .field("encryption_key", &self.encryption_key)
            .field("utils", &self.utils)
            .field("_sentry_guard", &"<HIDDEN>")
            .finish()
    }
}

pub fn init_platform_support(
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
) {
    std::env::set_var("RUST_BACKTRACE", "1");

    let sentry_guard = init_sentry();

    let bridge_collection = BridgeCollection {
        signing_key,
        encryption_key,
        utils,
        _sentry_guard: sentry_guard,
    };

    BRIDGE_COLLECTION
        .set(bridge_collection)
        .expect("Cannot call init_platform_support() more than once");
}

fn get_bridge_collection() -> &'static BridgeCollection {
    // crash if BRIDGES is not yet set
    BRIDGE_COLLECTION
        .get()
        .expect("BRIDGES used before init_platform_support() was called")
}
