use wallet::LockTimeoutConfiguration;

pub struct FlutterConfiguration {
    pub inactive_lock_timeout: u16,
    pub background_lock_timeout: u16,
}

impl From<&LockTimeoutConfiguration> for FlutterConfiguration {
    fn from(value: &LockTimeoutConfiguration) -> Self {
        FlutterConfiguration {
            inactive_lock_timeout: value.inactive_timeout,
            background_lock_timeout: value.background_timeout,
        }
    }
}
