mod account_server;
mod config;
mod digid;
mod init;
mod lock;
mod pid_issuer;
mod pin;
mod storage;
mod utils;

pub mod wallet;

pub use crate::{
    config::{AccountServerConfiguration, LockTimeoutConfiguration},
    init::{init_wallet, Wallet},
    pin::validation::{validate_pin, PinValidationError},
};

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_server::RemoteAccountServerClient, config::MockConfigurationRepository, storage::MockStorage,
    };
}
