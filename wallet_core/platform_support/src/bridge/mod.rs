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

static PLATFORM_SUPPORT: OnceCell<PlatformSupport> = OnceCell::new();

struct PlatformSupport {
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
    _sentry_guard: Option<ClientInitGuard>,
}

// Debug cannot be derived, because ClientInitGuard doesn't implement it.
impl std::fmt::Debug for PlatformSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlatformSupport")
            .field("signing_key", &self.signing_key)
            .field("encryption_key", &self.encryption_key)
            .field("utils", &self.utils)
            .finish_non_exhaustive()
    }
}

pub fn init_platform_support(
    signing_key: Box<dyn SigningKeyBridge>,
    encryption_key: Box<dyn EncryptionKeyBridge>,
    utils: Box<dyn UtilitiesBridge>,
) {
    std::env::set_var("RUST_BACKTRACE", "1");

    let sentry_guard = init_sentry();

    let platform_support = PlatformSupport {
        signing_key,
        encryption_key,
        utils,
        _sentry_guard: sentry_guard,
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
