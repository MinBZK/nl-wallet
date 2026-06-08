use chrono::SecondsFormat;
use wallet::configuration::WalletConfiguration;

use super::attestation::Format;

pub struct FlutterConfiguration {
    pub inactive_warning_timeout: u16,
    pub inactive_lock_timeout: u16,
    pub background_lock_timeout: u16,
    pub pid_attestations: Vec<PidAttestation>,
    pub static_assets_base_url: String,
    pub maintenance_window: Option<(String, String)>,
    pub version: String,
    pub environment: String,
}

impl From<&WalletConfiguration> for FlutterConfiguration {
    fn from(value: &WalletConfiguration) -> Self {
        FlutterConfiguration {
            inactive_warning_timeout: value.lock_timeouts.warning_timeout,
            inactive_lock_timeout: value.lock_timeouts.inactive_timeout,
            background_lock_timeout: value.lock_timeouts.background_timeout,
            pid_attestations: PidAttestation::pid_attestations_from_config(&value.pid_attributes),
            static_assets_base_url: value.static_assets_base_url.to_string(),
            maintenance_window: value.maintenance_window.as_ref().map(|window| {
                (
                    window.start.to_rfc3339_opts(SecondsFormat::Secs, true),
                    window.end.to_rfc3339_opts(SecondsFormat::Secs, true),
                )
            }),
            version: value.version.to_string(),
            environment: value.environment.clone(),
        }
    }
}

pub struct PidAttestation {
    pub format: Format,
    pub attestation_type: String,
}

impl PidAttestation {
    /// Flutter should interpet this `Vec` as a prioritised list of formats / attestation types. The first PID
    /// credential that matches this tuple should be shown, while any following ones that match should be hidden from
    /// the user. Note that SD-JWT is prioritized as a format, meaning that if any SD-JWT PID credential is present, an
    /// mdoc credential will be hidden.
    fn pid_attestations_from_config(pid_config: &wallet::configuration::PidAttributesConfiguration) -> Vec<Self> {
        pid_config
            .sd_jwt
            .keys()
            .cloned()
            .map(|attestation_type| Self {
                format: Format::SdJwt,
                attestation_type,
            })
            .chain(pid_config.mso_mdoc.keys().cloned().map(|attestation_type| Self {
                format: Format::MsoMdoc,
                attestation_type,
            }))
            .collect()
    }
}
