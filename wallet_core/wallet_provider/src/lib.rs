pub mod app;
pub mod app_dependencies;
pub mod errors;
pub mod server;
pub mod settings;

pub use wallet_provider_service::account_server::AccountServer;

#[cfg(feature = "stub")]
pub mod stub {
    pub use wallet_provider_domain::{generator::stub::EpochGenerator, model::FailingPinPolicy};
    pub use wallet_provider_service::account_server::stub::{account_server, TestDeps};
}
