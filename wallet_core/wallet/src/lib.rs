mod account_provider;
mod attestation;
mod config;
mod disclosure;
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
pub use crate::attestation::AttestationAttributeValue;
pub use crate::attestation::AttestationIdentity;
pub use crate::disclosure::DisclosureUriSource;
pub use crate::pin::validation::validate_pin;
pub use crate::storage::DisclosureStatus;
pub use crate::storage::DisclosureType;
pub use crate::storage::WalletEvent;
pub use crate::wallet::DisclosureProposal;
pub use crate::wallet::LockCallback;
pub use crate::wallet::UnlockMethod;
pub use crate::wallet::UriType;
pub use crate::wallet::Wallet;

pub mod attestation_data {
    pub use attestation_data::attributes::Attribute;
    pub use attestation_data::attributes::AttributeValue;
    pub use attestation_data::auth::reader_auth::DeletionPolicy;
    pub use attestation_data::auth::reader_auth::ReaderRegistration;
    pub use attestation_data::auth::reader_auth::RetentionPolicy;
    pub use attestation_data::auth::reader_auth::SharingPolicy;
    pub use attestation_data::auth::Image;
    pub use attestation_data::auth::LocalizedStrings;
    pub use attestation_data::auth::Organization;
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
    pub use crate::issuance::BSN_ATTR_NAME;
    pub use crate::issuance::PID_DOCTYPE;
    pub use crate::storage::MockStorage;
    pub use crate::update_policy::MockUpdatePolicyRepository;
}
