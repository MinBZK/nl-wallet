use std::fmt::Debug;

use async_trait::async_trait;

use super::get_platform_support;

// Implementation of AttestedKeyError from UDL
#[derive(Debug, thiserror::Error)]
pub enum AttestedKeyError {
    #[error("key/app attestation is not supported on this device")]
    AttestationNotSupported,
    #[error("the called method is not impemented for this platform")]
    MethodUnimplemented,
    #[error("vendor server is unreachable: {details}")]
    ServerUnreachable { details: String },
    #[error("{reason}")]
    Other { reason: String },
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
// Unfortunately we cannot use the built-in support for traits with async methods,
// as those are not object safe, so we have to rely on the async_trait crate instead.
#[async_trait]
pub trait AttestedKeyBridge: Send + Sync + Debug {
    fn key_type(&self) -> AttestedKeyType;
    async fn generate_identifier(&self) -> Result<String, AttestedKeyError>;
    async fn attest(&self, identifier: String, challenge: Vec<u8>) -> Result<AttestationData, AttestedKeyError>;
    async fn sign(&self, identifier: String, payload: Vec<u8>) -> Result<Vec<u8>, AttestedKeyError>;

    // Only supported on Android
    async fn public_key(&self, identifier: String) -> Result<Vec<u8>, AttestedKeyError>;
    async fn delete(&self, identifier: String) -> Result<(), AttestedKeyError>;
}

pub fn get_attested_key_bridge() -> &'static dyn AttestedKeyBridge {
    get_platform_support().attested_key.as_ref()
}
