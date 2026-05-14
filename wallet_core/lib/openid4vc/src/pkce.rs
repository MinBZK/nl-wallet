use base64::prelude::*;
use chrono::Duration;
use crypto::utils::random_string;
use crypto::utils::sha256;

/// TTL for PKCE flow store entries. Bounds how long the user has to complete
/// the upstream authentication between `/authorize` and `/token`.
pub const PKCE_FLOW_TTL: Duration = Duration::minutes(10);

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
#[derive(Debug)]
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

        let code_challenge = Self::challenge_for(&code_verifier);

        Self(code_verifier, code_challenge)
    }

    fn into_code_verifier(self) -> String {
        self.0
    }

    fn code_challenge(&self) -> &str {
        &self.1
    }
}

impl S256PkcePair {
    pub fn challenge_for(verifier: &str) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(sha256(verifier.as_bytes()))
    }
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use crate::store::Store;

    pub async fn test_pkce_store<P, F>(store: P, mut count_entries: F)
    where
        P: Store<String, String>,
        F: AsyncFnMut(&P) -> usize,
    {
        let wallet_code_challenge = "test-wallet-code-challenge".to_string();
        let upstream_code_verifier = "test-upstream-code-verifier".to_string();

        // Store an entry and consume it.
        store
            .store(wallet_code_challenge.clone(), upstream_code_verifier.clone())
            .await
            .unwrap();
        assert_eq!(count_entries(&store).await, 1);

        let result = store.consume(wallet_code_challenge.as_str()).await.unwrap();
        assert_eq!(result.as_deref(), Some(upstream_code_verifier.as_str()));
        assert_eq!(count_entries(&store).await, 0);

        // Consuming the same entry a second time returns None.
        let result = store.consume(wallet_code_challenge.as_str()).await.unwrap();
        assert!(result.is_none());

        // Consuming an unknown challenge returns None.
        let result = store.consume("unknown-wallet-code-challenge").await.unwrap();
        assert!(result.is_none());

        // Cleanup runs without error.
        store.cleanup().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::store::MemoryStore;

    #[tokio::test]
    async fn test_memory_pkce_store() {
        let store: MemoryStore<String, String> = MemoryStore::new(Duration::seconds(60));
        test::test_pkce_store(store, async |s| s.len()).await;
    }

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
