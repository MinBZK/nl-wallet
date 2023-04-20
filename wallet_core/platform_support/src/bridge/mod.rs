pub mod hw_keystore;
pub mod utils;

use once_cell::sync::OnceCell;

use self::{
    hw_keystore::{EncryptionKeyBridge, KeyStoreError, SigningKeyBridge},
    utils::{UtilitiesBridge, UtilitiesError},
};

// import generated Rust bindings
uniffi::include_scaffolding!("platform_support");

// This prevents a compilation warning that "uniffi_reexport_hack" is unused.
// It needs a top level use statement, see lib.rs
uniffi_reexport_scaffolding!();

static BRIDGE_COLLECTION: OnceCell<BridgeCollection> = OnceCell::new();

#[derive(Debug)]
struct BridgeCollection {
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
}

pub fn init_platform_support(
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
) {
    let bridge_collection = BridgeCollection {
        signing_key,
        encryption_key,
        utils,
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
