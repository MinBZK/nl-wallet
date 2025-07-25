pub mod error;
pub mod keyfactory;
pub mod poa;

#[cfg(any(test, feature = "mock_remote_key"))]
pub mod mock_remote;

pub static POA_JWT_TYP: &str = "poa+jwt";

pub use error::PoaError;
pub use error::PoaVerificationError;
pub use poa::Poa;
pub use poa::PoaPayload;

#[cfg(feature = "mock")]
pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";
