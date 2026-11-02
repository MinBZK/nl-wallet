use coset::Label;
use coset::iana;
use crypto::x509::CertificateParseError;
use crypto::x509::CertificateVerifyError;
use crypto::x509::crl::CertificateCrlVerificationError;
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

    #[error("mismatch between COSE algorithm and public key: {0:?}")]
    #[category(pd)]
    MismatchAlgorithmWithPublicKey(iana::Algorithm),

    #[error("ESP256 signature verification failed: {0}")]
    #[category(pd)]
    Esp256SignatureVerificationFailed(#[source] p256::ecdsa::Error),

    #[error("ESP384 signature verification failed: {0}")]
    #[category(pd)]
    Esp384SignatureVerificationFailed(#[source] p384::ecdsa::Error),

    #[error("RSA PKCS1v15 signature verification failed: {0}")]
    #[category(pd)]
    RsaPkcs1v15SignatureVerificationFailed(#[source] rsa::signature::Error),

    #[error("RSA-PSS signature verification failed: {0}")]
    #[category(pd)]
    RsaPssSignatureVerificationFailed(#[source] rsa::signature::Error),

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

    #[error("certificate parse error: {0}")]
    CertificateParse(#[source] CertificateParseError),

    #[error("certificate verify error: {0}")]
    CertificateVerify(#[source] CertificateVerifyError),

    #[error("certificate CRL verification error: {0}")]
    CertificateCrl(#[source] CertificateCrlVerificationError),

    #[error("signing failed: {0}")]
    #[category(pd)]
    Signing(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error("no signature received")]
    #[category(critical)]
    SignatureMissing,

    #[error("incorrect number of signatures received, expected: {expected}, received: {received}")]
    #[category(critical)]
    IncorrectSignatureCount { expected: usize, received: usize },
}

#[cfg(test)]
mod tests {
    use error_category::Category;

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
        assert_eq!(
            CoseError::IncorrectSignatureCount {
                expected: 2,
                received: 1
            }
            .category(),
            Category::Critical
        );
    }
}
