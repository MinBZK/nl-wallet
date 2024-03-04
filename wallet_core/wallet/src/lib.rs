mod account_provider;
mod config;
mod disclosure;
mod document;
mod instruction;
mod lock;
mod pin;
mod storage;
mod utils;
mod wallet;

pub mod errors;

pub use crate::{
    document::{
        Attribute, AttributeLabel, AttributeLabelLanguage, AttributeLabels, AttributeValue, DisclosureDocument,
        Document, DocumentAttributes, DocumentPersistence, DocumentType, GenderAttributeValue,
        MissingDisclosureAttributes,
    },
    pin::validation::validate_pin,
    wallet::{
        ConfigCallback, DisclosureProposal, DocumentsCallback, EventStatus, HistoryEvent, LockCallback, UriType, Wallet,
    },
};

pub mod mdoc {
    pub use nl_wallet_mdoc::utils::{
        auth::{Image, ImageType, LocalizedStrings, Organization},
        reader_auth::{DeletionPolicy, ReaderRegistration, RetentionPolicy, SharingPolicy},
    };
}

pub mod openid4vc {
    pub use openid4vc::oidc::OidcError;
}

pub mod wallet_common {
    pub use wallet_common::config::wallet_config::{
        AccountServerConfiguration, BaseUrl, DisclosureConfiguration, LockTimeoutConfiguration,
        PidIssuanceConfiguration, WalletConfiguration,
    };
}

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::{
        account_provider::{AccountProviderClient, HttpAccountProviderClient},
        config::{
            ConfigServerConfiguration, ConfigurationRepository, ConfigurationUpdateState,
            FileStorageConfigurationRepository, HttpConfigurationRepository, UpdateableConfigurationRepository,
            UpdatingFileHttpConfigurationRepository,
        },
        disclosure::{
            MdocDisclosureMissingAttributes, MdocDisclosureProposal, MdocDisclosureSession, MdocDisclosureSessionState,
        },
        storage::Storage,
    };
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::{
        account_provider::MockAccountProviderClient,
        config::{default_configuration, LocalConfigurationRepository},
        disclosure::{MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal, MockMdocDisclosureSession},
        storage::MockStorage,
        utils::reqwest::default_reqwest_client_builder,
    };
}
