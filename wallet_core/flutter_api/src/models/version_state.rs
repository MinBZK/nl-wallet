use wallet::wallet_common::VersionState;

pub enum FlutterVersionState {
    Ok,
    Notify,
    Recommend,
    Warn { expires_in_seconds: u64 },
    Block,
}

impl From<VersionState> for FlutterVersionState {
    fn from(value: VersionState) -> Self {
        match value {
            VersionState::Ok => FlutterVersionState::Ok,
            VersionState::Notify => FlutterVersionState::Notify,
            VersionState::Recommend => FlutterVersionState::Recommend,
            VersionState::Warn(duration) => FlutterVersionState::Warn {
                expires_in_seconds: duration.as_secs(),
            },
            VersionState::Block => FlutterVersionState::Block,
        }
    }
}
