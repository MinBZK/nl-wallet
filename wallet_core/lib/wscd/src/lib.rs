pub mod error;
pub mod poa;
pub mod wscd;

#[cfg(any(test, feature = "mock"))]
pub mod mock_remote;

pub static POA_JWT_TYP: &str = "poa+jwt";

pub use error::PoaError;
pub use error::PoaVerificationError;
pub use poa::Poa;
pub use poa::PoaPayload;
