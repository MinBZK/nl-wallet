use chrono::SecondsFormat;

use wallet::configuration::WalletConfiguration;

pub struct FlutterConfiguration {
    pub inactive_warning_timeout: u16,
    pub inactive_lock_timeout: u16,
    pub background_lock_timeout: u16,
    pub pid_attestation_types: Vec<String>,
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
            pid_attestation_types: value.pid_attributes.pid_attestation_types().map(String::from).collect(),
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
