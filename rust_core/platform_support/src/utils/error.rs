// implementation of UtilitiesError from UDL
#[derive(Debug, thiserror::Error)]
pub enum UtilitiesError {
    #[error("Bridging error: {reason:?}")]
    BridgingError { reason: String },
}
