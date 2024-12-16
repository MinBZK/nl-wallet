#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "test_common")]
pub mod common;
#[cfg(feature = "test_common")]
pub mod utils;

#[cfg(feature = "fake_digid")]
pub mod fake_digid;

#[cfg(any(feature = "performance_test", feature = "gba_pid_test"))]
pub fn default_deployed_app_identifier() -> apple_app_attest::AppIdentifier {
    // This is the default iOS team and bundle identifier configured for the Wallet Provider.
    apple_app_attest::AppIdentifier::new("XGL6UKBPLP", "nl.ictu.edi.wallet.latest")
}
