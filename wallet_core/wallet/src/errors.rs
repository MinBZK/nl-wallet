// Expose some downstream errors that may be useful.
pub mod openid {
    pub use openid::error::Error;
}

pub mod reqwest {
    pub use reqwest::Error;
}

pub use crate::{
    account_provider::{AccountProviderError, AccountProviderResponseError},
    config::{ConfigurationError, FileStorageError},
    digid::{DigidError, OpenIdError},
    document::{AttributeValueType, DocumentMdocError},
    instruction::{InstructionError, RemoteEcdsaKeyError},
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{
        DisclosureError, EventConversionError, EventStorageError, HistoryError, PidIssuanceError, ResetError,
        UriIdentificationError, WalletInitError, WalletRegistrationError, WalletUnlockError,
    },
};
