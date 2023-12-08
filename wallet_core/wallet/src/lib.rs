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
    document::{
        Attribute, AttributeLabel, AttributeLabelLanguage, AttributeLabels, AttributeValue, Document,
        DocumentAttributes, DocumentPersistence, DocumentType, GenderAttributeValue, MissingDisclosureAttributes,
        ProposedDisclosureDocument,
    },
    pin::validation::validate_pin,
    storage::{EventStatus, EventType, WalletEvent},
    wallet::{DisclosureProposal, UriType, Wallet},
};

pub mod mdoc {
    pub use nl_wallet_mdoc::utils::reader_auth::{
        DeletionPolicy, Image, ImageType, LocalizedStrings, Organization, ReaderRegistration, RetentionPolicy,
        SharingPolicy,
    };
}
pub mod x509 {
    pub use nl_wallet_mdoc::utils::x509::{Certificate, CertificateError, CertificateType};
}

pub use wallet_common::config::wallet_config::{LockTimeoutConfiguration, WalletConfiguration};

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{
        account_provider::{AccountProviderClient, HttpAccountProviderClient},
        config::{
            ConfigServerConfiguration, ConfigurationRepository, FileStorageConfigurationRepository,
            HttpConfigurationRepository,
        },
        digid::{DigidSession, HttpDigidSession},
        disclosure::{
            MdocDisclosureMissingAttributes, MdocDisclosureProposal, MdocDisclosureSession, MdocDisclosureSessionState,
        },
        pid_issuer::{HttpPidIssuerClient, PidIssuerClient},
        storage::Storage,
    };
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient,
        config::{default_configuration, LocalConfigurationRepository},
        digid::MockDigidSession,
        disclosure::MockMdocDisclosureSession,
        pid_issuer::MockPidIssuerClient,
        storage::MockStorage,
    };
}
