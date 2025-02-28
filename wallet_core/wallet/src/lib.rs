mod account_provider;
mod attestation;
mod config;
mod disclosure;
mod document;
mod instruction;
mod issuance;
mod lock;
mod pin;
mod repository;
mod storage;
mod update_policy;
mod wallet;
mod wte;

pub mod errors;

pub use crate::attestation::Attestation;
pub use crate::attestation::AttestationAttribute;
pub use crate::attestation::AttestationIdentity;
pub use crate::disclosure::DisclosureUriSource;
pub use crate::document::Attribute;
pub use crate::document::AttributeKey;
pub use crate::document::AttributeLabel;
pub use crate::document::AttributeLabelLanguage;
pub use crate::document::AttributeLabels;
pub use crate::document::AttributeValue;
pub use crate::document::Document;
pub use crate::document::DocumentAttributes;
pub use crate::document::DocumentPersistence;
pub use crate::document::DocumentType;
pub use crate::document::GenderAttributeValue;
pub use crate::pin::validation::validate_pin;
pub use crate::storage::DisclosureStatus;
pub use crate::storage::DisclosureType;
pub use crate::wallet::DisclosureProposal;
pub use crate::wallet::HistoryEvent;
pub use crate::wallet::LockCallback;
pub use crate::wallet::UnlockMethod;
pub use crate::wallet::UriType;
pub use crate::wallet::Wallet;

pub mod mdoc {
    pub use nl_wallet_mdoc::utils::auth::Image;
    pub use nl_wallet_mdoc::utils::auth::ImageType;
    pub use nl_wallet_mdoc::utils::auth::LocalizedStrings;
    pub use nl_wallet_mdoc::utils::auth::Organization;
    pub use nl_wallet_mdoc::utils::reader_auth::DeletionPolicy;
    pub use nl_wallet_mdoc::utils::reader_auth::ReaderRegistration;
    pub use nl_wallet_mdoc::utils::reader_auth::RetentionPolicy;
    pub use nl_wallet_mdoc::utils::reader_auth::SharingPolicy;
}

pub mod openid4vc {
    pub use openid4vc::attributes::Attribute;
    pub use openid4vc::attributes::AttributeValue;
    pub use openid4vc::verifier::SessionType;
}

pub mod sd_jwt {
    pub use sd_jwt::metadata::ClaimDisplayMetadata;
    pub use sd_jwt::metadata::DisplayMetadata;
    pub use sd_jwt::metadata::LogoMetadata;
    pub use sd_jwt::metadata::RenderingMetadata;
}

pub mod wallet_common {
    pub use wallet_common::built_info::version_string;
    pub use wallet_common::config::wallet_config::AccountServerConfiguration;
    pub use wallet_common::config::wallet_config::DisclosureConfiguration;
    pub use wallet_common::config::wallet_config::LockTimeoutConfiguration;
    pub use wallet_common::config::wallet_config::PidIssuanceConfiguration;
    pub use wallet_common::config::wallet_config::WalletConfiguration;
    pub use wallet_common::update_policy::VersionState;
    pub use wallet_common::urls::BaseUrl;
}

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::account_provider::AccountProviderClient;
    pub use crate::account_provider::HttpAccountProviderClient;
    pub use crate::config::default_config_server_config;
    pub use crate::config::default_wallet_config;
    pub use crate::config::FileStorageConfigurationRepository;
    pub use crate::config::HttpConfigurationRepository;
    pub use crate::config::WalletConfigurationRepository;
    pub use crate::disclosure::MdocDisclosureMissingAttributes;
    pub use crate::disclosure::MdocDisclosureProposal;
    pub use crate::disclosure::MdocDisclosureSession;
    pub use crate::disclosure::MdocDisclosureSessionState;
    pub use crate::issuance::DigidSession;
    pub use crate::issuance::HttpDigidSession;
    pub use crate::repository::BackgroundUpdateableRepository;
    pub use crate::repository::Repository;
    pub use crate::repository::RepositoryUpdateState;
    pub use crate::repository::UpdateableRepository;
    pub use crate::storage::Storage;
    pub use crate::update_policy::HttpUpdatePolicyRepository;
    pub use crate::update_policy::UpdatePolicyRepository;
    pub use crate::wte::WpWteIssuanceClient;
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::account_provider::MockAccountProviderClient;
    pub use crate::config::LocalConfigurationRepository;
    pub use crate::disclosure::MockMdocDisclosureMissingAttributes;
    pub use crate::disclosure::MockMdocDisclosureProposal;
    pub use crate::disclosure::MockMdocDisclosureSession;
    pub use crate::issuance::MockDigidSession;
    pub use crate::storage::MockStorage;
    pub use crate::update_policy::MockUpdatePolicyRepository;
}
