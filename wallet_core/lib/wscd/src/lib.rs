pub mod issuance;
#[cfg(feature = "mock")]
pub mod mock_remote;
pub mod payload;
pub mod wia;

#[cfg(feature = "mock")]
pub mod mock {
    pub const MOCK_WALLET_CLIENT_ID: &str = "mock_wallet_client_id";
}
