use rustls_pki_types::AlgorithmIdentifier;
use rustls_pki_types::SignatureVerificationAlgorithm;
use webpki::ring::ECDSA_P256_SHA256;

const ECDSA_SHA256_WITH_NULL_PARAMETERS: &[u8] =
    &[0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x04, 0x03, 0x02, 0x05, 0x00];

/// This algorithm enables key attestations for Pixel 3a phones.
pub static ECDSA_P256_SHA256_WITH_NULL_PARAMETERS: &dyn SignatureVerificationAlgorithm =
    &WrappedSignatureVerificationAlgorithm::wrap_with(ECDSA_P256_SHA256, ECDSA_SHA256_WITH_NULL_PARAMETERS);

/// Wraps a SignatureVerificationAlgorithm with a different signature algorithm identifier.
/// We use this to support AlgorithmIdentifiers that include ANY NULL parameters instead of absense of the field.
#[derive(Debug)]
struct WrappedSignatureVerificationAlgorithm<'a> {
    wrapped: &'a dyn SignatureVerificationAlgorithm,
    signature_alg_id: AlgorithmIdentifier,
}

impl<'a> WrappedSignatureVerificationAlgorithm<'a> {
    pub const fn wrap_with(wrapped: &'a dyn SignatureVerificationAlgorithm, signature_alg_id: &'static [u8]) -> Self {
        WrappedSignatureVerificationAlgorithm {
            wrapped,
            signature_alg_id: AlgorithmIdentifier::from_slice(signature_alg_id),
        }
    }
}

impl SignatureVerificationAlgorithm for WrappedSignatureVerificationAlgorithm<'_> {
    fn verify_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), rustls_pki_types::InvalidSignature> {
        self.wrapped.verify_signature(public_key, message, signature)
    }

    fn public_key_alg_id(&self) -> AlgorithmIdentifier {
        self.wrapped.public_key_alg_id()
    }

    fn signature_alg_id(&self) -> AlgorithmIdentifier {
        self.signature_alg_id
    }

    fn fips(&self) -> bool {
        self.wrapped.fips()
    }
}

#[cfg(test)]
mod tests {
    use rustls_pki_types::AlgorithmIdentifier;
    use rustls_pki_types::SignatureVerificationAlgorithm;

    use super::WrappedSignatureVerificationAlgorithm;

    const EMPTY: &[u8] = &[];
    const EMPTY_WITH_NULL_PARAMETERS: &[u8] = &[5, 0];

    #[derive(Debug)]
    struct MockSignatureVerificationAlgorithm {
        public_key_alg_id: AlgorithmIdentifier,
        signature_alg_id: AlgorithmIdentifier,
    }

    impl MockSignatureVerificationAlgorithm {
        fn empty() -> Self {
            Self {
                public_key_alg_id: AlgorithmIdentifier::from_slice(EMPTY),
                signature_alg_id: AlgorithmIdentifier::from_slice(EMPTY),
            }
        }
    }

    impl SignatureVerificationAlgorithm for MockSignatureVerificationAlgorithm {
        fn verify_signature(
            &self,
            _public_key: &[u8],
            _message: &[u8],
            _signature: &[u8],
        ) -> Result<(), rustls_pki_types::InvalidSignature> {
            Ok(())
        }

        fn public_key_alg_id(&self) -> AlgorithmIdentifier {
            self.public_key_alg_id
        }

        fn signature_alg_id(&self) -> AlgorithmIdentifier {
            self.signature_alg_id
        }

        fn fips(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_wrapped_signature_verification_algorithm() {
        let empty_algorithm = MockSignatureVerificationAlgorithm::empty();
        let actual = WrappedSignatureVerificationAlgorithm::wrap_with(&empty_algorithm, EMPTY_WITH_NULL_PARAMETERS);

        assert!(actual.verify_signature(EMPTY, EMPTY, EMPTY).is_ok());
        assert_eq!(actual.public_key_alg_id(), empty_algorithm.public_key_alg_id());
        assert_ne!(actual.signature_alg_id(), empty_algorithm.signature_alg_id());
        assert_eq!(
            actual.signature_alg_id(),
            AlgorithmIdentifier::from_slice(EMPTY_WITH_NULL_PARAMETERS)
        );
        assert_eq!(actual.fips(), empty_algorithm.fips());
    }
}
