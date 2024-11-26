use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use base64::prelude::*;
use derive_more::Debug;
use rustls_pki_types::CertificateDer;
use serde::Deserialize;
use serde::Serialize;
use webpki::anchor_from_trusted_cert;
use webpki::types::TrustAnchor;
use webpki::Error;
use x509_parser::error::X509Error;
use x509_parser::prelude::FromDer;
use x509_parser::x509::RelativeDistinguishedName;
use yoke::Yoke;
use yoke::Yokeable;

#[derive(Yokeable, Debug, Clone, Eq, PartialEq)]
struct ParsedTrustAnchor<'a> {
    trust_anchor: TrustAnchor<'a>,
}

type YokedTrustAnchor = Yoke<ParsedTrustAnchor<'static>, Arc<CertificateDer<'static>>>;

#[derive(Debug, Clone)]
pub struct BorrowingTrustAnchor(YokedTrustAnchor);

impl Hash for BorrowingTrustAnchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.backing_cart().hash(state);
    }
}

impl BorrowingTrustAnchor {
    pub fn from_der(der_bytes: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let certificate_der = CertificateDer::from(der_bytes.into());
        let yoke = Yoke::try_attach_to_cart(Arc::from(certificate_der), |cert| {
            let trust_anchor = anchor_from_trusted_cert(cert)?;
            Ok(ParsedTrustAnchor { trust_anchor })
        })?;

        Ok(BorrowingTrustAnchor(yoke))
    }

    pub fn get(&self) -> &TrustAnchor {
        &self.0.get().trust_anchor
    }

    pub fn trust_anchor_names(&self) -> Result<Vec<String>, x509_parser::nom::Err<X509Error>> {
        let (_, names) = RelativeDistinguishedName::from_der(self.get().subject.as_ref())?;

        let names = names
            .iter()
            .filter_map(|name| name.as_str().ok().map(str::to_string))
            .collect();

        Ok(names)
    }
}

impl AsRef<[u8]> for BorrowingTrustAnchor {
    fn as_ref(&self) -> &[u8] {
        self.0.backing_cart().as_ref()
    }
}

impl PartialEq for BorrowingTrustAnchor {
    fn eq(&self, other: &Self) -> bool {
        self.0.get() == other.0.get()
    }
}

impl Eq for BorrowingTrustAnchor {}

impl<'a> From<&'a BorrowingTrustAnchor> for TrustAnchor<'a> {
    fn from(value: &'a BorrowingTrustAnchor) -> Self {
        value.get().clone()
    }
}

impl Serialize for BorrowingTrustAnchor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let cert = self.as_ref();
        if serializer.is_human_readable() {
            BASE64_STANDARD.encode(cert).serialize(serializer)
        } else {
            cert.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for BorrowingTrustAnchor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let der_bytes = if deserializer.is_human_readable() {
            BASE64_STANDARD
                .decode(String::deserialize(deserializer)?)
                .map_err(serde::de::Error::custom)
        } else {
            Deserialize::deserialize(deserializer)
        }?;

        BorrowingTrustAnchor::from_der(der_bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::trust_anchor::BorrowingTrustAnchor;

    #[test]
    fn der_trust_anchor_serialization() {
        let anchor_str = "MIIBkzCCATqgAwIBAgIUOCjkeBboSUVO3A+Wq8Xb4Ize3twwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwOY2EuZXhhbXBs\
            ZS5jb20wHhcNMjMxMTE3MDc1OTQzWhcNMjQxMTE2MDc1OTQzWjAZMRcwFQYDVQQDDA5jYS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCC\
            qGSM49AwEHA0IABMwoWnLasOGW6ogQ0TeojJTOAQirhLkxX0rqWGXe97sb6LrfsUGx5URdzNhXO8REBZyhszEH+xrYEX5hBPGvXnOjYDBe\
            MB0GA1UdDgQWBBS6toHYF2P6gnKEnMjYuXRvqwFLmTAfBgNVHSMEGDAWgBS6toHYF2P6gnKEnMjYuXRvqwFLmTAPBgNVHRMBAf8EBTADAQ\
            H/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNHADBEAiB16lDCCRPtST/h3mYM86V7FhodF47j0OZWY57jmDxstQIgQHt8XU2CYYCSSt42\
            nw4CJrY9QCwosFay0VSMh9nqUMA=";

        let json_anchor_str = format!("\"{}\"", anchor_str);

        let deserialized: BorrowingTrustAnchor = serde_json::from_str(&json_anchor_str).unwrap();
        let serialized_anchor = serde_json::to_string(&deserialized).unwrap();

        assert_eq!(json_anchor_str, serialized_anchor.as_str());
    }
}
