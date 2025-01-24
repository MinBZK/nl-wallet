#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "test_common")]
pub mod common;
#[cfg(feature = "test_common")]
pub mod utils;

#[cfg(feature = "fake_digid")]
pub mod fake_digid;

#[cfg(any(feature = "performance_test", feature = "gba_pid_test"))]
pub mod default {
    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AttestationEnvironment;

    pub fn attestation_environment() -> AttestationEnvironment {
        AttestationEnvironment::Development
    }

    pub fn app_identifier() -> AppIdentifier {
        // This is the default iOS team and bundle identifier configured for the Wallet Provider.
        AppIdentifier::new("XGL6UKBPLP", "nl.ictu.edi.wallet.latest")
    }
}
