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
    instruction::{InstructionError, RemoteEcdsaKeyError},
    pid_issuer::PidIssuerError,
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{PidIssuanceError, WalletInitError, WalletRegistrationError, WalletUnlockError},
};
