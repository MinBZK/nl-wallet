// Expose some downstream errors that may be useful.
pub mod reqwest {
    pub use reqwest::Error;
}

pub use crate::{
    account_provider::{AccountProviderError, AccountProviderResponseError},
    config::{ConfigurationError, FileStorageError},
    disclosure::DisclosureUriError,
    document::{AttributeValueType, DocumentMdocError},
    instruction::{InstructionError, RemoteEcdsaKeyError},
    issuance::DigidSessionError,
    pin::{key::PinKeyError, validation::PinValidationError},
    storage::{KeyFileError, StorageError},
    wallet::{
        DisclosureError, EventConversionError, EventStorageError, HistoryError, PidIssuanceError, ResetError,
        UriIdentificationError, WalletInitError, WalletRegistrationError, WalletUnlockError,
    },
};
