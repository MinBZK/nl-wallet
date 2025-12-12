use axum::response::IntoResponse;
use axum::response::Response;
use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use derive_more::FromStr;
use http::StatusCode;
use metrics::counter;

use hsm::service::HsmError;
use http_utils::error::HttpJsonError;
use http_utils::error::HttpJsonErrorType;
use wallet_account::messages::errors::AccountError;
use wallet_account::messages::errors::AccountErrorType;
use wallet_provider_service::account_server::ChallengeError;
use wallet_provider_service::account_server::InstructionError;
use wallet_provider_service::account_server::RegistrationError;
use wallet_provider_service::account_server::WalletCertificateError;
use wallet_provider_service::wua_issuer::HsmWuaIssuerError;

// Make a newtype to circumvent the orphan rule.
#[derive(Debug, Clone, From, AsRef, Display, FromStr)]
pub struct WalletProviderErrorType(AccountErrorType);

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
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
    Wua(#[from] HsmWuaIssuerError),
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
                InstructionError::Validation(_)
                | InstructionError::NonExistingKey(_)
                | InstructionError::UnknownPidAttestationType(_)
                | InstructionError::MissingRecoveryCode
                | InstructionError::InvalidRecoveryCode
                | InstructionError::AttributesConversion(_)
                | InstructionError::AccountNotTransferable
                | InstructionError::NoAccountTransferInProgress
                | InstructionError::AccountTransferWalletsMismatch
                | InstructionError::AccountTransferIllegalState
                | InstructionError::AccountTransferCanceled
                | InstructionError::AppVersionMismatch { .. }
                | InstructionError::SdJwtError(_)
                | InstructionError::PinRecoveryAccountMismatch => Self::InstructionValidation,
                InstructionError::WalletCertificate(WalletCertificateError::UserBlocked) => Self::AccountBlocked,
                InstructionError::Signing(_)
                | InstructionError::Storage(_)
                | InstructionError::WalletCertificate(_)
                | InstructionError::WuaIssuance(_)
                | InstructionError::HsmError(_)
                | InstructionError::Poa(_)
                | InstructionError::PopSigning(_)
                | InstructionError::JwkConversion(_)
                | InstructionError::ObtainStatusClaim(_) => Self::Unexpected,
            },
            WalletProviderError::Hsm(_) => Self::Unexpected,
            WalletProviderError::Wua(_) => Self::Unexpected,
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
        register_error_metric(&self);
        HttpJsonError::<WalletProviderErrorType>::from(self).into_response()
    }
}

fn register_error_metric(error: &WalletProviderError) {
    let inner_error: &'static str = match error {
        WalletProviderError::Challenge(inner) => inner.into(),
        WalletProviderError::Registration(inner) => inner.into(),
        WalletProviderError::Instruction(inner) => inner.into(),
        WalletProviderError::Hsm(inner) => inner.into(),
        WalletProviderError::Wua(inner) => inner.into(),
    };

    let error: &'static str = error.into();
    counter!(
        "nlwallet_error_response",
        "service" => "wallet_provider",
        "error" => error,
        "inner_error" => inner_error
    )
    .increment(1);
}

#[cfg(test)]
mod tests {
    #![expect(clippy::type_complexity)]

    use std::sync::Arc;
    use std::sync::Mutex;

    use metrics::Counter;
    use metrics::CounterFn;
    use metrics::Gauge;
    use metrics::Histogram;
    use metrics::Key;
    use metrics::KeyName;
    use metrics::Metadata;
    use metrics::Recorder;
    use metrics::SharedString;
    use metrics::Unit;

    use super::*;

    struct MockCounter {
        labels: Vec<(String, String)>,
        counters: Arc<Mutex<Vec<Vec<(String, String)>>>>,
    }

    impl CounterFn for MockCounter {
        fn increment(&self, _value: u64) {
            self.counters.lock().unwrap().push(self.labels.clone());
        }

        fn absolute(&self, _value: u64) {
            unimplemented!()
        }
    }

    #[derive(Clone, Default)]
    struct MockRecorder {
        counters: Arc<Mutex<Vec<Vec<(String, String)>>>>,
    }

    impl Recorder for MockRecorder {
        fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}
        fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}
        fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

        fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
            let labels: Vec<_> = key
                .labels()
                .map(|label| (label.key().to_string(), label.value().to_string()))
                .collect();

            Counter::from_arc(Arc::new(MockCounter {
                labels,
                counters: Arc::clone(&self.counters),
            }))
        }

        fn register_gauge(&self, _key: &Key, _metadata: &Metadata<'_>) -> Gauge {
            unimplemented!()
        }
        fn register_histogram(&self, _key: &Key, _metadata: &Metadata<'_>) -> Histogram {
            unimplemented!()
        }
    }

    #[test]
    fn test_register_error_metric() {
        let recorder = MockRecorder::default();
        let error =
            WalletProviderError::Challenge(ChallengeError::WalletCertificate(WalletCertificateError::UserBlocked));

        metrics::with_local_recorder(&recorder, || {
            register_error_metric(&error);
        });

        let counters = recorder.counters.lock().unwrap();
        assert_eq!(counters.len(), 1);

        let labels = &counters[0];
        assert!(
            labels.iter().any(|(key, val)| key == "error" && val == "Challenge"),
            "Missing or incorrect error label. Got labels: {:?}",
            labels
        );
        assert!(
            labels
                .iter()
                .any(|(key, val)| key == "inner_error" && val == "WalletCertificate"),
            "Missing or incorrect inner_error label. Got labels: {:?}",
            labels
        );
    }
}
