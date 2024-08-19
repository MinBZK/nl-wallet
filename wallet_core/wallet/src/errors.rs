// Expose some downstream errors that may be useful.
pub mod reqwest {
    pub use reqwest::Error;
}

pub mod openid4vc {
    pub use openid4vc::{
        disclosure_session::{VpClientError, VpMessageClientError, VpMessageClientErrorType},
        issuance_session::IssuanceSessionError,
        oidc::OidcError,
    };
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
