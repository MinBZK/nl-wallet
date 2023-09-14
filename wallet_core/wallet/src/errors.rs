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
    pid_issuer::PidIssuerError,
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{InstructionError, PidIssuanceError, WalletInitError, WalletRegistrationError, WalletUnlockError},
};
