mod account_provider;
mod attestation;
mod config;
mod digid;
mod instruction;
mod lock;
mod pin;
mod repository;
mod reqwest;
mod storage;
mod update_policy;
mod wallet;

pub mod errors;

pub use crate::attestation::AttestationAttribute;
pub use crate::attestation::AttestationAttributeValue;
pub use crate::attestation::AttestationIdentity;
pub use crate::attestation::AttestationPresentation;
pub use crate::pin::validation::validate_pin;
pub use crate::storage::DisclosureStatus;
pub use crate::storage::WalletEvent;
pub use crate::wallet::DisclosureProposalPresentation;
pub use crate::wallet::DisclosureUriSource;
pub use crate::wallet::LockCallback;
pub use crate::wallet::UnlockMethod;
pub use crate::wallet::UriType;
pub use crate::wallet::Wallet;
pub use crate::wallet::WalletClients;

pub mod attestation_data {
    pub use attestation_data::attributes::Attribute;
    pub use attestation_data::attributes::AttributeValue;
    pub use attestation_data::auth::Image;
    pub use attestation_data::auth::LocalizedStrings;
    pub use attestation_data::auth::Organization;
    pub use attestation_data::auth::reader_auth::DeletionPolicy;
    pub use attestation_data::auth::reader_auth::ReaderRegistration;
    pub use attestation_data::auth::reader_auth::RetentionPolicy;
    pub use attestation_data::auth::reader_auth::SharingPolicy;
    pub use attestation_data::disclosure_type::DisclosureType;
}

pub mod configuration {
    pub use wallet_configuration::wallet_config::AccountServerConfiguration;
    pub use wallet_configuration::wallet_config::DisclosureConfiguration;
    pub use wallet_configuration::wallet_config::LockTimeoutConfiguration;
    pub use wallet_configuration::wallet_config::PidIssuanceConfiguration;
    pub use wallet_configuration::wallet_config::WalletConfiguration;
}

pub mod openid4vc {
    pub use openid4vc::verifier::SessionType;
}

pub mod sd_jwt_vc_metadata {
    pub use sd_jwt_vc_metadata::ClaimDisplayMetadata;
    pub use sd_jwt_vc_metadata::DisplayMetadata;
    pub use sd_jwt_vc_metadata::Image;
    pub use sd_jwt_vc_metadata::LogoMetadata;
    pub use sd_jwt_vc_metadata::RenderingMetadata;
}

pub mod utils {
    pub use http_utils::urls::BaseUrl;
    pub use update_policy_model::update_policy::VersionState;
    pub use utils::built_info::version_string;
}

#[cfg(feature = "wallet_deps")]
pub mod wallet_deps {
    pub use crate::account_provider::AccountProviderClient;
    pub use crate::account_provider::HttpAccountProviderClient;
    pub use crate::config::FileStorageConfigurationRepository;
    pub use crate::config::HttpConfigurationRepository;
    pub use crate::config::WalletConfigurationRepository;
    pub use crate::config::default_config_server_config;
    pub use crate::config::default_wallet_config;
    pub use crate::digid::DigidClient;
    pub use crate::digid::DigidSession;
    pub use crate::digid::HttpDigidClient;
    pub use crate::repository::BackgroundUpdateableRepository;
    pub use crate::repository::Repository;
    pub use crate::repository::RepositoryUpdateState;
    pub use crate::repository::UpdateableRepository;
    pub use crate::storage::Storage;
    pub use crate::update_policy::HttpUpdatePolicyRepository;
    pub use crate::update_policy::UpdatePolicyRepository;
}

#[cfg(feature = "mock")]
pub mod mock {
    pub use crate::account_provider::MockAccountProviderClient;
    pub use crate::attestation::BSN_ATTR_NAME;
    pub use crate::attestation::PID_DOCTYPE;
    pub use crate::config::LocalConfigurationRepository;
    pub use crate::digid::MockDigidClient;
    pub use crate::digid::MockDigidSession;
    pub use crate::storage::StorageStub;
    pub use crate::update_policy::MockUpdatePolicyRepository;
}
