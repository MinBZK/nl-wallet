mod account_provider;
mod config;
mod digid;
mod document;
mod instruction;
mod lock;
mod pid_issuer;
mod pin;
mod pkce;
mod storage;
mod utils;
mod wallet;

pub mod errors;

pub use crate::{
    account_provider::AccountProviderClient,
    config::{
        AccountServerConfiguration, Configuration, ConfigurationRepository, LockTimeoutConfiguration,
        PidIssuanceConfiguration,
    },
    digid::DigidSession,
    document::{
        Attribute, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document, DocumentPersistence, DocumentType,
        GenderAttributeValue,
    },
    pid_issuer::PidIssuerClient,
    pin::validation::validate_pin,
    storage::Storage,
    wallet::{UriType, Wallet},
};

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{
        account_provider::HttpAccountProviderClient, config::LocalConfigurationRepository, digid::HttpDigidSession,
        instruction::RemoteEcdsaKey, pid_issuer::HttpPidIssuerClient,
    };
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient, digid::MockDigidSession, pid_issuer::MockPidIssuerClient,
        storage::MockStorage,
    };
}
