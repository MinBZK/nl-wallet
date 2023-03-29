use thiserror::Error;

#[derive(Debug, Error)]
pub enum HardwareKeyStoreError {
    #[error(transparent)]
    KeyStoreError(#[from] KeyStoreError),
    #[error("Error decoding public key from hardware: {0}")]
    PublicKeyError(#[from] p256::pkcs8::spki::Error),
}

// implementation of KeyStoreError from UDL
#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("Key error: {reason}")]
    KeyError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}
