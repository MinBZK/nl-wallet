use coset::Label;
use crypto::x509::CertificateError;
use error_category::ErrorCategory;

use crate::serialization::CborError;

#[derive(thiserror::Error, Debug, ErrorCategory)]
#[category(defer)]
pub enum CoseError {
    #[error("missing payload")]
    #[category(critical)]
    MissingPayload,
    #[error("missing label {0:?}")]
    #[category(pd)]
    MissingLabel(Label),
    #[error("missing protected COSE algorithm")]
    #[category(critical)]
    MissingAlgorithm,
    #[error("unsupported protected COSE algorithm: {0:?}")]
    #[category(pd)]
    UnsupportedAlgorithm(coset::Algorithm),
    #[error("ECDSA signature parsing failed: {0}")]
    #[category(pd)]
    EcdsaSignatureParsingFailed(#[source] p256::ecdsa::Error),
    #[error("ECDSA signature verification failed: {0}")]
    #[category(pd)]
    EcdsaSignatureVerificationFailed(#[source] p256::ecdsa::Error),
    #[error("MAC verification failed")]
    #[category(critical)]
    MacVerificationFailed,
    #[error("CBOR error: {0}")]
    Cbor(#[source] CborError),
    #[error("signing certificate header did not contain bytes")]
    #[category(critical)]
    CertificateUnexpectedHeaderType,
    #[error("x5chain certificate chain is empty")]
    #[category(critical)]
    EmptyCertificateChain,
    #[error("x5chain certificate array must contain at least two certificates, found {0}")]
    #[category(critical)]
    CertificateChainTooShort(usize),
    #[error("certificate error: {0}")]
    Certificate(#[source] CertificateError),
    #[error("signing failed: {0}")]
    #[category(pd)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("no signature received")]
    #[category(critical)]
    SignatureMissing,
}

#[cfg(test)]
mod tests {
    use error_category::Category;
    use error_category::ErrorCategory as _;

    use super::*;

    #[test]
    fn errors_with_header_values_are_personal_data() {
        for error in [
            CoseError::MissingLabel(Label::Text("personal data".to_owned())),
            CoseError::UnsupportedAlgorithm(coset::Algorithm::Text("personal data".to_owned())),
        ] {
            assert_eq!(error.category(), Category::PersonalData);
        }
    }

    #[test]
    fn structural_errors_without_input_values_are_critical() {
        assert_eq!(CoseError::MissingPayload.category(), Category::Critical);
        assert_eq!(CoseError::MissingAlgorithm.category(), Category::Critical);
    }
}
