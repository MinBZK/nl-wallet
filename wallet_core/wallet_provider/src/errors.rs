use axum::response::IntoResponse;
use axum::response::Response;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use http::StatusCode;

use hsm::service::HsmError;
use wallet_account::messages::errors::AccountError;
use wallet_account::messages::errors::AccountErrorType;
use wallet_common::http_error::HttpJsonError;
use wallet_common::http_error::HttpJsonErrorType;
use wallet_provider_service::account_server::ChallengeError;
use wallet_provider_service::account_server::InstructionError;
use wallet_provider_service::account_server::RegistrationError;
use wallet_provider_service::account_server::WalletCertificateError;
use wallet_provider_service::wte_issuer::HsmWteIssuerError;

// Make a newtype to circumvent the orphan rule.
#[derive(Debug, Clone, From, AsRef, Display, FromStr)]
pub struct WalletProviderErrorType(AccountErrorType);

#[derive(Debug, thiserror::Error)]
pub enum WalletProviderError {
    #[error("{0}")]
    Challenge(#[from] ChallengeError),
    #[error("{0}")]
    Registration(#[from] RegistrationError),
    #[error("{0}")]
    Instruction(#[from] InstructionError),
    #[error("{0}")]
    Hsm(#[from] HsmError),
    #[error("{0}")]
    Wte(#[from] HsmWteIssuerError),
}

impl HttpJsonErrorType for WalletProviderErrorType {
    fn title(&self) -> String {
        let title = match self.as_ref() {
            AccountErrorType::Unexpected => "An unexpected error occurred",
            AccountErrorType::ChallengeValidation => "Could not validate registration challenge",
            AccountErrorType::AttestationValidation => "Could not validate key / app attestation",
            AccountErrorType::RegistrationParsing => "Could not parse or validate registration message",
            AccountErrorType::IncorrectPin => "The PIN provided is incorrect",
            AccountErrorType::PinTimeout => "PIN checking is currently in timeout",
            AccountErrorType::AccountBlocked => "The requested account is blocked",
            AccountErrorType::InstructionValidation => "Could not validate instruction",
        };

        title.to_string()
    }

    fn status_code(&self) -> StatusCode {
        match self.as_ref() {
            AccountErrorType::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
            AccountErrorType::ChallengeValidation => StatusCode::UNAUTHORIZED,
            AccountErrorType::AttestationValidation => StatusCode::UNAUTHORIZED,
            AccountErrorType::RegistrationParsing => StatusCode::BAD_REQUEST,
            AccountErrorType::IncorrectPin => StatusCode::FORBIDDEN,
            AccountErrorType::PinTimeout => StatusCode::FORBIDDEN,
            AccountErrorType::AccountBlocked => StatusCode::UNAUTHORIZED,
            AccountErrorType::InstructionValidation => StatusCode::FORBIDDEN,
        }
    }
}

impl From<WalletProviderError> for AccountError {
    fn from(value: WalletProviderError) -> Self {
        match value {
            WalletProviderError::Challenge(error) => match error {
                ChallengeError::WalletCertificate(WalletCertificateError::UserBlocked) => Self::AccountBlocked,
                ChallengeError::WalletCertificate(_) => Self::ChallengeValidation,
                _ => Self::ChallengeValidation,
            },
            WalletProviderError::Registration(error) => match error {
                RegistrationError::ChallengeDecoding(_) => Self::ChallengeValidation,
                RegistrationError::ChallengeValidation(_) => Self::ChallengeValidation,
                RegistrationError::AppleAttestation(_) => Self::AttestationValidation,
                RegistrationError::AndroidKeyAttestation(_) => Self::AttestationValidation,
                RegistrationError::AndroidAppAttestation(_) => Self::AttestationValidation,
                RegistrationError::MessageParsing(_) => Self::RegistrationParsing,
                RegistrationError::MessageValidation(_) => Self::RegistrationParsing,
                RegistrationError::SerialNumberMismatch { .. } => Self::RegistrationParsing,
                RegistrationError::PinPubKeyEncoding(_) => Self::Unexpected,
                RegistrationError::CertificateStorage(_) => Self::Unexpected,
                RegistrationError::WalletCertificate(_) => Self::Unexpected,
                RegistrationError::HsmError(_) => Self::Unexpected,
            },
            WalletProviderError::Instruction(error) => match error {
                InstructionError::IncorrectPin(data) => Self::IncorrectPin(data),
                InstructionError::PinTimeout(data) => Self::PinTimeout(data),
                InstructionError::AccountBlocked => Self::AccountBlocked,
                InstructionError::Validation(_) | InstructionError::NonexistingKey(_) => Self::InstructionValidation,
                InstructionError::Signing(_)
                | InstructionError::Storage(_)
                | InstructionError::WalletCertificate(_)
                | InstructionError::WteIssuance(_)
                | InstructionError::HsmError(_)
                | InstructionError::Poa(_) => Self::Unexpected,
            },
            WalletProviderError::Hsm(_) => Self::Unexpected,
            WalletProviderError::Wte(_) => Self::Unexpected,
        }
    }
}

impl From<WalletProviderError> for HttpJsonError<WalletProviderErrorType> {
    fn from(value: WalletProviderError) -> Self {
        let detail = value.to_string();
        let account_error = AccountError::from(value);

        Self::new(
            AccountErrorType::from(&account_error).into(),
            detail,
            account_error.into(),
        )
    }
}

impl IntoResponse for WalletProviderError {
    fn into_response(self) -> Response {
        HttpJsonError::<WalletProviderErrorType>::from(self).into_response()
    }
}
