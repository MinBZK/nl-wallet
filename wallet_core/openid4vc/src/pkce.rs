use base64::prelude::*;

use crypto::utils::random_string;
use crypto::utils::sha256;

/// The maximum length for a PKCE verifier is 128 characters.
const CODE_VERIFIER_LENGTH: usize = 128;

/// This represents a code verifier and code challenge pair for
/// a PKCE exchange.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PkcePair {
    /// The code challenge method implemented by this pair.
    const CODE_CHALLENGE_METHOD: &'static str = "INVALID";

    /// Randomly generate a PKCE code verifier and its accompanying challenge.
    fn generate() -> Self
    where
        Self: Sized;

    /// The code verifier of this pair, which is a random string of characters.
    fn into_code_verifier(self) -> String;

    /// The code challenge of this pair, which is base64 encoded.
    fn code_challenge(&self) -> &str;
}

// The tuple contains the code verifier and code challenge, in order.
pub struct S256PkcePair(String, String);

impl PkcePair for S256PkcePair {
    const CODE_CHALLENGE_METHOD: &'static str = "S256";

    fn generate() -> Self
    where
        Self: Sized,
    {
        // This randomly generated string only uses alphanumeric
        // characters and does not use 4 of the characters allowed
        // by the standard.
        let code_verifier = random_string(CODE_VERIFIER_LENGTH);

        let hash = sha256(code_verifier.as_bytes());
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(hash);

        Self(code_verifier, code_challenge)
    }

    fn into_code_verifier(self) -> String {
        self.0
    }

    fn code_challenge(&self) -> &str {
        &self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s256_pkce_pair() {
        let pair = S256PkcePair::generate();

        let challenge = pair.code_challenge().to_string();
        let verifier = pair.into_code_verifier();

        assert_eq!(verifier.len(), CODE_VERIFIER_LENGTH);

        assert_eq!(
            sha256(verifier.as_bytes()),
            BASE64_URL_SAFE_NO_PAD.decode(challenge).unwrap()
        );
    }
}
