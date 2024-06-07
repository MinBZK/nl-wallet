// Expose some downstream errors that may be useful.
pub mod reqwest {
    pub use reqwest::Error;
}

pub mod mdoc {
    pub use nl_wallet_mdoc::{holder::HolderError, Error};
}

pub mod openid4vc {
    pub use openid4vc::{issuance_session::IssuanceSessionError, oidc::OidcError};
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
