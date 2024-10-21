use std::fmt::Debug;

use uniffi::UnexpectedUniFFICallbackError;

use super::get_platform_support;

// Implementation of AttestedKeyError from UDL
#[derive(Debug, thiserror::Error)]
pub enum AttestedKeyError {
    #[error("key error: {reason}")]
    KeyError { reason: String },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// Implementation of IdentifierAttestedKeyError from UDL
#[derive(Debug, thiserror::Error)]
pub enum IdentifierAttestedKeyError {
    #[error("key error (retain_identifier: {retain_identifier}): {reason}")]
    KeyError { reason: String, retain_identifier: bool },
    #[error("bridging error: {reason}")]
    BridgingError { reason: String },
}

// This is required to catch UnexpectedUniFFICallbackError
impl From<UnexpectedUniFFICallbackError> for AttestedKeyError {
    fn from(value: UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// This is required to catch UnexpectedUniFFICallbackError
impl From<UnexpectedUniFFICallbackError> for IdentifierAttestedKeyError {
    fn from(value: UnexpectedUniFFICallbackError) -> Self {
        Self::BridgingError { reason: value.reason }
    }
}

// Implementation of AttestedKeyType from UDL
pub enum AttestedKeyType {
    Apple,
    Google,
}

// Implementation of AttestationData from UDL
pub enum AttestationData {
    Apple {
        attestation_data: Vec<u8>,
    },
    Google {
        certificate_chain: Vec<Vec<u8>>,
        app_attestation_token: Vec<u8>,
    },
}

// Implementation of AttestedKeyBridge from UDL
pub trait AttestedKeyBridge: Send + Sync + Debug {
    fn key_type(&self) -> Result<AttestedKeyType, AttestedKeyError>;
    fn generate_identifier(&self) -> Result<String, AttestedKeyError>;
    fn attest(&self, identifier: String, challenge: Vec<u8>) -> Result<AttestationData, IdentifierAttestedKeyError>;
    fn sign(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, AttestedKeyError>;

    // Only supported on Android
    fn public_key(&self, identifier: String) -> Result<Vec<u8>, AttestedKeyError>;
    fn delete(&self, identifier: String) -> Result<(), AttestedKeyError>;
}

pub fn get_attested_key_bridge() -> &'static dyn AttestedKeyBridge {
    get_platform_support().attested_key.as_ref()
}
