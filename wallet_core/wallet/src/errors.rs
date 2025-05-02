// Expose some downstream errors that may be useful.
pub mod reqwest {
    pub use reqwest::Error;
}

pub mod openid4vc {
    pub use openid4vc::disclosure_session::VpClientError;
    pub use openid4vc::disclosure_session::VpMessageClientError;
    pub use openid4vc::disclosure_session::VpMessageClientErrorType;
    pub use openid4vc::errors::AuthorizationErrorCode;
    pub use openid4vc::errors::ErrorResponse;
    pub use openid4vc::issuance_session::IssuanceSessionError;
    pub use openid4vc::oidc::OidcError;
}

pub use crate::account_provider::AccountProviderError;
pub use crate::account_provider::AccountProviderResponseError;
pub use crate::config::ConfigurationError;
pub use crate::disclosure::DisclosureUriError;
pub use crate::instruction::InstructionError;
pub use crate::instruction::RemoteEcdsaKeyError;
pub use crate::issuance::DigidSessionError;
pub use crate::pin::change::ChangePinError;
pub use crate::pin::key::PinKeyError;
pub use crate::pin::validation::PinValidationError;
pub use crate::repository::HttpClientError;
pub use crate::storage::KeyFileError;
pub use crate::storage::StorageError;
pub use crate::update_policy::UpdatePolicyError;
pub use crate::wallet::DisclosureBasedIssuanceError;
pub use crate::wallet::DisclosureError;
pub use crate::wallet::HistoryError;
pub use crate::wallet::IssuanceError;
pub use crate::wallet::ResetError;
pub use crate::wallet::UriIdentificationError;
pub use crate::wallet::WalletInitError;
pub use crate::wallet::WalletRegistrationError;
pub use crate::wallet::WalletUnlockError;
