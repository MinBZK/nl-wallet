use wallet::configuration::WalletConfiguration;

pub struct FlutterConfiguration {
    pub inactive_warning_timeout: u16,
    pub inactive_lock_timeout: u16,
    pub background_lock_timeout: u16,
    pub static_assets_base_url: String,
    pub version: u64,
}

impl From<&WalletConfiguration> for FlutterConfiguration {
    fn from(value: &WalletConfiguration) -> Self {
        FlutterConfiguration {
            inactive_warning_timeout: value.lock_timeouts.warning_timeout,
            inactive_lock_timeout: value.lock_timeouts.inactive_timeout,
            background_lock_timeout: value.lock_timeouts.background_timeout,
            static_assets_base_url: value.static_assets_base_url.to_string(),
            version: value.version,
        }
    }
}
