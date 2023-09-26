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
    digid::DigidClient,
    document::{
        Attribute, AttributeLabel, AttributeLabelLanguage, AttributeValue, Document, DocumentPersistence, DocumentType,
        GenderAttributeValue,
    },
    pid_issuer::PidIssuerClient,
    pin::validation::validate_pin,
    storage::Storage,
    wallet::{RedirectUriType, Wallet},
};

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{
        account_provider::HttpAccountProviderClient, config::LocalConfigurationRepository, digid::HttpDigidClient,
        instruction::RemoteEcdsaKey, pid_issuer::HttpPidIssuerClient,
    };
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient, digid::MockDigidClient, pid_issuer::MockPidIssuerClient,
        storage::MockStorage,
    };
}
