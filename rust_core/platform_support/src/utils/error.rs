// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error)]
pub enum UtilitiesError {
    #[error("Platform error: {reason}")]
    PlatformError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}
