mod account_provider;
mod config;
mod digid;
mod init;
mod lock;
mod pid_issuer;
mod pin;
mod pkce;
mod storage;
mod utils;

pub mod errors;
pub mod wallet;

pub use crate::{
    config::{AccountServerConfiguration, LockTimeoutConfiguration},
    init::{init_wallet, Wallet},
    pin::validation::validate_pin,
};

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{account_provider::HttpAccountProviderClient, digid::HttpDigidClient, pid_issuer::PidIssuerClient};
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient, config::MockConfigurationRepository, digid::MockDigidClient,
        pid_issuer::MockPidRetriever, storage::MockStorage,
    };
}
