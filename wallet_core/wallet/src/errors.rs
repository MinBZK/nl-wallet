// Expose some downstream errors that may be useful.
pub use openid::error::Error as OpenIdError;
pub use reqwest::Error as ReqwestError;

pub use crate::{
    account_provider::{AccountProviderError, AccountProviderResponseError},
    digid::{DigidAuthenticatorError, OpenIdAuthenticatorError},
    pid_issuer::PidRetrieverError,
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{PidIssuanceError, WalletInitError, WalletRegistrationError, WalletUnlockError},
};
