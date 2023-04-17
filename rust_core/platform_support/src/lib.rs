pub mod hw_keystore;
pub mod utils;

#[cfg(feature = "hardware")]
mod bridge;

#[cfg(feature = "integration-test")]
pub mod integration_test;

// this prevents a compilation warning, see bridge/mod.rs
#[cfg(feature = "hardware")]
use bridge::uniffi_reexport_hack;

// if the hardware feature is enabled, prefer hardware implementations
#[cfg(feature = "hardware")]
pub mod preferred {
    use crate::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};
    use crate::utils::hardware::HardwareUtilities;

    pub type PlatformEcdsaKey = HardwareEcdsaKey;
    pub type PlatformEncryptionKey = HardwareEncryptionKey;
    pub type PlatformUtilities = HardwareUtilities;
}

// otherwise if the software feature is enabled, prefer software fallbacks
#[cfg(all(not(feature = "hardware"), feature = "software"))]
pub mod preferred {
    use crate::hw_keystore::software::{SoftwareEcdsaKey, SoftwareEncryptionKey};
    use crate::utils::software::SoftwareUtilities;

    pub type PlatformEcdsaKey = SoftwareEcdsaKey;
    pub type PlatformEncryptionKey = SoftwareEncryptionKey;
    pub type PlatformUtilities = SoftwareUtilities;
}

// otherwise just just alias the Never type
#[cfg(not(any(feature = "hardware", feature = "software")))]
pub mod preferred {
    use never::Never;

    pub type PlatformEcdsaKey = Never;
    pub type PlatformEncryptionKey = Never;
    pub type PlatformUtilities = Never;
}
