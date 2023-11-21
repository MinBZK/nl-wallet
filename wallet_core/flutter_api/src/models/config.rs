use wallet::WalletConfiguration;

pub struct FlutterConfiguration {
    pub inactive_lock_timeout: u16,
    pub background_lock_timeout: u16,
}

impl From<&WalletConfiguration> for FlutterConfiguration {
    fn from(value: &WalletConfiguration) -> Self {
        FlutterConfiguration {
            inactive_lock_timeout: value.lock_timeouts.inactive_timeout,
            background_lock_timeout: value.lock_timeouts.background_timeout,
        }
    }
}
