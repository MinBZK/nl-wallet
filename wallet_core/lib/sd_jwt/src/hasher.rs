// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use base64::prelude::*;

use crypto::utils::sha256;

use crate::sd_alg::SdAlg;

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
    fn alg(&self) -> SdAlg;

    /// Returns the base64url-encoded digest of a `disclosure`.
    fn encoded_digest(&self, disclosure: &str) -> String {
        let hash = self.digest(disclosure.as_bytes());
        BASE64_URL_SAFE_NO_PAD.encode(hash)
    }
}

/// An implementation of [`Hasher`] that uses the `sha-256` hash function.
#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn digest(&self, input: &[u8]) -> Vec<u8> {
        sha256(input)
    }

    fn alg(&self) -> SdAlg {
        SdAlg::Sha256
    }
}

// Examples taken from <https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-07.html#name-disclosures>
#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::hasher::Hasher;
    use crate::hasher::Sha256Hasher;
    use crate::sd_alg::SdAlg;

    #[rstest]
    #[case(
        Sha256Hasher,
        "WyI2cU1RdlJMNWhhaiIsICJmYW1pbHlfbmFtZSIsICJNw7ZiaXVzIl0",
        "uutlBuYeMDyjLLTpf6Jxi7yNkEF35jdyWMn9U7b_RYY"
    )]
    #[case(
        Sha256Hasher,
        "WyJlSThaV205UW5LUHBOUGVOZW5IZGhRIiwgImVtYWlsIiwgIlwidW51c3VhbCBlbWFpbCBhZGRyZXNzXCJAZXhhbXBsZS5qcCJd",
        "Kuet1yAa0HIQvYnOVd59hcViO9Ug6J2kSfqYRBeowvE"
    )]
    #[case(
        Sha256Hasher,
        "WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgIkZSIl0",
        "w0I8EKcdCtUPkGCNUrfwVp2xEgNjtoIDlOxc9-PlOhs"
    )]
    #[case(
        SdAlg::Sha256.hasher().unwrap(),
        "WyI2cU1RdlJMNWhhaiIsICJmYW1pbHlfbmFtZSIsICJNw7ZiaXVzIl0",
        "uutlBuYeMDyjLLTpf6Jxi7yNkEF35jdyWMn9U7b_RYY"
    )]
    #[case(
        SdAlg::Sha256.hasher().unwrap(),
        "WyJlSThaV205UW5LUHBOUGVOZW5IZGhRIiwgImVtYWlsIiwgIlwidW51c3VhbCBlbWFpbCBhZGRyZXNzXCJAZXhhbXBsZS5qcCJd",
        "Kuet1yAa0HIQvYnOVd59hcViO9Ug6J2kSfqYRBeowvE"
    )]
    #[case(
        SdAlg::Sha256.hasher().unwrap(),
        "WyJsa2x4RjVqTVlsR1RQVW92TU5JdkNBIiwgIkZSIl0",
        "w0I8EKcdCtUPkGCNUrfwVp2xEgNjtoIDlOxc9-PlOhs"
    )]
    fn test_hasher(#[case] hasher: impl Hasher, #[case] disclosure: &str, #[case] expected_hash: &str) {
        let hash = hasher.encoded_digest(disclosure);
        assert_eq!(hash, expected_hash);
    }
}
