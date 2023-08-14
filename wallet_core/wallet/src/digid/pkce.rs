const CODE_VERIFIER_LENGTH: usize = 128;

/// This trait is used to isolate the [`pkce`] dependency.
#[cfg_attr(test, mockall::automock)]
pub trait PkceSource {
    /// Generate a PKCE verifier and code challenge pair.
    fn generate_verifier_and_challenge() -> (String, String);
}

pub struct PkceGenerator;

impl PkceSource for PkceGenerator {
    fn generate_verifier_and_challenge() -> (String, String) {
        // Generate a random 128-byte code verifier (must be between 43 and 128 bytes)
        let code_verifier = pkce::code_verifier(CODE_VERIFIER_LENGTH);

        // Generate an encrypted code challenge accordingly
        let code_challenge = pkce::code_challenge(&code_verifier);

        // Generate PKCE verifier
        let pkce_verifier = String::from_utf8(code_verifier).expect("Generated PKCE verifier is not valid UTF-8");

        (pkce_verifier, code_challenge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This simply tests the ingration with the [`pkce`] crate.
    /// No assumptions are made about the returned values, besides
    /// them being non-empty strings.
    #[test]
    fn test_pkce_generator_generate_verifier_and_challenge() {
        let (pkce_verifier1, code_challenge1) = PkceGenerator::generate_verifier_and_challenge();

        assert!(!pkce_verifier1.is_empty());
        assert!(!code_challenge1.is_empty());
        assert_ne!(pkce_verifier1, code_challenge1);

        let (pkce_verifier2, code_challenge2) = PkceGenerator::generate_verifier_and_challenge();

        assert!(!pkce_verifier2.is_empty());
        assert!(!code_challenge2.is_empty());
        assert_ne!(pkce_verifier2, code_challenge2);
        assert_ne!(pkce_verifier1, pkce_verifier2);
        assert_ne!(code_challenge1, code_challenge2);
    }
}
