pub mod error;
pub mod issuance;
pub mod poa;
pub mod wia;

pub use error::PoaError;
pub use error::PoaVerificationError;
pub use poa::Poa;
pub use poa::PoaPayload;

#[cfg(feature = "mock")]
pub mod mock {
    pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";
}
