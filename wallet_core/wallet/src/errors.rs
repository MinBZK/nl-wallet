// Expose some downstream errors that may be useful.
pub mod openid {
    pub use openid::error::Error;
}

pub mod reqwest {
    pub use reqwest::Error;
}

pub use crate::{
    account_provider::{AccountProviderError, AccountProviderResponseError},
    digid::{DigidError, OpenIdError},
    pid_issuer::PidRetrieverError,
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{PidIssuanceError, WalletInitError, WalletRegistrationError, WalletUnlockError},
};
