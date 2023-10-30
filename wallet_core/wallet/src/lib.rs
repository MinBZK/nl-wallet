mod account_provider;
mod config;
mod digid;
mod disclosure;
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
    config::{AccountServerConfiguration, Configuration, LockTimeoutConfiguration, PidIssuanceConfiguration},
    document::{
        Attribute, AttributeLabel, AttributeLabelLanguage, AttributeLabels, AttributeValue, Document,
        DocumentPersistence, DocumentType, GenderAttributeValue, MissingDisclosureAttributes,
    },
    pin::validation::validate_pin,
    wallet::{UriType, Wallet},
};

pub mod mdoc {
    pub use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
}
#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{
        account_provider::{AccountProviderClient, HttpAccountProviderClient},
        config::{ConfigurationRepository, LocalConfigurationRepository},
        digid::{DigidSession, HttpDigidSession},
        disclosure::MdocDisclosureSession,
        pid_issuer::{HttpPidIssuerClient, PidIssuerClient},
        storage::Storage,
    };
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient, digid::MockDigidSession, pid_issuer::MockPidIssuerClient,
        storage::MockStorage,
    };
}
