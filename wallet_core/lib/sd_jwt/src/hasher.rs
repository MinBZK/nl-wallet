// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use base64::prelude::*;

use crypto::utils::sha256;

pub const SHA_ALG_NAME: &str = "sha-256";

/// Used to implement hash functions to be used for encoding/decoding.
///
/// ## Note
///
/// Implementations of this trait are expected only for algorithms listed in
/// the IANA "Named Information Hash Algorithm" registry.
/// See [Hash Function Claim](https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-hash-function-claim)
pub trait Hasher {
    /// Digests input to produce unique fixed-size hash value in bytes.
    fn digest(&self, input: &[u8]) -> Vec<u8>;

    /// Returns the name of hash function used.
    ///
    /// ## Note
    ///
    /// The hash algorithm identifier MUST be a hash algorithm value from the
    /// "Hash Name String" column in the IANA "Named Information Hash Algorithm"  
    fn alg_name(&self) -> &str;

    /// Returns the base64url-encoded digest of a `disclosure`.
    fn encoded_digest(&self, disclosure: &str) -> String {
        let hash = self.digest(disclosure.as_bytes());
        BASE64_URL_SAFE_NO_PAD.encode(hash)
    }
}

/// An implementation of [`Hasher`] that uses the `sha-256` hash function.
#[derive(Default, Clone, Copy, Debug)]
pub struct Sha256Hasher;

impl Sha256Hasher {
    /// Creates a new [`ShaHasher`]
    pub fn new() -> Self {
        Sha256Hasher {}
    }
}

impl Hasher for Sha256Hasher {
    fn digest(&self, input: &[u8]) -> Vec<u8> {
        sha256(input)
    }

    fn alg_name(&self) -> &'static str {
        SHA_ALG_NAME
    }
}

// Some test values taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures
#[cfg(test)]
mod test {
    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;

    #[test]
    fn test1() {
        let disclosure = "WyI2cU1RdlJMNWhhaiIsICJmYW1pbHlfbmFtZSIsICJNw7ZiaXVzIl0";
        let hasher = Sha256Hasher::new();
        let hash = hasher.encoded_digest(disclosure);
        assert_eq!("uutlBuYeMDyjLLTpf6Jxi7yNkEF35jdyWMn9U7b_RYY", hash);
    }

    #[test]
    fn test2() {
        let disclosure =
            "WyJlSThaV205UW5LUHBOUGVOZW5IZGhRIiwgImVtYWlsIiwgIlwidW51c3VhbCBlbWFpbCBhZGRyZXNzXCJAZXhhbXBsZS5qcCJd";
        let hasher = Sha256Hasher::new();
        let hash = hasher.encoded_digest(disclosure);
        assert_eq!("Kuet1yAa0HIQvYnOVd59hcViO9Ug6J2kSfqYRBeowvE", hash);
    }

    #[test]
    fn test3() {
        let disclosure = "WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgIkZSIl0";
        let hasher = Sha256Hasher::new();
        let hash = hasher.encoded_digest(disclosure);
        assert_eq!("w0I8EKcdCtUPkGCNUrfwVp2xEgNjtoIDlOxc9-PlOhs", hash);
    }
}
