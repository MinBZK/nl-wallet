#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "test_common")]
pub mod common;
#[cfg(feature = "test_common")]
pub mod utils;

#[cfg(feature = "fake_digid")]
pub mod fake_digid;

#[cfg(feature = "performance_test")]
pub mod default {
    use apple_app_attest::AppIdentifier;
    use apple_app_attest::AttestationEnvironment;

    pub fn attestation_environment() -> AttestationEnvironment {
        option_env!("APPLE_ATTESTATION_ENVIRONMENT")
            .map(|environment| match environment {
                "development" => AttestationEnvironment::Development,
                "production" => AttestationEnvironment::Production,
                _ => panic!("Invalid Apple attestation environment"),
            })
            .unwrap_or(AttestationEnvironment::Development)
    }

    pub fn app_identifier() -> AppIdentifier {
        let team_identifier = option_env!("TEAM_IDENTIFIER");
        let bundle_identifier = option_env!("BUNDLE_IDENTIFIER");

        // Create an iOS app identifier if both environment variables are provided, otherwise fall back to the default.
        if let (Some(team_identifier), Some(bundle_identifier)) = (team_identifier, bundle_identifier) {
            AppIdentifier::new(team_identifier, bundle_identifier)
        } else {
            AppIdentifier::new("XGL6UKBPLP", "nl.ictu.edi.wallet.latest")
        }
    }
}
