use std::str::FromStr;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use derive_more::Display;
use derive_more::From;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

/// The KeyIdentifier of a public key from a certificate.
///
/// This is a SHA-1 hash (RFC 5280) or equivalent 20 bytes of a SHA-2 hash (RFC 7093).
#[derive(derive_more::Debug, Clone, PartialEq, Eq, From, Display, SerializeDisplay, DeserializeFromStr)]
#[display("{}", BASE64_URL_SAFE_NO_PAD.encode(&self.0))]
#[debug("{}", self)]
pub struct KeyIdentifier(Vec<u8>);

impl FromStr for KeyIdentifier {
    type Err = base64::DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(KeyIdentifier(BASE64_URL_SAFE_NO_PAD.decode(s)?))
    }
}

#[cfg(test)]
mod tests {
    use p256::pkcs8::EncodePublicKey;

    use crate::server_keys::generate::Ca;
    use crate::utils::sha256;
    use crate::x509::BorrowingCertificate;

    #[test]
    fn parse_aki() {
        let ca = Ca::generate_mock();
        let certificate = BorrowingCertificate::from_certificate_der(ca.certificate().clone())
            .expect("self signed CA should contain a valid X.509 certificate");

        // `rcgen` computes the AKI as the first 20 bytes of the SHA256 of the DER encoding of the public key,
        // as per RFC 7093.
        let pubkey_der = ca
            .to_signing_key()
            .unwrap()
            .verifying_key()
            .to_public_key_der()
            .unwrap();
        let hash = sha256(pubkey_der.as_bytes());
        let hash = &hash[0..20];

        assert_eq!(hash, &certificate.authority_key_id().unwrap().0);
    }
}
